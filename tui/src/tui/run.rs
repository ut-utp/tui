//! TODO!

use super::Res as Result;
use super::Tui;
use super::events::{self, Event, WidgetEvent, FocusEvent, CrosstermEvent};
use super::widget::Widget;
use crate::strings::{s, HelloMsg, StartupMsg};

use lc3_traits::control::Control;
use lc3_application_support::event_loop::Backoff;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use chrono::{DateTime, offset::Local};
use crossterm::ExecutableCommand;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, DisableMouseCapture};
use failure::err_msg;
use tui::terminal::Terminal;
use tui::backend::Backend;
use tui::widgets::Text as TuiText;
use tui::layout::Rect;
use tui::style::{Color, Style};

use std::io::{Stdout, Write};

specialize! {
    desktop => { use std::sync::mpsc::Sender; }
    web => { use futures_channel::mpsc::UnboundedSender; }
}

impl<'a, 'int, C, I, O> Tui<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
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
        root: &mut impl Widget<'a, 'int, C, I, O, B>,
        last_window_size: &mut Option<(u16, u16)>,
    ) -> bool
    where
        B: Backend,
        B: ExecutableCommand<&'static str>,
        Terminal<B>: Send,
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

                let text = TuiText::styled(line, Style::default().fg(Color::Green));
                v.push(text);
            }

            if let ActualEvent(Resize(x, y)) = event {
                last_window_size = Some((x, y));
            }

            match event {
                Error(err) => {
                    // TODO: should we crash here?
                    log::error!("Got a crossterm error: {:?}.", err)
                },

                // Currently, we only redraw on ticks (TODO: is this okay or should we
                // redraw on events too?):
                Tick => {
                    drop(root.update(WidgetEvent::Update, &mut tui.data, term));

                    term.draw(|mut f| {
                        let area = f.size();
                        if (last_window_size == Some((area.width, area.height)) ||
                                    last_window_size == None)
                                && area.area() > 0 {
                            Widget::render(&mut root, &tui.data, &mut f, area)
                        }

                        Widget::render(&mut root, &tui.data, &mut f, area)
                    }).unwrap() // TODO: is unwrapping okay here?
                }

                ActualEvent(e) => match e {
                    // Capture `ctrl + q`/`alt + f4` and forward everything else:
                    Key(KeyEvent { code: KeyCode::Char('w'), modifiers: KeyModifiers::CONTROL }) |
                    Key(KeyEvent { code: KeyCode::F(4), modifiers: KeyModifiers::ALT }) => {
                        return false
                    }
                    e => drop(root.update(e.into(), &mut tui.data, term)),
                }

                _ => unreachable!("Got {:?} which shouldn't be possible.", event),
            }

            // onwards! (i.e. don't stop)
            true
        }).map_err(|_| err_msg("Channel disconnected; maybe something crashed?"))
    }

    // Run with default layout and a backend of your choosing.
    pub fn run<B: Backend>(self, term: &mut Terminal<B>) -> Result<()>
    where
        B: ExecutableCommand<&'static str>,
        Terminal<B>: Send,
    {
        self.run_with_custom_layout(term, crate::layout::layout(None, vec![]))
    }

    web => {
        // Note: this is almost exactly a copy of the `desktop` counterpart.
        pub async fn run_with_custom_layout<B: Backend>(
            mut self,
            term: &mut Terminal<B>,
            mut root: impl Widget<'a, 'int, C, I, O, B>,
        ) -> Result<()>
        where
            B: ExecutableCommand<&'static str>,
            Terminal<B>: Send,
        {
            // init!
            self.init();

            // TODO: potentially construct this from user configurable options!
            let backoff = Backoff::default();

            // Event things:
            let (event_recv, tx) = events::start_event_stream(term.backend_mut(), self.update_period)?;

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
        pub async fn run<B: Backend>(self, term: &mut Terminal<B>) -> Result<()>
        where
            B: ExecutableCommand<&'static str>,
            Terminal<B>: Send,
        {
            self.run_with_custom_layout(term, crate::layout::layout(None, vec![])).await
        }

        // Run with crossterm; with or without your own special layout.
        pub async fn run_with_xtermjs<'c>(
            self,
            root_widget: Option<impl Widget<'a, 'int, C, I, O, tui::backend::CrosstermBackend<'c, Vec<u8>>>>,
            terminal: &'c xterm_js_sys::Terminal,
        ) -> Result<()> {
            use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
            use crossterm::execute;
            use tui::backend::CrosstermBackend;

            term.focus();

            let backend = CrosstermBackend::new(term);
            let mut terminal = Terminal::new(backend)?;

            execute!(terminal, EnterAlternateScreen)?;
            terminal.hide_cursor()?;

            let res = if let Some(w) = root_widget {
                self.run_with_custom_layout(&mut terminal, w).await
            } else {
                self.run(&mut terminal).await
            };

            execute!(terminal, DisableMouseCapture)?;
            terminal.show_cursor()?;
            execute!(terminal, LeaveAlternateScreen)?;

            res
        }
    }
}}
