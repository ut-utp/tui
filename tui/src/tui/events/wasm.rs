//! Event listener implementation that's stream based (for WebAssembly targets).

use super::super::Res as Result;
use super::Event;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::event::EventStream;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::cursor::Show;
use crossterm::{ExecutableCommand, execute};
use crossterm::ErrorKind as CrosstermError;
use futures_core::stream::Stream;
use futures_channel::mpsc::{self, UnboundedSender as Sender};
use futures_util::stream::{TryStreamExt, StreamExt, select};
use tui::backend::CrosstermBackend;

use std::time::Duration;
use std::io::Write;

trait Unify<T> { fn unify(self) -> T; }

impl<T> Unify<T> for std::result::Result<T, T> {
    fn unify(self) -> T {
        match self {
            Ok(val) | Err(val) => val
        }
    }
}

// Note that this still takes a `tick` Duration, but it's a little bit weird
// since RAF (request animation frame) is used internally.

pub(in crate::tui) fn start_event_stream<'c, T: 'c>(
    term: &mut CrosstermBackend<'c, T>,
    tick: Duration,
) -> Result<(impl Stream<Item = Event> + 'c, Sender<Event>)>
where
    T: Write,
{
    let (tx, rx) = mpsc::unbounded();

    let stream = start_crossterm_event_stream(term)?;
    start_tick_stream(tick, tx.clone())?;

    Ok((select(stream, rx), tx))
}

// Some of this is copied from the `not_wasm` impl (in `not_wasm.rs`).
//
// Note that we don't have any notion of "exiting" on wasm which is why this
// is missing lots of stuff like checking for Alt + q.
fn start_crossterm_event_stream<'c, T>(
    term: &mut CrosstermBackend<'c, T>,
) -> Result<impl Stream<Item = Event> + 'c>
where
    T: Write,
{
    let _ = term.execute(EnableMouseCapture)?;

    Ok(EventStream::new(term.buffer.terminal)
        .map_ok(Event::ActualEvent)
        .map_err(Event::Error)
        .map(Unify::unify))
}


fn start_tick_stream(period: Duration, tx: Sender<Event>) -> Result<()> {
    let mut last = 0.0f64;
    let interval = period.as_secs_f64() * 1_000f64;

    rafi::AnimationFrameCallbackWrapper::new()
        .leak()
        .safe_start(move |timestamp| {
            if timestamp - last > interval {
                last = timestamp;

                if let Err(_) = tx.unbounded_send(Event::Tick) {
                    // Stop if we failed to send the tick.
                    return false;
                }
            }

            true
        });

    Ok(())
}


// TODO: spin this off into its own crate:
mod rafi {
    use js_sys::Function;
    use wasm_bindgen::{prelude::*, JsCast};
    use web_sys::Window;

    use std::cell::Cell;

    #[wasm_bindgen]
    #[derive(Default)]
    pub struct AnimationFrameCallbackWrapper {
        // These are both boxed because we want stable addresses!
        handle: Box<Cell<Option<i32>>>,
        func: Option<Box<dyn FnMut(f64) -> bool + 'static>>,
    }

    #[allow(clippy::option_map_unit_fn)]
    impl Drop for AnimationFrameCallbackWrapper {
        fn drop(&mut self) {
            self.handle.get().map(cancel_animation_frame);
        }
    }

    fn cancel_animation_frame(handle: i32) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(handle)
            .unwrap()
    }

    impl AnimationFrameCallbackWrapper /*<'a>*/ {
        pub fn new() -> Self {
            Self {
                handle: Box::new(Cell::new(None)),
                func: None,
            }
        }

        pub fn leak(self) -> &'static mut Self {
            Box::leak(Box::new(self))
        }

        /// To use this, you'll probably have to leak the wrapper.
        ///
        /// `Self::leak` can help you with this.
        pub fn safe_start(&'static mut self, func: impl FnMut(f64) -> bool + 'static) {
            unsafe { self.inner(func) }
        }

        /// This is extremely prone to crashing and is probably unsound; use at your
        /// own peril.
        #[inline(never)]
        pub unsafe fn start<'s, 'f: 's>(
            &'s mut self,
            func: impl FnMut(f64) -> bool + 'f,
        ) {
            self.inner(func)
        }

        #[allow(unused_unsafe, clippy::borrowed_box)]
        unsafe fn inner<'s, 'f: 's>(&'s mut self, func: impl FnMut(f64) -> bool + 'f) {
            if let Some(handle) = self.handle.get() {
                cancel_animation_frame(handle)
            }

            let func: Box<dyn FnMut(f64) -> bool + 'f> = Box::new(func);
            // Crime!
            let func: Box<dyn FnMut(f64) -> bool + 'static> =
                unsafe { core::mem::transmute(func) };
            self.func = Some(func);

            // This is the dangerous part; we're saying this is okay because we
            // cancel the RAF on Drop of this structure so, in theory, when the
            // function goes out of scope, the RAF will also be cancelled and the
            // invalid reference won't be used.
            let wrapper: &'static mut Self = unsafe { core::mem::transmute(self) };

            let window = web_sys::window().unwrap();

            fn recurse(
                f: &'static mut Box<dyn FnMut(f64) -> bool + 'static>,
                h: &'static Cell<Option<i32>>,
                window: Window,
            ) -> Function {
                let val = Closure::once_into_js(move |timestamp| {
                    // See: https://github.com/rust-lang/rust/issues/42574
                    let f = f;

                    if h.get().is_none() {
                        return;
                    }

                    if f(timestamp) {
                        let next = recurse(f, h, window.clone());
                        let id = window.request_animation_frame(&next).unwrap();
                        h.set(Some(id));
                    } else {
                        // No need to drop the function here, really.
                        // It'll get dropped whenever the wrapper gets dropped.
                        // drop(w.func.take());

                        // We should remove the handle though, so that when the
                        // wrapper gets dropped it doesn't try to cancel something
                        // that already ran.
                        let _ = h.take();
                    }
                });

                val.dyn_into().unwrap()
            }

            let func: &'static mut Box<dyn FnMut(f64) -> bool + 'static> =
                wrapper.func.as_mut().unwrap();
            let starting = recurse(func, &wrapper.handle, window.clone());
            wrapper
                .handle
                .set(Some(window.request_animation_frame(&starting).unwrap()));
        }
    }
}
