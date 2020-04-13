//! TODO!

use super::widget_impl_support::*;
use super::load_button::*;

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
pub struct Modeline<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    event_fut: Option<C::EventFuture>,
    colour: Color,
    button1: Rect,
    button2: Rect,
    button3: Rect,
    loadB: Vec<Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>>,
    focus: u8,
}

impl<'a, 'int, C, I, O, B> Modeline<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub fn new<W: Widget<'a, 'int, C, I, O, B> + 'a>(button: W) -> Self {
        Self::new_with_colour(button, Color::Blue)
    }

    pub fn new_with_colour<W: Widget<'a, 'int, C, I, O, B> + 'a>(button: W, colour:Color) -> Self {
        Self {
            event_fut: None,
            colour,
            button1: Rect::default(),
            button2: Rect::default(),
            button3: Rect::default(),
            loadB: vec![Box::new(button)],
            focus: 0,
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

impl<'a, 'int, C, I, O, B> TuiWidget for Modeline<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Modeline<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let mut bColour = self.colour;
        let mut mColour = self.colour;
        let mut b1Colour = Colour::Green;
        let mut b2Colour = Colour::Magenta;
        let mut b3Colour = Colour::White;

        if self.focus > 0 {
            bColour = Colour::Red
        }

        if data.sim.get_state() == State::RunningUntilEvent {
            b1Colour = Colour::Yellow;
        }

        match self.focus {
            1 => mColour = Colour::Red,
            2 => b1Colour = Colour::Red,
            3 => b2Colour = Colour::Red,
            4 => b3Colour = Colour::Red,
            _ => {},
        }

        let mut bg_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(bColour))
            .title("");

        bg_block.draw(area, buf);

        let area = bg_block.inner(area);
        let area = Rect::new(area.x, area.y, area.width/2, area.height);

        let mut bg_block = Block::default()
            .style(Style::default().bg(mColour))
            .title("");

        bg_block.draw(area, buf);

        self.button1 = Rect::new(area.width + area.width/12, area.y, area.width*2/12, area.height);
        self.button2 = Rect::new(area.width + area.width/12*4, area.y, area.width*2/12, area.height);
        self.button3 = Rect::new(area.width + area.width/12*7, area.y, area.width*5/12, area.height);

        if data.sim.get_state() == State::RunningUntilEvent{
            let text = [TuiText::styled("Pause", Style::default().fg(Colour::Yellow))];
            let mut para = Paragraph::new(text.iter())
                .style(Style::default())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(b1Colour))
                    .title(""))
                .alignment(Alignment::Center)
                .wrap(true);
            para.draw(self.button1,buf);
        } else {
            let text = [TuiText::styled("Run", Style::default().fg(Colour::Green))];
            let mut para = Paragraph::new(text.iter())
                .style(Style::default())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(b1Colour))
                    .title(""))
                .alignment(Alignment::Center)
                .wrap(true);
            para.draw(self.button1,buf);
        }
        
            


        let text = [TuiText::styled("Step", Style::default().fg(Colour::Magenta))];
        let mut para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(b2Colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
            

        para.draw(self.button2,buf);

        bg_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(b3Colour))
            .title("");

        bg_block.draw(self.button3, buf);

        Widget::draw(&mut *self.loadB[0], data, bg_block.inner(self.button3), buf);

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

        if self.focus == 0 {
            self.focus = 1;
        }

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => {self.focus = 0; false},
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, x, y, _)) => {
                if self.button1.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 2;
                    if data.sim.get_state() == State::RunningUntilEvent{
                        self.pause(data)
                    } else {
                        self.run(data);
                    }
                } else if self.button2.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 3;
                    self.step(data);
                } else if self.button3.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 4;
                    self.loadB[0].update(event, data, terminal);
                }
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
                KeyEvent { code: KeyCode::Enter, modifiers: EMPTY } => {
                    match self.focus {
                        2 => {
                            if data.sim.get_state() == State::RunningUntilEvent{
                                self.pause(data)
                            } else {
                                self.run(data);
                            }
                        },
                        3 => self.step(data),
                        4 => {self.loadB[0].update(event, data, terminal);},
                        _ => {},
                    }
                    true
                }
                KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::CONTROL } => {
                    if self.focus < 4 {
                        self.focus += 1;;
                    }
                    true
                }
                KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::CONTROL } => {
                    if self.focus > 1 {
                        self.focus -= 1;
                    }
                    true
                }
                _ => false,
            }
            _ => false,
        }
    }
}
