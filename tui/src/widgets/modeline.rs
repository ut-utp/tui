//! TODO!

use super::widget_impl_support::*;
use ModelineFocus::*;

use tui::widgets::Paragraph;
use tui::style::{Color, Style};

use core::future::Future;
use core::task::{Context, Waker, Poll};

use lc3_traits::control::{Event, State};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelineFocus {
    NoFocus,
    ExecutionControl,
    StepOver,
    StepIn,
    StepOut,
    Reset,
    Load,
}

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
    color: Color,
    execution_control_button: Rect,
    step_over_button: Rect,
    step_in_button: Rect,
    step_out_button: Rect,
    reset_button: Rect,
    load_button: Rect,
    reset_flag: bool,
    load_b: Vec<Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>>,
    focus: ModelineFocus,
}

impl<'a, 'int, C, I, O, B> Modeline<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub fn new<W: Widget<'a, 'int, C, I, O, B> + 'a>(button: W) -> Self {
        Self::new_with_color(button, Color::Blue)
    }

    pub fn new_with_color<W: Widget<'a, 'int, C, I, O, B> + 'a>(button: W, color:Color) -> Self {
        Self {
            event_fut: None,
            color,
            execution_control_button: Rect::default(),
            step_over_button: Rect::default(),
            step_in_button: Rect::default(),
            step_out_button: Rect::default(),
            reset_button: Rect::default(),
            load_button: Rect::default(),
            reset_flag: false,
            load_b: vec![Box::new(button)],
            focus: NoFocus,
        }
    }

    fn step(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.current_event = data.sim.step();
    }

    fn pause(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.sim.pause();
    }

    fn load(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) {
        self.load_b[0].update(event, data, terminal);
        data.reset_flag = data.reset_flag.wrapping_add(1);
        drop(data.current_event.take())
    }
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
        data.log("[modeline] Resetting Sim\n", Color::Magenta);
        data.sim.reset();
        data.input_string.replace(String::from(""));
        data.console_hist.borrow_mut().clear();
        data.mem_reg_inter = (0,0);
        data.reset_flag = data.reset_flag.wrapping_add(1);

        // Resolve the pending future, if there is one.
        if let Some(e) = self.event_fut.take() {
            data.sim.step();
            block_on(e);
        }

        data.log("[modeline] Reset Complete\n", Color::Green);
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
    const NUM_SEGMENTS: u16 = 10;
    const MARGIN_FRACTION: u16 = 400;
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
        let mut box_color = self.color;
        let mut execution_color = Color::Green;
        let mut step_over_color = Color::Cyan;
        let mut step_in_color = Color::Cyan;
        let mut step_out_color = Color::Cyan;
        let mut reset_color = Color::Yellow;
        let mut load_color = Color::White;

        if self.focus != NoFocus {
            box_color = Color::Red
        }

        if self.focus != Reset {
            self.reset_flag = false;
        }

        if data.sim.get_state() == State::RunningUntilEvent {
            execution_color = Color::Yellow;
        }

        let focus_color = Color::Red;
        match self.focus {
            ExecutionControl => execution_color = focus_color,
            StepOver => step_over_color = focus_color,
            StepIn => step_in_color = focus_color,
            StepOut => step_out_color = focus_color,
            Reset => reset_color = focus_color,
            Load => load_color = focus_color,
            NoFocus => {},
        };

        let mut bg_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(box_color))
            .title("");

        bg_block.draw(area, buf);

        let area = bg_block.inner(area);
        let area = Rect::new(area.x, area.y, area.width, area.height);

//        let mut bg_block = Block::default()
//            .style(Style::default().bg(mColor))
//            .title("");
//
//        bg_block.draw(area, buf);

        let state_block = create_rect(0, 1, area);
        let cur_event_block = create_rect(1, 3, area);
        self.step_over_button = create_rect(4, 1, area);
        self.step_in_button = create_rect(5, 1, area);
        self.step_out_button = create_rect(6, 1, area);
        self.execution_control_button = create_rect(7, 1, area);
        self.reset_button = create_rect(8, 1, area);
        self.load_button = create_rect(9, 1, area);

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
                    Event::DepthBreakpoint => format!(""),      // TODO: Decide whether or not to show event on depth breakpoint
                    Event::Error {err} => {
                        state_color = Color::LightRed;
                        format!("Error: {}!", err)
                    },
                    Event::Interrupted => format!("Interrupted!"),
                    Event::Halted => format!("Halted!"),
                    _ => format!("")
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

        let text = [TuiText::styled("Step Over", Style::default().fg(step_over_color))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(step_over_color))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.step_over_button,buf);

        let text = [TuiText::styled("Step In", Style::default().fg(step_in_color))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(step_in_color))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.step_in_button,buf);

        let text = [TuiText::styled("Step Out", Style::default().fg(step_out_color))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(step_out_color))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.step_out_button,buf);

        let mut vec = Vec::new();
        if data.sim.get_state() == State::RunningUntilEvent{
            vec.push(TuiText::styled("Pause", Style::default().fg(Color::Yellow)));
        } else {
            vec.push(TuiText::styled("Run", Style::default().fg(Color::Green)));
        }
        let mut para = Paragraph::new(vec.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(execution_color))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.execution_control_button,buf);

        let mut vec = Vec::new();
        if self.reset_flag {
            vec.push(TuiText::styled(s!(ResetConfirmationMsg), Style::default().fg(Color::Red)));
        } else {
            vec.push(TuiText::styled("Reset", Style::default().fg(Color::Yellow)));
        }
        para = Paragraph::new(vec.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(reset_color))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.draw(self.reset_button,buf);

        bg_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(load_color))
            .title("");
        bg_block.draw(self.load_button, buf);

        Widget::draw(&mut *self.load_b[0], data, bg_block.inner(self.load_button), buf);

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
                
                assert!(data.current_event.is_none()); // We're being defensive; I thini this holds.
                let event_color = match event {
                    Event::Breakpoint {addr} => Color::Red,
                    Event::MemoryWatch {addr, data} => Color::Rgb(0x30, 0x49, 0xDE),
                    Event::Error {err} => Color::LightRed,
                    Event::Interrupted => Color::Yellow,
                    Event::Halted => Color::Gray,
                    _ => Color::DarkGray,
                };
                data.log(format!("[modeline] Got an event! {:?}\n", event), event_color);
                data.current_event = Some(event);
            }
        }

        if self.focus == NoFocus {
            if event != Update {
                self.focus = ExecutionControl;
            }
        }

        match event {
            Focus(FocusEvent::GotFocus) => {true},
            Focus(FocusEvent::LostFocus) => {self.focus = NoFocus; false},
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, x, y, _)) => {
                if self.execution_control_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = ExecutionControl;
                    if data.sim.get_state() == State::RunningUntilEvent{
                        self.pause(data)
                    } else {
                        self.run(data);
                    }
                } else if self.step_over_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = StepOver;
                    data.sim.set_relative_depth_breakpoint(0);
                    self.run(data);
                } else if self.step_in_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = StepIn;
                    self.step(data);
                } else if self.step_out_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = StepOut;
                    data.sim.set_relative_depth_breakpoint(-1);
                    self.run(data);
                } else if self.reset_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = Reset;
                    if self.reset_flag{
                        self.reset(data);
                        self.reset_flag = false;
                    } else {
                        self.reset_flag = true;
                    }
                } else if self.load_button.intersects(Rect::new(x,y,1,1)) {
                    self.load(event, data, terminal);
                    self.focus = Load;
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
                    self.focus = Reset;
                    if self.reset_flag{
                        self.reset(data);
                        self.reset_flag = false;
                    } else {
                        self.reset_flag = true;
                    }
                    true
                }
                KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL } => {
                    self.load(event, data, terminal);
                    true
                }
                KeyEvent { code: KeyCode::Enter, modifiers: EMPTY } => {
                    match self.focus {
                        ExecutionControl => {
                            if data.sim.get_state() == State::RunningUntilEvent{
                                self.pause(data)
                            } else {
                                self.run(data);
                            }
                        },
                        StepOver => self.step(data),
                        StepIn => {
                            data.sim.set_relative_depth_breakpoint(0);
                            self.run(data);
                        }
                        StepOut => {
                            data.sim.set_relative_depth_breakpoint(-1);
                            self.run(data);
                        },
                        Reset => {
                             if self.reset_flag{
                                self.reset(data);
                                self.reset_flag = false;
                            } else {
                                self.reset_flag = true;
                            }
                        }
                        Load => {self.load(event, data, terminal)},
                        NoFocus => {},
                    }
                    true
                }
                KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::CONTROL } => {
                    self.focus = match self.focus {
                        StepOver => StepIn,
                        StepIn => StepOut,
                        StepOut => ExecutionControl,
                        ExecutionControl => Reset,
                        Reset => Load,
                        Load => Load,
                        NoFocus => NoFocus,
                    };
                    true
                }
                KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::CONTROL } => {
                    self.focus = match self.focus {
                        StepOver => StepOver,
                        StepIn => StepOver,
                        StepOut => StepIn,
                        ExecutionControl => StepOut,
                        Reset => ExecutionControl,
                        Load => Reset,
                        NoFocus => NoFocus,
                    };
                    true
                }
                _ => false,
            }
            _ => false,
        }
    }
}
