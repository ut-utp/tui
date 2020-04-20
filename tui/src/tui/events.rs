//! TODO!

use super::Res as Result;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::cursor::Show;
use crossterm::{ExecutableCommand, execute};
use crossterm::ErrorKind as CrosstermError;
pub use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::Builder as ThreadBuilder;
use std::time::Duration;
use std::io::Write;

/// All events that our event threads produce.
#[derive(Debug)]
#[non_exhaustive]
pub(in crate::tui) enum Event {
    Error(CrosstermError),
    Tick,
    ActualEvent(CrosstermEvent),
    #[doc(hidden)]
    FlushEventsBarrier(u8),
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
    Update, // Just another name for Tick.
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

pub(in crate::tui) fn start_event_threads<T>(term: &mut T, tick: Duration) -> Result<(Receiver<Event>, Sender<Event>)>
where
    T: ExecutableCommand<&'static str>
{
    let (tx, rx) = mpsc::channel();

    start_crossterm_event_thread(term, tx.clone())?;
    start_tick_thread(tick, tx.clone())?;

    Ok((rx, tx))
}

// You don't have to be using a crossterm backend with `tui` to use this. I
// think. (TODO)
fn start_crossterm_event_thread<T>(term: &mut T, tx: Sender<Event>) -> Result<()>
where
    T: ExecutableCommand<&'static str>
{
    let _ = term.execute(EnableMouseCapture)?;

    // We could use the async version here (`crossterm::event::EventStream`) but
    // doing so doesn't get us anything other than additional dependencies (it'd
    // make sense if we were using async functions in other places in the
    // application but we're not and most of our operations are synchronous
    // anyways).

    // Ideally this would be a const, but BitOr isn't const and bitflags offers
    // us no way to generate the below in a const context. As such, we compute
    // it out here (outside of the loop) so there's no chance it gets computed
    // repeatedly.
    use crossterm::event::{KeyCode, KeyModifiers as Km};
    let ctrl_shift_q: KeyEvent = KeyEvent { code: KeyCode::Char('q'), modifiers: Km::ALT};

    fn exit() -> Result<()> {
        let mut out = std::io::stdout();

        // This is roughly copied from `tui/run.rs`
        execute!(out, DisableMouseCapture)?;
        execute!(out, Show)?; // Show cursor.

        crossterm::terminal::disable_raw_mode()?;
        execute!(out, LeaveAlternateScreen)?;

        Ok(())
    }

    let _ = ThreadBuilder::new()
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
            // terminated so we exit gracefully rather than panic).
            if let Err(_) = match crossterm::event::read() {
                Ok(e) => {
                    if let CrosstermEvent::Key(key) = e {
                        if key == ctrl_shift_q {
                            exit().unwrap();

                            eprintln!("Force Quit!");
                            std::process::exit(1);
                        }
                    }

                    tx.send(Event::ActualEvent(e))
                },
                Err(err) => tx.send(Event::Error(err)),
            } {
                exit().unwrap();
                let _ = crate::debug::run_if_debugging(|| eprintln!("Event thread exiting!"));
                break
            }
        })?;

    Ok(())
}


fn start_tick_thread(period: Duration, tx: Sender<Event>) -> Result<()> {
    let _ = ThreadBuilder::new()
        .name("TUI: Tick Thread".to_string())
        .spawn(move || loop {
            // Same deal here as above; exit if the channel fails.
            if let Err(_) = tx.send(Event::Tick) {
                crate::debug::run_if_debugging(|| eprintln!("Tick thread exiting!"));
                break
            }
            std::thread::sleep(period);
        })?;

    Ok(())
}
