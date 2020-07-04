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

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
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
            .for_each(|(idx, (addr, _))| self.data.wp.insert(addr, idx));

        self.data.sim.get_breakpoints()
            .iter().copied()
            .filter_map(std::convert::identity)
            .enumerate()
            .for_each(|(idx, addr)| self.data.bp.insert(addr, idx));
    }

    // Matches the interface `Backoff` has for the function it takes; return value
    // indicates whether to continue.
    fn handle_event<B>(&mut self, event: Event, term: &mut Terminal<B>) -> bool
    where
        B: ExecutableCommand<&'static str>,
        Terminal<B>: Send,
    {
        let (event_recv, tx) = events::start_event_threads(term.backend_mut(), self.update_period)?;

        // TODO: potentially construct this from user configurable options!
        let backoff = Backoff::default();

        // Focus the root and never unfocus it!
        // (The root widget really should accept focus but we don't check that it does
        // here; if we're told to run with an empty widget tree we shall.)
        let _ = root.update(WidgetEvent::Focus(FocusEvent::GotFocus), &mut self.data, term);

        self.data.log(s!(HelloMsg), Color::Cyan);
        self.data.log(s!(StartupMsg), Color::Magenta);

        let wps = self.data.sim.get_memory_watchpoints();
        let mut i = 0;
        while let Some(wp) = wps[i] {
            self.data.wp.insert(wp.0, i);
            i = i + 1;
        }

        let bps = self.data.sim.get_breakpoints();
        i = 0;
        while let Some(bp) = bps[i] {
            self.data.bp.insert(bp, i);
            i = i + 1;
        }

        let mut last_window_size = None;

        backoff.run_tick_with_event_with_project(&mut self, |t| t.data.sim, event_recv, |tui, event| {
            use Event::*;
            use CrosstermEvent::*;

            // Empty the queue if we're told to. This should handle nested requests
            // (i.e. if we're told to empty the queue while we're already doing so).
            if let Some(f) = tui.data.flush_all_events {
                use super::Flush::*;

                match f {
                    Requested(i) => {
                        tx.send(FlushEventsBarrier(i)).unwrap();
                        tui.data.flush_all_events = Some(Acknowledged(i));
                    }
                    Acknowledged(c) => {
                        match event {
                            FlushEventsBarrier(i) => {
                                // If its count matches us, we're done clearing.
                                if c == i {
                                    drop(tui.data.flush_all_events.take());
                                    let Rect { width, height, .. } = term.size().unwrap();
                                    tx.send(ActualEvent(Resize(width, height))).unwrap();
                                }
                            }
                            _ => {
                                /* Otherwise, continue discarding events. */
                                if let Some(ref mut v) = tui.data.debug_log {
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

            log::trace!("Event: {:?}", event);

            if let Some(ref mut v) = tui.data.debug_log {
                let time = std::time::SystemTime::now();
                let time: DateTime<Local> = time.into();
                let line = format!("[EVENT] @ {}: {:?}\n", time.format("%d/%m/%Y %T%.6f"), event);

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

    // Run with crossterm; with or without your own special layout.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_crossterm<'c>(self, root_widget: Option<impl Widget<'a, 'int, C, I, O, tui::backend::CrosstermBackend<'c, Stdout>>>) -> Result<()> {
        use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
        use crossterm::execute;

        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        let backend = tui::backend::CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.hide_cursor()?;

        let res = if let Some(w) = root_widget {
            self.run_with_custom_layout(&mut terminal, w)
        } else {
            self.run(&mut terminal)
        };

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

        res
    }
}
