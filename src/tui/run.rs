//! TODO!

use super::Res as Result;
use super::Tui;
use super::events::{self, Event, WidgetEvent, FocusEvent, CrosstermEvent};
use super::widget::Widget;

use lc3_traits::control::Control;
use lc3_application_support::event_loop::Backoff;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use chrono::{DateTime, offset::Local};
use crossterm::{ExecutableCommand, execute};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, DisableMouseCapture};
use failure::err_msg;
use tui::terminal::Terminal;
use tui::backend::{Backend, CrosstermBackend};

use std::io::{Stdout, Write};

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
    pub fn run_with_custom_layout<B: Backend>(mut self, term: &mut Terminal<B>, mut root: impl Widget<'a, 'int, C, I, O, B>) -> Result<()>
    where
        B: ExecutableCommand<&'static str>
    {
        let event_recv = events::start_event_threads(term.backend_mut(), self.update_period)?;

        // TODO: potentially construct this from user configurable options!
        let backoff = Backoff::default();

        // Focus the root and never unfocus it!
        // (The root widget really should accept focus but we don't check that it does
        // here; if we're told to run with an empty widget tree we shall.)
        let _ = root.update(WidgetEvent::Focus(FocusEvent::GotFocus), &mut self.data);

        backoff.run_tick_with_event_with_project(&mut self, |t| t.data.sim, event_recv, |tui, event| {
            use Event::*;
            use CrosstermEvent::*;

            log::trace!("Event: {:?}", event);

            if let Some(ref mut s) = tui.data.log {
                let time = std::time::SystemTime::now();
                let time: DateTime<Local> = time.into();
                s.push_str(format!("[EVENT] @ {}: {:?}\n", time.format("%d/%m/%Y %T%.6f"), event).as_ref())
            }

            match event {
                Error(err) => {
                    // TODO: should we crash here?
                    log::error!("Got a crossterm error: {:?}.", err)
                },

                // Currently, we only redraw on ticks (TODO: is this okay or should we
                // redraw on events too?):
                Tick => term.draw(|mut f| {
                    let area = f.size();
                    Widget::render(&mut root, &tui.data, &mut f, area)
                }).unwrap(), // TODO: is unwrapping okay here?

                ActualEvent(e) => match e {
                    // Capture `ctrl + q`/`alt + f4` and forward everything else:
                    Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL }) |
                    Key(KeyEvent { code: KeyCode::Char('w'), modifiers: KeyModifiers::CONTROL }) |
                    Key(KeyEvent { code: KeyCode::F(4), modifiers: KeyModifiers::ALT }) => {
                        return false
                    }
                    e => drop(root.update(e.into(), &mut tui.data)),
                }
            }

            // onwards! (i.e. don't stop)
            true
        }).map_err(|_| err_msg("Channel disconnected; maybe something crashed?"))
    }

    // Run with default layout and a backend of your choosing.
    pub fn run<B: Backend>(self, term: &mut Terminal<B>) -> Result<()>
    where
        B: ExecutableCommand<&'static str>
    {
        self.run_with_custom_layout(term, crate::layout::layout())
    }

    // Run with crossterm; with or without your own special layout.
    pub fn run_with_crossterm(self, root_widget: Option<impl Widget<'a, 'int, C, I, O, CrosstermBackend<Stdout>>>) -> Result<()> {
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.hide_cursor()?;

        let res = if let Some(w) = root_widget {
            self.run_with_custom_layout(&mut terminal, w)
        } else {
            self.run(&mut terminal)
        };

        // TODO: v should maybe happen when the Crossterm Event Thread exits, but this
        // is okay for now.
        execute!(std::io::stdout(), DisableMouseCapture)?;

        terminal.show_cursor()?;
        crossterm::terminal::disable_raw_mode()?;
        execute!(std::io::stdout(), LeaveAlternateScreen)?;

        res
    }
}
