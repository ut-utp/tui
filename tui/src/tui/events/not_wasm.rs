//! Event listener implementation that's thread based (for targets that aren't
//! WebAssembly).

use super::super::Res as Result;
use super::{Event, CrosstermEvent, KeyEvent};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::cursor::Show;
use crossterm::{ExecutableCommand, execute};
use crossterm::ErrorKind as CrosstermError;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::Builder as ThreadBuilder;
use std::time::Duration;
use std::io::Write;

pub(in crate::tui) fn start_event_threads<T>(
    term: &mut T,
    tick: Duration
) -> Result<(Receiver<Event>, Sender<Event>)>
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
fn start_crossterm_event_thread<T>(
    term: &mut T,
    tx: Sender<Event>
) -> Result<()>
where
    T: ExecutableCommand<&'static str>,
{
    let _ = term.execute(EnableMouseCapture)?;

    // We could use the async version here (`crossterm::event::EventStream`) but
    // doing so doesn't get us anything other than additional dependencies (it'd
    // make sense if we were using async functions in other places in the
    // application but we're not and most of our operations are synchronous
    // anyways).

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
                    use crossterm::event::{KeyCode, KeyModifiers as Km};

                    // Ideally this would be Ctrl + Shift + q but crossterm
                    // doesn't seem to do a good job actually relaying Ctrl +
                    // Shift key events. So, we'll do Alt + q.
                    if let CrosstermEvent::Key(KeyEvent {
                        code: KeyCode::Char('q'), modifiers: Km::CONTROL
                    }) = e {
                        exit().unwrap();

                        eprintln!("Force Quit!");
                        std::process::exit(1);
                    }

                    tx.send(Event::ActualEvent(e))
                },
                Err(err) => tx.send(Event::Error(err)),
            } {
                exit().unwrap();
                let _ = crate::debug::run_if_debugging(||
                    eprintln!("Event thread exiting!"));
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
                crate::debug::run_if_debugging(||
                    eprintln!("Tick thread exiting!"));
                break
            }
            std::thread::sleep(period);
        })?;

    Ok(())
}
