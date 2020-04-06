//! TODO!

use super::widget_impl_support::*;

use tui::widgets::{Paragraph};
use tui::style::{Color, Style};

use core::marker::{PhantomData, Unpin};
use core::pin::Pin;
use core::future::Future;
use core::task::{Context, Waker, Poll};

use lc3_traits::control::{Event, State};

// A fake Future executor that is capable of doing exactly no work.
//
// The correct thing to do is to use futures_executor::block_on but we run on
// platforms without threads and that function uses TLS (Thread Local Storage),
// so: crimes.
//
// // TODO: We can use `pin_utils::pin_mut!` to drop the `Unpin` requirement.
fn block_on<F: Future/* + Unpin*/>(f: F) -> F::Output {
    // shhh!
    use lc3_traits::control::rpc::device::RW_CLONE;

    pin_utils::pin_mut!(f);

    #[allow(unsafe_code)] // See the note above..
    let poll = f.poll(&mut Context::from_waker(
        &unsafe { Waker::from_raw(RW_CLONE(&())) }
    ));

    if let Poll::Ready(res) = poll {
        res
    } else {
        panic!("We were given a Future that wasn't already ready!")
    }
}

#[allow(explicit_outlives_requirement)]
#[derive(Debug, Clone, PartialEq)]
pub struct Modeline<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    event_fut: Option<C::EventFuture>,
    colour: Color,
    _p: PhantomData<(&'int (), &'a I, &'a O, C)>,
    focus: bool,
}

impl<'a, 'int, C, I, O> Modeline<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub fn new() -> Self {
        Self::new_with_colour(Color::Blue)
    }

    pub fn new_with_colour(colour:Color) -> Self {
        Self {
            event_fut: None,
            colour,
            _p: PhantomData,
            focus: false,
        }
    }

    fn step(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.sim.step();
        drop(data.current_event.take())
    }

    fn pause(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.sim.pause();
    }

    // fn load
    // TODO: should also call `drop(data.current_event.take())`

    fn run(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        // Only call `run_until_event` if we're not already running until an event.
        if State::RunningUntilEvent != data.sim.get_state() {
            // Dispose of any currently running futures correctly.
            if let Some(e) = self.event_fut.take() {
                // If we're calling this (i.e. if we're not actively running
                // until an event) blocking on this should return _immediately_.
                block_on(e);
            }

            self.event_fut = Some(data.sim.run_until_event());
        } else {
            // Just to make sure!

            // eprintln!("Already running!");
            assert!(self.event_fut.is_some());
        }

        drop(data.current_event.take())
    }

    fn reset(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.sim.reset();

        // Resolve the pending future, if there is one.
        if let Some(e) = self.event_fut.take() {
            data.sim.step();
            block_on(e);
        }

        drop(data.current_event.take())
    }
}

impl<'a, 'int, C, I, O> TuiWidget for Modeline<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Modeline<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let mut bColour = self.colour;
        if self.focus {
            bColour = Colour::Red;
        }

        let mut bg_block = Block::default()
            .style(Style::default().bg(self.colour))
            .borders(Borders::TOP)
            .border_style(Style::default().fg(bColour))
            .title("");

        bg_block.draw(area, buf);

        let area = bg_block.inner(area);
        // TODO!

    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        // data.log(format!("Event in the modeline! {:?}", event), Color::Red);

        // If we're currently running an event and the state changes, we've got
        // ourselves an event.
        if let Some(_) = self.event_fut {
            if State::RunningUntilEvent != data.sim.get_state() {
                let event = block_on(self.event_fut.take().unwrap());

                data.log(format!("[mode] Got an event! {:?}", event), Color::Blue);

                assert!(data.current_event.is_none()); // We're being defensive; I thini this holds.
                data.current_event = Some(event);
            }
        }

        self.focus = true;

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => {self.focus = false; false},
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => {
                self.run(data);
                true
            }

            Key(e) => match e {
                KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL } => {
                    self.step(data);
                    true
                }
                KeyEvent { code: KeyCode::Char('p'), modifiers: KeyModifiers::CONTROL } => {
                    self.pause(data);
                    true
                }
                KeyEvent { code: KeyCode::Char('r'), modifiers: KeyModifiers::CONTROL } => {
                    self.run(data);
                    true
                }
                _ => false,
            }
            _ => false,
        }
    }
}
