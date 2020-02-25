//! TODO!

use super::Res as Result;

use crossterm::event::{EnableMouseCapture, Event as CrosstermEvent};
use crossterm::ExecutableCommand;
use crossterm::ErrorKind as CrosstermError;

use std::sync::mpsc::{self, Receiver};
use std::thread::Builder as ThreadBuilder;
use std::time::Duration;
use std::sync::mpsc::Sender;

// pub(crate) type Event = CrosstermEvent;

pub(in crate::tui) struct Event {
    Error(CrosstermError),
    Tick,
    ActualEvent(CrosstermEvent),
}

// pub(crate) enum WidgetEvent {

// }

pub(crate) type WidgetEvent = crossterm::event::Event;

pub(crate) fn start_event_threads<T>(term: &mut T, tick: Duration) -> Result<Receiver<Event>>
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
    term.execute(EnableMouseCpture)?;

    // We could use the async version here (`crossterm::event::EventStream`) but
    // doing so doesn't get us anything other than additional dependencies (it'd
    // make sense if we were using async functions in other places in the
    // application but we're not and most of our operations are synchronous
    // anyways).

    ThreadBuilder::new()
        .name("TUI: Crossterm Event Thread")
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

    // let (tx, rx) = mpsc::channel();


    // let tx = tx.clone();
    // thread::spawn(move || {
    //     let input = input();
    //     let reader = input.read_sync();
    //     for event in reader {
    //         match event {
    //             Event::Key(key) => {
    //                 if let Err(_) = tx.send(Event::Input(key.clone())) {
    //                     return;
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    // });


    // todo!()
}


pub(crate) fn start_tick_thread(period: Duration,, tx: Sender<Event>) -> Result<()> {
    ThreadBuilder::new()
        .name("TUI: Tick Thread")
        .spawn(move || loop {
            // Same deal here as above; terminate if the channel fails.
            tx.send(Event::Tick).unwrap();
            std::thread::sleep(period);
        })?;

    Ok(())

    // let tx = tx.clone();
    // thread::spawn(move || {
    //     let tx = tx.clone();
    //     loop {
    //         tx.send(Event::Tick).unwrap();
    //         thread::sleep(Duration::from_millis(cli.tick_rate));
    //     }
    // });
}
