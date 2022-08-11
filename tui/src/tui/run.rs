//! TODO!

use super::Res as Result;
use super::Tui;
use super::TuiTypes;
use super::events::{self, Event, WidgetEvent, FocusEvent, CrosstermEvent};
use super::widget::Widget;
use super::widget::WidgetTypesSpec;
use crate::strings::{s, HelloMsg, StartupMsg};

use lc3_traits::control::Control;
use lc3_application_support::event_loop::Backoff;

use anyhow::anyhow;
use chrono::{DateTime, offset::Local};
use crossterm::ExecutableCommand;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, DisableMouseCapture};
use tui::terminal::Terminal;
use tui::widgets::Text as TuiText;
use tui::layout::Rect;
use tui::style::{Color, Style};

use std::io::{Stdout, Write};
use std::panic;

specialize! {
    desktop => { use std::sync::mpsc::Sender; }
    web => { use futures_channel::mpsc::UnboundedSender as Sender; }
}

impl<'a, T: TuiTypes> Tui<'a, T> {
    // some one time initialization stuff
    fn init(&mut self) {
        // Say hello:
        self.data.log(s!(HelloMsg), Color::Cyan);
        self.data.log(s!(StartupMsg), Color::Magenta);

        // Copy over any existing breakpoints/watchpoints:
        self.data.sim.get_memory_watchpoints()
            .iter().copied()
            .filter_map(std::convert::identity)
            .enumerate()
            .for_each(|(idx, (addr, _))|
                assert!(self.data.wp.insert(addr, idx).is_none())
            );

        self.data.sim.get_breakpoints()
            .iter().copied()
            .filter_map(std::convert::identity)
            .enumerate()
            .for_each(|(idx, addr)|
                assert!(self.data.bp.insert(addr, idx).is_none())
            );
    }

    // Matches the interface `Backoff` has for the function it takes; return value
    // indicates whether to continue.
    fn handle_event<B>(
        &mut self,
        event: Event,
        term: &mut Terminal<B>,
        tx: &Sender<Event>,
        root: &mut impl Widget<WidgetTypesSpec<T, B>>,
        last_window_size: &mut Option<(u16, u16)>,
    ) -> bool
    where
        B: tui::backend::Backend,
        B: ExecutableCommand<&'static str>,
    {
        use Event::*;
        use CrosstermEvent::*;

        macro_rules! send {
            ($expr:expr) => {
                specialize! { [var: ___ex]
                    desktop => { tx.send($expr).unwrap() }
                    web => { tx.unbounded_send($expr).unwrap() }
                }
            };
        }

        // Empty the queue if we're told to. This should handle nested requests
        // (i.e. if we're told to empty the queue while we're already doing so).
        if let Some(f) = self.data.flush_all_events {
            use super::Flush::*;

            match f {
                Requested(i) => {
                    send!(FlushEventsBarrier(i));

                    self.data.flush_all_events = Some(Acknowledged(i));
                }
                Acknowledged(c) => {
                    match event {
                        FlushEventsBarrier(i) => {
                            // If its count matches us, we're done clearing.
                            if c == i {
                                drop(self.data.flush_all_events.take());
                                let Rect { width, height, .. } = term.size().unwrap();
                                send!(ActualEvent(Resize(width, height)));
                            }
                        }
                        _ => {
                            /* Otherwise, continue discarding events. */
                            if let Some(ref mut v) = self.data.debug_log {
                                let time = std::time::SystemTime::now();
                                let time: DateTime<Local> = time.into();
                                let line = format!("[EVENT] @ {}, discarded: {:?}\n", time.format("%d/%m/%Y %T%.6f"), event);

                                v.push(TuiText::styled(line, Style::default().fg(Color::Yellow)));
                            }
                        }
                    }
                }
            }

            return true;
        }

        // Next, log events.
        log::trace!("Event: {:?}", event);

        if let Some(ref mut v) = self.data.debug_log {
            let time = std::time::SystemTime::now();
            let time: DateTime<Local> = time.into();
            let line = format!("[EVENT] @ {}: {:?}\n", time.format("%d/%m/%Y %T%.6f"), event);

            let text = TuiText::styled(line, Style::default().fg(Color::Green));
            v.push(text);
        }

        // Some bookkeeping:
        if let ActualEvent(Resize(x, y)) = event {
            *last_window_size = Some((x, y));
        }

        // Finally, the actual event handling.
        match event {
            Error(err) => {
                // TODO: should we crash here?
                log::error!("Got a crossterm error: {:?}.", err)
            },

            // Currently, we only redraw on ticks (TODO: is this okay or should we
            // redraw on events too?):
            Tick => {
                drop(root.update(WidgetEvent::Update, &mut self.data, term));

                term.draw(|mut f| {
                    let area = f.size();
                    if (last_window_size == &Some((area.width, area.height)) ||
                                last_window_size == &None)
                            && area.area() > 0 {
                        Widget::render(root, &self.data, &mut f, area)
                    }

                    Widget::render(root, &self.data, &mut f, area)
                }).unwrap() // TODO: is unwrapping okay here?
            }

            ActualEvent(e) => match e {
                // Capture `ctrl + q`/`alt + f4` and forward everything else:
                Key(KeyEvent { code: KeyCode::Char('w'), modifiers: KeyModifiers::CONTROL }) |
                Key(KeyEvent { code: KeyCode::F(4), modifiers: KeyModifiers::ALT }) => {
                    return false
                }
                e => drop(root.update(e.into(), &mut self.data, term)),
            }

            _ => unreachable!("Got {:?} which shouldn't be possible.", event),
        }

        // onwards! (i.e. don't stop)
        true
    }
}

specialize! {
    desktop => {}
    web => {
        use tui::backend::CrosstermBackend;
        use xterm_js_sys::Terminal as XtermJsTerminal;
    }
}

impl<'a, T: TuiTypes> Tui<'a, T> { specialize! {
    desktop => {
        /// TODO: docs
        pub fn run_with_custom_layout<B: tui::backend::Backend + 'a>(
            mut self,
            term: &mut Terminal<B>,
            mut root: impl Widget<WidgetTypesSpec<T, B>>,
        ) -> Result<()>
        where
            B: ExecutableCommand<&'static str>,
            Terminal<B>: Send,
        {
            // init!
            self.init();

            // TODO: potentially construct this from user configurable options!
            let backoff = Backoff::default();

            // Spin up the event thing:
            let (event_recv, tx) = events::start_event_threads(term.backend_mut(), self.update_period)?;

            // Focus the root and never unfocus it!
            // (The root widget really should accept focus but we don't check that it does
            // here; if we're told to run with an empty widget tree we shall.)
            let _ = root.update(WidgetEvent::Focus(FocusEvent::GotFocus), &mut self.data, term);

            let mut last_window_size = None;

            // TODO: expose better errors from here! have `run_tick_with_event_with_project` return
            // a type that impls `Error`!
            backoff.run_tick_with_event_with_project(&mut self, |t| t.data.sim, event_recv, |tui, event| {
                tui.handle_event(event, term, &tx, &mut root, &mut last_window_size)
            }).map_err(|_| anyhow!("Channel disconnected; maybe something crashed?"))
        }

        // Run with default layout and a backend of your choosing.
        pub fn run<B: tui::backend::Backend + 'a>(self, term: &mut Terminal<B>) -> Result<()>
        where
            B: ExecutableCommand<&'static str>,
            Terminal<B>: Send,
        {
            self.run_with_custom_layout(term, crate::layout::layout(None, vec![]))
        }

        // Run with crossterm; with or without your own special layout.
        pub fn run_with_crossterm<'c: 'a>(
            self,
            root_widget: Option<
                impl Widget<
                    WidgetTypesSpec<T, tui::backend::CrosstermBackend<'c, Stdout>>
                >
            >
        ) -> Result<()> {
            use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
            use crossterm::execute;

            let mut stdout = std::io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            crossterm::terminal::enable_raw_mode()?;

            let backend = tui::backend::CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            terminal.hide_cursor()?;

            let panic_res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                if let Some(w) = root_widget {
                    self.run_with_custom_layout(&mut terminal, w)
                } else {
                    self.run(&mut terminal)
                }
            }));

            // TODO: v should maybe happen when the Crossterm Event Thread exits, but this
            // is okay for now.
            //
            // Right now we duplicate this logic in the crossterm event thread's exit since
            // it can also trigger a quit and when we quit from here, there's no guarantee
            // that the crossterm event thread doesn't die before it gets to reset the
            // screen.
            execute!(std::io::stdout(), DisableMouseCapture)?;

            terminal.show_cursor()?;
            crossterm::terminal::disable_raw_mode()?;
            execute!(std::io::stdout(), LeaveAlternateScreen)?;

            // We handle `panic_res` _after_ resetting the terminal so that we
            // don't leave the user's terminal in a bad state (i.e. we always run
            // the reset sequence above, even when panicking).
            match panic_res {
                Ok(res) => res,
                Err(panic_payload) => {
                    // TODO: stacktrace; message to open a github issue
                    //
                    // like: https://www.christopherbiscardi.com/custom-github-issue-templated-error-messages-for-panic-with-eyre-and-rust
                    // TODO: set backtrace style, etc.
                    panic::resume_unwind(panic_payload)
                }
            }
        }
    }

    web => {
        // Note: this is almost exactly a copy of the `desktop` counterpart.
        pub async fn run_with_custom_layout<'t: 'a, W: Write + 't>(
            mut self,
            term: &mut Terminal<CrosstermBackend<'t, W>>,
            mut root: impl Widget<WidgetTypesSpec<T, CrosstermBackend<'t, W>>>,
        ) -> Result<()> {
            // init!
            self.init();

            // TODO: potentially construct this from user configurable options!
            let backoff = Backoff::default();

            // Event things:
            let (event_recv, tx) = events::start_event_stream(
                term.backend_mut(),
                self.update_period,
            )?;

            let mut last_window_size = None;

            backoff.run_tick_with_event_with_project(
                &mut self,
                |t| t.data.sim,
                event_recv,
                |tui, event| tui.handle_event(event, term, &tx, &mut root, &mut last_window_size),
            )
            .await
            .map_err(|_| err_msg("Channel disconnected; maybe something crashed?"))
        }

        // Run with default layout and a backend of your choosing.
        pub async fn run<'t: 'a, W: Write + 't>(self, term: &mut Terminal<CrosstermBackend<'t, W>>) -> Result<()> {
            self.run_with_custom_layout(term, crate::layout::layout(None, vec![])).await
        }

        // Run with crossterm; with or without your own special layout.
        pub async fn run_with_xtermjs<'c: 'a>(
            self,
            root_widget: Option<impl Widget<WidgetTypesSpec<T, tui::backend::CrosstermBackend<'c, Vec<u8>>>>>,
            term: &'c XtermJsTerminal,
        ) -> Result<()> {
            use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
            use crossterm::execute;
            use tui::backend::CrosstermBackend;

            term.focus();

            let mut backend = CrosstermBackend::new(term);
            execute!(&mut backend, EnterAlternateScreen)?;

            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;

            let res = if let Some(w) = root_widget {
                self.run_with_custom_layout(&mut terminal, w).await
            } else {
                self.run(&mut terminal).await
            };

            terminal.show_cursor()?;

            let backend = terminal.backend_mut();
            execute!(backend, DisableMouseCapture, LeaveAlternateScreen)?;

            res
        }
    }
}}
