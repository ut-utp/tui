//! TODO!

use super::Res as Result;
use super::Tui;
use super::events::{self, Event, WidgetEvent, FocusEvent, CrosstermEvent};
use super::widget::{Widget, Widgets};

use lc3_traits::control::Control;
use lc3_application_support::event_loop::Backoff;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use crossterm::{ExecutableCommand, execute};
use crossterm::terminal::EnterAlternateScreen;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use failure::err_msg;
use tui::terminal::Terminal;
use tui::backend::{Backend, CrosstermBackend};

use std::io::{Stdout, Write};

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
    pub fn run_with_custom_layout<B: Backend>(mut self, term: &mut Terminal<B>, mut root: Widgets<'a, 'int, C, I, O, B>) -> Result<()>
    where
        B: ExecutableCommand<&'static str>
    {
        let event_recv = events::start_event_threads(term.backend_mut(), self.update_period)?;

        todo!()
    }

    // Run with crossterm; with or without your own special layout.
    pub fn run_with_crossterm(self, root_widget: Option<Widgets<'a, 'int, C, I, O, CrosstermBackend<Stdout>>>) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;

        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.hide_cursor()?;

        if let Some(w) = root_widget {
            self.run_with_custom_layout(&mut terminal, w)
        } else {
            self.run(&mut terminal)
        }
    }
}
