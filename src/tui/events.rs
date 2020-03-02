//! TODO!

use super::Res as Result;

use crossterm::event::EnableMouseCapture;
use crossterm::ExecutableCommand;
use crossterm::ErrorKind as CrosstermError;
pub use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};

use std::sync::mpsc::{self, Receiver};
use std::thread::Builder as ThreadBuilder;
use std::time::Duration;
use std::sync::mpsc::Sender;


/// All events that our event threads produce.
#[derive(Debug)]
pub(in crate::tui) enum Event {
    Error(CrosstermError),
    Tick,
    ActualEvent(CrosstermEvent),
}

/// The only events that actually make their to Widgets.
///
/// All other events (i.e. the others in the [`Event`] enum) are handled
/// "internally".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum WidgetEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Focus(FocusEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FocusEvent {
    GotFocus,
    LostFocus
}

impl From<CrosstermEvent> for WidgetEvent {
    fn from(event: CrosstermEvent) -> Self {
        use CrosstermEvent::*;
        match event {
            Key(k) => WidgetEvent::Key(k),
            Mouse(m) => WidgetEvent::Mouse(m),
            Resize(x, y) => WidgetEvent::Resize(x, y),
        }
    }
}

pub(in crate::tui) fn start_event_threads<T>(term: &mut T, tick: Duration) -> Result<Receiver<Event>>
where
    T: ExecutableCommand<&'static str>
{
    let (tx, rx) = mpsc::channel();

    start_crossterm_event_thread(term, tx.clone())?;
    start_tick_thread(tick, tx)?;

    Ok(rx)
}

// You don't have to be using a crossterm backend with `tui` to use this. I
// think. (TODO)
fn start_crossterm_event_thread<T>(term: &mut T, tx: Sender<Event>) -> Result<()>
where
    T: ExecutableCommand<&'static str>
{
    term.execute(EnableMouseCapture)?;

    // We could use the async version here (`crossterm::event::EventStream`) but
    // doing so doesn't get us anything other than additional dependencies (it'd
    // make sense if we were using async functions in other places in the
    // application but we're not and most of our operations are synchronous
    // anyways).

    ThreadBuilder::new()
        .name("TUI: Crossterm Event Thread".to_string())
        .spawn(move || loop {
            // Note that if we get an error here, we do not crash or end the
            // thread.
            //
            // If the receiver wishes to 'handle' the error by crashing the
            // application, they are free to do so; we trust that the OS will
            // stop this thread once the main thread kills the program.
            //
            // We do, however, terminate if the mpsc channel returns an error
            // (we assume that if this happens it means that the recipient
            // terminated).
            match crossterm::event::read() {
                Ok(e) => tx.send(Event::ActualEvent(e)),
                Err(err) => tx.send(Event::Error(err)),
            }.unwrap()
        })?;

    Ok(())
}


fn start_tick_thread(period: Duration, tx: Sender<Event>) -> Result<()> {
    ThreadBuilder::new()
        .name("TUI: Tick Thread".to_string())
        .spawn(move || loop {
            // Same deal here as above; terminate if the channel fails.
            tx.send(Event::Tick).unwrap();
            std::thread::sleep(period);
        })?;

    Ok(())
}
