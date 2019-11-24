//! A command line simulator for the LC-3 with additional peripherals.
//!
//! TODO!

// TODO: forbid

#![warn(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
// TODO: deny
#![warn(
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    missing_docs,
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

use crossterm::{input, AlternateScreen, InputEvent, KeyEvent, RawScreen};

use tui::backend::CrosstermBackend;
use tui::Terminal;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};

use std::io::stdout;

use lc3_isa::{Addr, Instruction, Reg, Word};
use lc3_traits::control::{Control, State};
use lc3_traits::peripherals::adc::{AdcPin, AdcPinArr, AdcReadError, AdcState};
use lc3_traits::peripherals::gpio::{GpioPin, GpioPinArr, GpioReadError, GpioState};
use lc3_traits::peripherals::pwm::{PwmPin, PwmPinArr, PwmState};
use lc3_traits::peripherals::timers::{TimerArr, TimerId, TimerState};

use lc3_baseline_sim::interp::{
    InstructionInterpreter, InstructionInterpreterPeripheralAccess, Interpreter,
    InterpreterBuilder, MachineState, PeripheralInterruptFlags,
};
use lc3_baseline_sim::sim::Simulator;

use lc3_shims::memory::{FileBackedMemoryShim, MemoryShim};
use lc3_shims::peripherals::PeripheralsShim;

use std::convert::TryInto;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum Event<I> {
    Input(I),
    Tick,
}

struct Cli {
    tick_rate: u64,
    log: bool,
}

fn main() -> Result<(), failure::Error> {
    let file: String = format!("test_prog.mem");

    let _flags: PeripheralInterruptFlags = PeripheralInterruptFlags::new();
    //let mut memory = FileBackedMemoryShim::from(&file);
    //let memory = MemoryShim::default();
    let mut memory = FileBackedMemoryShim::from_existing_file(&file).unwrap();

    let mut interp: Interpreter<'_, _, PeripheralsShim<'_>> = InterpreterBuilder::new() //.build();
        .with_defaults()
        .with_memory(memory)
        .with_interrupt_flags_by_ref(&_flags)
        .build();

    interp.reset();

    let mut sim = Simulator::new(interp);

    sim.set_pc(0x3000);

    // sim.reset();

    let screen = AlternateScreen::to_alternate(true)?;
    let backend = CrosstermBackend::with_alternate_screen(screen)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    
    let cli = Cli{
        tick_rate: 250,
        log: true,
    };

    let mut input_mode: bool = false;
    let mut input_out = String::from("");
    let mut console_out = String::from("");

    let mut active: bool = true;
        //stderrlog::new().quiet(!cli.log).verbosity(4).init()?;
        let (tx, rx) = mpsc::channel();
        {
            let tx = tx.clone();
            thread::spawn(move || {
                let input = input();
                let reader = input.read_sync();
                for event in reader {
                    match event {
                        InputEvent::Keyboard(key) => {
                            if let Err(_) = tx.send(Event::Input(key.clone())) {
                                return;
                            }
                            if key == KeyEvent::Char('q') {
                                active = false;
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            });
        }

        {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(Duration::from_millis(cli.tick_rate));
                }
            });
        }
    


    console_out.push_str("Startup Complete \n");

    let mut step = 0;

    let mut i = 0;

    while active {

        if step == 1 {
            sim.step();
        }

        /*let x = terminal.get_cursor();
        let x = match x {
            Ok(data) => data,
            Err(error) => (0,0),
        };
        let x = format!("Cursor: {} {}\n", x.0, x.1);
        console_out.push_str(&x);*/
        

        match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent::Char(c) => {
                    if input_mode == false{
                        match c{
                            's' => if step != 1 { sim.step(); },
                            'p' => step = 0,
                            'r' => step = 1,
                            _ => {}
                        }
                    } else {
                        let x = format!("{}", c);
                        input_out.push_str(&x);
                    }
                }
                KeyEvent::Insert => {
                    if input_mode == false {
                        input_mode = true;
                    } else {
                        input_mode = false;
                        console_out.push_str("Input: ");
                        console_out.push_str(&input_out.clone());
                        console_out.push_str("\n");
                        input_out = String::from("");
                    }
                }
                _ => {}
            },
            Event::Tick => {
            }
        }

        terminal.draw(|mut f| {
            //Creates vertical device for footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(10),
                        Constraint::Length(5),
                    ].as_ref()
                )
                .split(f.size());

            let buttons = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Min(20), Constraint::Length(8), Constraint::Length(8), Constraint::Length(8)].as_ref())
                .split(chunks[1]);

            let body = chunks[0];

            //Divides top half into left and right
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(body);

            //Creates space for Memory and register status windows
            let left_pane = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(6), Constraint::Length(7)].as_ref())
                .split(panes[0]);

            //Creates console output + IO
            let right_pane = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(13), Constraint::Length(16)].as_ref())
                .split(panes[1]);

            let console = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
                .split(right_pane[0]);



            Block::default()
                 .title("> ")
                 .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                 .render(&mut f, console[1]);

            Block::default()
                 .title("IO")
                 .borders(Borders::ALL)
                 .render(&mut f, right_pane[1]);

            //Further breakdown of IO
            let io_panel = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(5), Constraint::Length(3), Constraint::Length(2), Constraint::Length(3)].as_ref())
                .split(right_pane[1]);


            let timers_n_clock = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
                .split(io_panel[3]);

            //TEXT BELOW HERE

            //Footer Text
            let text = [
                Text::styled("To control the TUI, you can use S to Step through instructions, P to Pause, and R to Run, or click the appropriate button", Style::default().modifier(Modifier::BOLD))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Footer")
                            .title_style(Style::default().fg(Color::Red).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, chunks[1]);

            //Footer Buttons
            let text = [
                Text::styled("Step", Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
            ];
            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                )
                .render(&mut f, buttons[1]);

            let text = [
                Text::styled("Pause", Style::default().fg(Color::Red).modifier(Modifier::BOLD))
            ];
            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                )
                .render(&mut f, buttons[2]);

            let text = [
                Text::styled("Run", Style::default().fg(Color::Green).modifier(Modifier::BOLD))
            ];
            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                )
                .render(&mut f, buttons[3]);

            //Register Status Text
            let regs_psr_pc = sim.get_registers_psr_and_pc();
            let (regs, psr, pc) = regs_psr_pc;

            let text = [
                Text::raw(format!("R0:  {:#018b} {:#06x} {:#05}\n", regs[0], regs[0], regs[0])),
                Text::raw(format!("R1:  {:#018b} {:#06x} {:#05}\n", regs[1], regs[1], regs[1])),
                Text::raw(format!("R2:  {:#018b} {:#06x} {:#05}\n", regs[2], regs[2], regs[2])),
                Text::raw(format!("R3:  {:#018b} {:#06x} {:#05}\n", regs[3], regs[3], regs[3])),
                Text::raw(format!("R4:  {:#018b} {:#06x} {:#05}\n", regs[4], regs[4], regs[4]))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Registers + PC + PSR")
                            .title_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, left_pane[1]);

            let right_regs = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(left_pane[1]);

            let text = [
                Text::raw(format!("R5:  {:#018b} {:#06x} {:#05}\n", regs[5], regs[5], regs[5])),
                Text::raw(format!("R6:  {:#018b} {:#06x} {:#05}\n", regs[6], regs[6], regs[6])),
                Text::raw(format!("R7:  {:#018b} {:#06x} {:#05}\n", regs[7], regs[7], regs[7])),
                Text::raw(format!("PSR: {:#018b} {:#06x} {:#05}\n", psr, psr, psr)),
                Text::raw(format!("PC:  {:#018b} {:#06x} {:#05}\n", pc, pc, pc))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::NONE)
                )
                .wrap(true)
                .render(&mut f, right_regs[1]);


            //Memory
            let mut mem: [Word; 50] = [0; 50];
            let mut x: u16 = 0;
            while x != 50 {
                mem[x as usize] = sim.read_word(pc.wrapping_sub(2).wrapping_add(x));
                x = x + 1;
            }


            let mut s =  String::from("");
            x = 0;
            while x != 50 {
                let inst: Instruction = match mem[x as usize].try_into(){
                    Ok(data) => data,
                    Err(error) => Instruction::AddReg{dr: Reg::R0, sr1: Reg::R0, sr2: Reg::R0,},
                };
                if x == 2{
                    s.push_str("|--> ");
                }else{
                    s.push_str("|    ");
                }
                s.push_str(&format!("{:#06x} {:#018b} {:#06x} {:#05}    {}\n", pc.wrapping_sub(2).wrapping_add(x), mem[x as usize], mem[x as usize], mem[x as usize], inst));
                x = x + 1;
            }

            let text = [
                Text::raw(s)
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Memory")
                            .title_style(Style::default().fg(Color::Red).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, left_pane[0]);

            //Console


            let text = [
                Text::raw(console_out.clone())
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                            .title("Output")
                            .title_style(Style::default().fg(Color::Green)),
                )
                .wrap(true)
                .render(&mut f, console[0]);

            let text = [
                Text::raw(input_out.clone())
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                            .title(">")
                            .title_style(Style::default().fg(Color::Green)),
                )
                .wrap(true)
                .render(&mut f, console[1]);

            //IO

            //GPIO
            let GPIO_states = sim.get_gpio_states();
            let gpioin = sim.get_gpio_reading();

            let gpio = match gpioin[GpioPin::G0]{
                Ok(val) => format!("GPIO 0:  {}\n", val),
                Err(e) => format!("GPIO 0:  -\n"),
            };

            let t0 = match GPIO_states[GpioPin::G0] {
                GpioState::Disabled => Text::raw(format!("GPIO 0:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let gpio = match gpioin[GpioPin::G1]{
                Ok(val) => format!("GPIO 1:  {}\n", val),
                Err(e) => format!("GPIO 1:  -\n"),
            };

            let t1 = match GPIO_states[GpioPin::G1] {
                GpioState::Disabled => Text::raw(format!("GPIO 1:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let gpio = match gpioin[GpioPin::G2]{
                Ok(val) => format!("GPIO 2:  {}\n", val),
                Err(e) => format!("GPIO 2:  -\n"),
            };

            let t2 = match GPIO_states[GpioPin::G2] {
                GpioState::Disabled => Text::raw(format!("GPIO 2:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let gpio = match gpioin[GpioPin::G3]{
                Ok(val) => format!("GPIO 3:  {}\n", val),
                Err(e) => format!("GPIO 3:  -\n"),
            };

            let t3 = match GPIO_states[GpioPin::G3] {
                GpioState::Disabled => Text::raw(format!("GPIO 3:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let text = [t0, t1, t2, t3];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                            .title("GPIO")
                            .title_style(Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, io_panel[0]);

            let gpio = match gpioin[GpioPin::G4]{
                Ok(val) => format!("GPIO 4:  {}\n", val),
                Err(e) => format!("GPIO 4:  -\n"),
            };

            let t0 = match GPIO_states[GpioPin::G4] {
                GpioState::Disabled => Text::raw(format!("GPIO 4:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let gpio = match gpioin[GpioPin::G5]{
                Ok(val) => format!("GPIO 5:  {}\n", val),
                Err(e) => format!("GPIO 5:  -\n"),
            };

            let t1 = match GPIO_states[GpioPin::G5] {
                GpioState::Disabled => Text::raw(format!("GPIO 5:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let gpio = match gpioin[GpioPin::G6]{
                Ok(val) => format!("GPIO 6:  {}\n", val),
                Err(e) => format!("GPIO 6:  -\n"),
            };

            let t2 = match GPIO_states[GpioPin::G6] {
                GpioState::Disabled => Text::raw(format!("GPIO 6:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let gpio = match gpioin[GpioPin::G7]{
                Ok(val) => format!("GPIO 7:  {}\n", val),
                Err(e) => format!("GPIO 7:  -\n"),
            };

            let t3 = match GPIO_states[GpioPin::G7] {
                GpioState::Disabled => Text::raw(format!("GPIO 7:  Disabled\n")),
                _ => Text::raw(gpio),
            };

            let text = [t0, t1, t2, t3];

            let right_GPIO = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(io_panel[0]);

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::TOP | Borders::RIGHT)
                )
                .wrap(true)
                .render(&mut f, right_GPIO[1]);

            //ADC
            let ADC_states = sim.get_adc_states();
            let adcin = sim.get_adc_reading();

            let adc = match adcin[AdcPin::A0] {
                Ok(number) => format!("ADC 0:   {:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(e) => format!("ADC 0:   -                  -      -\n"),
            };

            let t0 = match ADC_states[AdcPin::A0] {
                AdcState::Disabled => Text::raw(format!("ADC 0:   Disabled\n")),
                AdcState::Enabled => Text::raw(adc),
            };

            let adc = match adcin[AdcPin::A1] {
                Ok(number) => format!("ADC 1:   {:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(e) => format!("ADC 1:   -                  -      -\n"),
            };

            let t1 = match ADC_states[AdcPin::A1] {
                AdcState::Disabled => Text::raw(format!("ADC 1:   Disabled\n")),
                AdcState::Enabled => Text::raw(adc),
            };

            let text = [t0, t1];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                            .title("ADC")
                            .title_style(Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, io_panel[1]);

            let right_ADC = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(io_panel[1]);

            let adc = match adcin[AdcPin::A2] {
                Ok(number) => format!("ADC 2:   {:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(e) => format!("ADC 2:   -                  -      -\n"),
            };

            let t0 = match ADC_states[AdcPin::A2] {
                AdcState::Disabled => Text::raw(format!("ADC 2:   Disabled\n")),
                AdcState::Enabled => Text::raw(adc),
            };

            let adc = match adcin[AdcPin::A3] {
                Ok(number) => format!("ADC 3:   {:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(e) => format!("ADC 3:   -                  -      -\n"),
            };

            let t1 = match ADC_states[AdcPin::A3] {
                AdcState::Disabled => Text::raw(format!("ADC 3:   Disabled\n")),
                AdcState::Enabled => Text::raw(adc),
            };

            let text = [t0,t1];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::TOP | Borders::RIGHT)
                )
                .wrap(true)
                .render(&mut f, right_ADC[1]);

            //PWM
            let PWM_states = sim.get_pwm_states();
            let PWM = sim.get_pwm_config();

            let text = match PWM_states[PwmPin::P0] {
                PwmState::Disabled => [Text::raw(format!("PWM 0:   Disabled"))],
                PwmState::Enabled(_) => [Text::raw(format!("PWM 0:   {:#018b} {:#06x} {:#05}\n", PWM[PwmPin::P0], PWM[PwmPin::P0], PWM[PwmPin::P0]))],
            };

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                            .title("PWM")
                            .title_style(Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, io_panel[2]);

            let right_PWM = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(io_panel[2]);

            let text = match PWM_states[PwmPin::P1] {
                PwmState::Disabled => [Text::raw(format!("PWM 1:   Disabled"))],
                PwmState::Enabled(_) => [Text::raw(format!("PWM 1:   {:#018b} {:#06x} {:#05}\n", PWM[PwmPin::P1], PWM[PwmPin::P1], PWM[PwmPin::P1]))],
            };

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::TOP | Borders::RIGHT)
                )
                .wrap(true)
                .render(&mut f, right_PWM[1]);

            //Timers
            let timer_state = sim.get_timer_states();
            let timer = sim.get_timer_config();

            let t0 = match timer_state[TimerId::T0] {
                TimerState::Disabled => Text::raw(format!("Timer 1: Disabled\n")),
                TimerState::Repeated => Text::raw(format!("Repeat:  {:#018b} {:#06x} {:#05}\n", timer[TimerId::T0], timer[TimerId::T0], timer[TimerId::T0])),
                TimerState::SingleShot => Text::raw(format!("Single:  {:#018b} {:#06x} {:#05}\n", timer[TimerId::T0], timer[TimerId::T0], timer[TimerId::T0])),
            };

            let t1 = match timer_state[TimerId::T1] {
                TimerState::Disabled => Text::raw(format!("Timer 2: Disabled\n")),
                TimerState::Repeated => Text::raw(format!("Repeat:  {:#018b} {:#06x} {:#05}\n", timer[TimerId::T1], timer[TimerId::T1], timer[TimerId::T1])),
                TimerState::SingleShot => Text::raw(format!("Single:  {:#018b} {:#06x} {:#05}\n", timer[TimerId::T1], timer[TimerId::T1], timer[TimerId::T1])),
            };

            let text = [t0,t1];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL & !(Borders::RIGHT))
                            .title("Timers")
                            .title_style(Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, timers_n_clock[0]);

            //Clock
            let clock = sim.get_clock();

            let text = [
                Text::raw(format!("{:#018b} {:#06x} {:#05}\n", clock, clock, clock))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Clock")
                            .title_style(Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, timers_n_clock[1]);

        })?;
    }

    Ok(())
}
