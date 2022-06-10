//! TODO!

use super::widget_impl_support::*;
use ModelineFocus::*;

use core::future::Future;
use core::task::{Context, Waker, Poll};

use lc3_traits::control::{Event, State, StepControl};
use lc3_os::USER_PROG_START_ADDR;
use lc3_isa::{Addr, OS_START_ADDR};

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
    colour: Colour,
    execution_control_button: Rect,
    step_over_button: Rect,
    step_in_button: Rect,
    step_out_button: Rect,
    reset_button: Rect,
    load_button: Rect,
    reset_flag: bool,
    load_flag: u8,
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
        Self::new_with_colour(button, c!(Modeline))
    }

    pub fn new_with_colour<W: Widget<'a, 'int, C, I, O, B> + 'a>(button: W, colour:Colour) -> Self {
        Self {
            event_fut: None,
            colour,
            execution_control_button: Rect::default(),
            step_over_button: Rect::default(),
            step_in_button: Rect::default(),
            step_out_button: Rect::default(),
            reset_button: Rect::default(),
            load_button: Rect::default(),
            reset_flag: false,
            load_flag: 0,
            load_b: vec![Box::new(button)],
            focus: NoFocus,
        }
    }

    fn step(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.current_event = data.sim.step();
    }

    fn step_in(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        StepControl::step_in(data.sim);
        self.run(data);
    }

    fn step_out(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        StepControl::step_out(data.sim);
        self.run(data);
    }

    fn step_over(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        StepControl::step_over(data.sim);
        self.run(data);
    }

    fn pause(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.sim.pause();
    }

    fn load(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) {
        if self.load_b[0].update(event, data, terminal) {
            data.load_flag = data.load_flag.wrapping_add(1);
            self.reset(data)
        }
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

        if self.focus == StepOver || self.focus == StepIn || self.focus == StepOut {
            self.focus = ExecutionControl;
        }

        drop(data.current_event.take())
    }

    fn skip_os(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        // Only do this if we're using the OS.
        if !data.use_os {
            return;
        }

        let start_addr: Addr = data.sim.read_word(USER_PROG_START_ADDR);

        // If the start addr is 0 somehow, assume that we're not actually using
        // the OS / that something went wrong.
        //
        // Also don't do this if we're not at the OS start location.

        if start_addr == 0 {
            log::warn!("No user prog start address specified at {:#4X}.",
                USER_PROG_START_ADDR);

            return;
        }

        if data.sim.get_pc() != OS_START_ADDR {
            log::warn!("Tried to skip the OS startup when not at the \
                OS Start location! Expected to be at {:#4X}; was actually at \
                {:#4X}.", data.sim.get_pc(), OS_START_ADDR);

            return;
        }

        // kludge to support breakpoints at _exactly_ 0x200:
        if data.sim.get_breakpoints()
            .iter()
            .filter_map(|a| *a)
            .any(|addr| addr == OS_START_ADDR)
        {
            log::trace!("BP on OS Start Address ({:#4X})!", OS_START_ADDR);

            return;
        }

        while data.sim.get_pc() != start_addr {
            // If we get an event while running the OS startup, bail.
            //
            // This allows users to set breakpoints in the OS if they so choose.
            if let Some(event) = data.current_event {
                log::trace!("Got an event while trying to skip past the OS! \
                    At PC = {:#4X}, event: `{:?}`.", data.sim.get_pc(), event);

                return;
            }

            self.step(data);
        }
    }

    fn reset(&mut self, data: &mut TuiData<'a, 'int, C, I, O>) {
        data.log("[modeline] Resetting Sim\n", c!(Pause));
        data.sim.reset();
        data.console_input_string.get_mut().clear();
        data.console_hist.get_mut().clear();
        data.mem_reg_inter = (0,0);
        data.reset_flag = data.reset_flag.wrapping_add(1);

        // Resolve the pending future, if there is one.
        if let Some(e) = self.event_fut.take() {
            data.sim.step();
            block_on(e);
        }

        // TODO: find a better workaround than this:
        data.sim.reset();

        self.skip_os(data);

        data.log("[modeline] Reset Complete\n", c!(Success));
        drop(data.current_event.take())
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
        let mut box_colour = self.colour;
        let mut execution_colour = c!(Run);
        let mut step_over_colour = c!(StepButtons);
        let mut step_in_colour = c!(StepButtons);
        let mut step_out_colour = c!(StepButtons);
        let mut reset_colour = c!(Pause);
        let mut load_colour = c!(LoadB);

        if self.focus != NoFocus {
            box_colour = c!(Focus);
        }

        // TODO: use a time based thing to unset the reset_flag instead of stealing
        // focus and unsetting it when losing focus..
        if self.focus != Reset {
            self.reset_flag = false;
        }

        let running = data.sim.get_state() == State::RunningUntilEvent;

        if running {
            execution_colour = c!(Pause);
            step_over_colour = c!(Disabled);
            step_in_colour = c!(Disabled);
            step_out_colour = c!(Disabled);
        }

        match self.focus {
            ExecutionControl => execution_colour = c!(Focus),
            StepOver => step_over_colour = c!(Focus),
            StepIn => step_in_colour = c!(Focus),
            StepOut => step_out_colour = c!(Focus),
            Reset => reset_colour = c!(Focus),
            Load => load_colour = c!(Focus),
            NoFocus => {},
        };

        let mut bg_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(box_colour))
            .title("");

        bg_block.render(area, buf);

        let area = bg_block.inner(area);
        let area = Rect::new(area.x, area.y, area.width, area.height);

//        let mut bg_block = Block::default()
//            .style(Style::default().bg(mColour))
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

        let state = match data.sim.get_state() {
            State::Halted => "HALTED",
            State::Paused => "PAUSED",
            State::RunningUntilEvent => "RUNNING",
        };
        let state_text = [TuiText::styled(state, Style::default().fg(self.colour))];
        let mut para = Paragraph::new(state_text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.colour))
                .title("Current State"))
            .alignment(Alignment::Center)
            .wrap(true);
        para.render(state_block, buf);

        let mut box_colour   = self.colour;
        let event = match data.get_current_event() {
            Some(event) => {
                match event {
                    Event::Breakpoint {addr} => format!("Breakpoint at {:#x}!", addr),
                    Event::MemoryWatch {addr, data} => format!("Watchpoint at {:#x} with data {:#x}!", addr, data),
                    Event::DepthReached { current_depth } => format!(""),      // TODO: Decide whether or not to show event on depth breakpoint
                    Event::Error {err} => {
                        box_colour = c!(Error);
                        format!("Error: {}!", err)
                    },
                    Event::Interrupted => format!("Interrupted!"),
                    Event::Halted => format!("Halted!"),
                    _ => format!("")
                }
            },
            None => format!(""),
        };
        let event_text = [TuiText::styled(event, Style::default().fg(box_colour))];
        let mut para = Paragraph::new(event_text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(box_colour))
                .title("Current Event"))
            .alignment(Alignment::Left)
            .wrap(true);
        para.render(cur_event_block, buf);

        let text = [TuiText::styled("Step Over", Style::default().fg(step_over_colour))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(step_over_colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.render(self.step_over_button,buf);

        let text = [TuiText::styled("Step In", Style::default().fg(step_in_colour))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(step_in_colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.render(self.step_in_button,buf);

        let text = [TuiText::styled("Step Out", Style::default().fg(step_out_colour))];
        para = Paragraph::new(text.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(step_out_colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.render(self.step_out_button,buf);

        let mut vec = Vec::new();
        if running {
            vec.push(TuiText::styled("Pause", Style::default().fg(c!(Pause))));
        } else {
            vec.push(TuiText::styled("Run", Style::default().fg(c!(Run))));
        }
        let mut para = Paragraph::new(vec.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(execution_colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.render(self.execution_control_button,buf);

        let mut vec = Vec::new();
        if self.reset_flag {
            vec.push(TuiText::styled(s!(ResetConfirmationMsg), Style::default().fg(c!(Reset))));
        } else {
            vec.push(TuiText::styled("Reset", Style::default().fg(c!(Pause))));
        }
        para = Paragraph::new(vec.iter())
            .style(Style::default())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(reset_colour))
                .title(""))
            .alignment(Alignment::Center)
            .wrap(true);
        para.render(self.reset_button,buf);

        bg_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(load_colour))
            .title("");
        bg_block.render(self.load_button, buf);

        Widget::draw(&mut *self.load_b[0], data, bg_block.inner(self.load_button), buf);

    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        // If we're currently running an event and the state changes, we've got
        // ourselves an event.
        let running = data.sim.get_state() == State::RunningUntilEvent;

        if let Some(_) = self.event_fut {
            if !running {
                let event = block_on(self.event_fut.take().unwrap());

                assert!(data.current_event.is_none()); // We're being defensive; I thini this holds.
                let event_colour = match event {
                    Event::Breakpoint {addr} => c!(Breakpoint),
                    Event::MemoryWatch {addr, data} => c!(Watchpoint),
                    Event::Error {err} => c!(Error),
                    Event::Interrupted => c!(Pause),
                    Event::Halted => c!(Halted),
                    _ => c!(mDefault),
                };
                data.log(format!("[modeline] Got an event! {:?}\n", event), event_colour);
                data.current_event = Some(event);
            }
        }

        if self.focus == NoFocus {
            if event != Update {
                self.focus = ExecutionControl;
            }
        }

        if self.load_flag != data.load_flag {
            self.skip_os(data);
            self.load_flag = data.load_flag;
        }

        match event {
            Focus(FocusEvent::GotFocus) => {true},
            Focus(FocusEvent::LostFocus) => {self.focus = NoFocus; false},
            // Mouse(MouseEvent::Up(_, _, _, _)) => true,
            // Mouse(MouseEvent::Down(_, x, y, _)) => {
            Mouse(MouseEvent::Down(_, _, _, _)) => true,
            Mouse(MouseEvent::Up(_, x, y, _)) => {
                if self.execution_control_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = ExecutionControl;
                    if running {
                        self.pause(data)
                    } else {
                        self.run(data);
                    }
                } else if !running && self.step_over_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = StepOver;
                    self.step_over(data);
                } else if !running && self.step_in_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = StepIn;
                    // This is eq. to step, but we'll do this for consistency.
                    self.step_in(data);
                } else if !running && self.step_out_button.intersects(Rect::new(x,y,1,1)) {
                    self.focus = StepOut;
                    self.step_out(data);
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
                KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::ALT } |
                KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL } => {
                    if !running { self.step_over(data); }
                    true
                }

                KeyEvent { code: KeyCode::Char('i'), modifiers: KeyModifiers::ALT } |
                KeyEvent { code: KeyCode::Char('i'), modifiers: KeyModifiers::CONTROL } => {
                    if !running { self.step(data); }
                    true
                }

                KeyEvent { code: KeyCode::Char('o'), modifiers: KeyModifiers::ALT } |
                KeyEvent { code: KeyCode::Char('o'), modifiers: KeyModifiers::CONTROL } => {
                    if !running { self.step_out(data); }
                    true
                }

                KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL } => {
                    if !running {
                        self.step(data);
                    }
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
                KeyEvent { code: KeyCode::Char('t'), modifiers: KeyModifiers::CONTROL } => {
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
                            if running {
                                self.pause(data)
                            } else {
                                self.run(data);
                            }
                        },
                        StepOver => {
                            if !running { self.step_over(data) }
                        },
                        StepIn => {
                            if !running { self.step(data); }
                        }
                        StepOut => {
                            if !running { self.step_out(data) }
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
                    if running && self.focus == StepOut {
                        self.focus = ExecutionControl;
                    }
                    true
                }
                _ => false,
            }
            _ => false,
        }
    }
}
