//! TODO!

use lc3_traits::control::Control;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use super::Res as Result;
use super::Tui;
use super::events;

// use crossterm::Crossterm;
use crossterm::{ExecutableCommand, execute};
use tui::terminal::Terminal;
use tui::backend::{Backend, CrosstermBackend};

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
    // TODO: not sure if this is worth doing
    pub fn run<B: Backend>(self, term: &mut Terminal<B>) -> Result<()>
    where
        B: ExecutableCommand<&'static str>
    {
        let event_recv = events::start_event_threads(term)?;

        todo!()
    }

    // pub fn run_with_crossterm(self, term: &mut Terminal<CrosstermBackend>) -> Result<()> {
    //     // start_crossterm_event_thread(terminal.input());
    //     let event_recv = start_crossterm_event_thread();
    //     // periodic_event(input());

    //     unimplemented!()
    // }

    pub fn run_with_crossterm/*_default*/(self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;

        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.hide_cursor()?;

        // self.run_with_crossterm(&mut terminal)
        self.run(&mut terminal)
    }
}
