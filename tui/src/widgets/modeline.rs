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

#[allow(explicit_outlives_requirements)]
pub struct Modeline<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    event_fut: Option<C::EventFuture>,
    colour: Color,
    execution_control_button: Rect,
    step_button: Rect,
    reset_button: Rect,
    load_button: Rect,
    reset_flag: bool,
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
            execution_control_button: Rect::default(),
            step_button: Rect::default(),
            reset_button: Rect::default(),
            load_button: Rect::default(),
            reset_flag: false,
            loadB: vec![Box::new(button)],
            focus: 0,
        }
    }

    fn step(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.current_event = data.sim.step();
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
                data.current_event = Some(block_on(e));
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
        data.input_string.replace(String::from(""));
        data.history_vec.borrow_mut().clear();
        data.reset_flag += 1;

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

// Divides rect into NUM_SEGMENTS equal segments
// Creates a rect based on index and size (# of segments)
fn create_rect(index: u16, size: u16, area:Rect) -> Rect {
    const NUM_SEGMENTS: u16 = 8;
    const MARGIN_FRACTION: u16 = 200;
    Rect::new(
        area.width/NUM_SEGMENTS*index + area.width/MARGIN_FRACTION,
        area.y,
        area.width/NUM_SEGMENTS*size - 2*area.width/MARGIN_FRACTION,
        area.height)
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
        let mut b3Colour = Colour::Yellow;
        let mut b4Colour = Colour::White;

        if self.focus > 0 {
            bColour = Colour::Red
        }

        if self.focus != 4 {
            self.reset_flag = false;
        }

        if data.sim.get_state() == State::RunningUntilEvent {
            b1Colour = Colour::Yellow;
        }

        match self.focus {
            1 => mColour = Colour::Red,
            2 => b1Colour = Colour::Red,
            3 => b2Colour = Colour::Red,
            4 => b3Colour = Colour::Red,
            5 => b4Colour = Colour::Red,
            _ => {},
        }

        let mut bg_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(bColour))
            .title("");

        bg_block.draw(area, buf);

        let area = bg_block.inner(area);
        let area = Rect::new(area.x, area.y, area.width, area.height);

//        let mut bg_block = Block::default()
//            .style(Style::default().bg(mColour))
//            .title("");
//
//        bg_block.draw(area, buf);

        let state_block = create_rect(0, 1, area);
        let cur_event_block = create_rect(1, 3, area);
        self.execution_control_button = create_rect(4, 1, area);
        self.step_button = create_rect(5, 1, area);
        self.reset_button = create_rect(6, 1, area);
        self.load_button = create_rect(7, 1, area);

        let mut state_color = Color::White;
        let state = match data.sim.get_state() {
            State::Halted => "HALTED",
            State::Paused => "PAUSED",
            State::RunningUntilEvent => "RUNNING",
        };
        let state_text = [TuiText::styled(state, Style::default().fg(state_color))];
        let mut para = Paragraph::new(state_text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state_color))
                .title("Current State"))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(state_block, buf);

        let event = match data.get_current_event() {
            Some(event) => {
                match event {
                    Event::Breakpoint {addr} => format!("Breakpoint at {:#x}!", addr),
                    Event::MemoryWatch {addr, data} => format!("Watchpoint at {:#x} with data {:#x}!", addr, data),
                    Event::Error {err} => {
                        state_color = Color::Red;
                        format!("Error: {}!", err)
                    },
                    Event::Interrupted => format!("Interrupted!"),
                    Event::Halted => format!("Halted!"),
                }
            },
            None => format!(""),
        };
        let event_text = [TuiText::styled(event, Style::default().fg(state_color))];
        let mut para = Paragraph::new(event_text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state_color))
                .title("Current Event"))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(cur_event_block, buf);

        let mut vec = Vec::new();
        if data.sim.get_state() == State::RunningUntilEvent{
            vec.push(TuiText::styled("Pause", Style::default().fg(Colour::Yellow)));
        } else {
            vec.push(TuiText::styled("Run", Style::default().fg(Colour::Green)));
        }
        let mut para = Paragraph::new(vec.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(b1Colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.execution_control_button,buf);

        let text = [TuiText::styled("Step", Style::default().fg(Colour::Magenta))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(b2Colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.step_button,buf);

        let mut vec = Vec::new();
        if self.reset_flag {
            vec.push(TuiText::styled(s!(ResetConfirmationMsg), Style::default().fg(Colour::Red)));
        } else {
            vec.push(TuiText::styled("Reset", Style::default().fg(Colour::Yellow)));
        }
        para = Paragraph::new(vec.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(b3Colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.reset_button,buf);

        bg_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(b4Colour))
            .title("");
        bg_block.draw(self.load_button, buf);

        Widget::draw(&mut *self.loadB[0], data, bg_block.inner(self.load_button), buf);

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

                data.log(format!("[modeline] Got an event! {:?}\n", event), Color::Blue);

                assert!(data.current_event.is_none()); // We're being defensive; I thini this holds.
                data.current_event = Some(event);
            }
        }

        if self.focus == 0 {
            if event != WidgetEvent::Update {
                self.focus = 2;
            }
        }

        match event {
            Focus(FocusEvent::GotFocus) => {true},
            Focus(FocusEvent::LostFocus) => {self.focus = 0; false},
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, x, y, _)) => {
                if self.execution_control_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 2;
                    if data.sim.get_state() == State::RunningUntilEvent{
                        self.pause(data)
                    } else {
                        self.run(data);
                    }
                } else if self.step_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 3;
                    self.step(data);
                } else if self.reset_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 4;
                    if self.reset_flag{
                        self.reset(data);
                        self.reset_flag = false;
                    } else {
                        self.reset_flag = true;
                    }
                } else if self.load_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = 5;
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
                KeyEvent { code: KeyCode::Char('r'), modifiers: KeyModifiers::ALT } => {
                    self.focus = 4;
                    if self.reset_flag{
                        self.reset(data);
                        self.reset_flag = false;
                    } else {
                        self.reset_flag = true;
                    }
                    true
                }
                KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL } => {
                    self.loadB[0].update(event, data, terminal);
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
                        4 => {
                             if self.reset_flag{
                                self.reset(data);
                                self.reset_flag = false;
                            } else {
                                self.reset_flag = true;
                            }
                        }
                        5 => {self.loadB[0].update(event, data, terminal);},
                        _ => {},
                    }
                    true
                }
                KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::CONTROL } => {
                    if self.focus < 5 {
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
