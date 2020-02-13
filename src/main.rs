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

use lc3_isa::{Addr, Instruction, Reg, Word};
use lc3_traits::control::metadata::{DeviceInfo, Identifier, ProgramId, ProgramMetadata};
use lc3_traits::control::rpc::SyncEventFutureSharedState;
use lc3_traits::control::{Control, State};
use lc3_traits::peripherals::adc::{AdcPin, AdcState};
use lc3_traits::peripherals::gpio::{GpioPin, GpioState};
use lc3_traits::peripherals::pwm::{PwmPin, PwmState};
use lc3_traits::peripherals::timers::{TimerId, TimerState};
use lc3_traits::peripherals::PeripheralSet;
use lc3_traits::control::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};

use lc3_shims::peripherals::{
    AdcShim, ClockShim, GpioShim, InputShim, OutputShim, PwmShim, SourceShim, TimersShim,
};

use lc3_traits::control::rpc::encoding::Transparent;
use lc3_traits::control::rpc::mpsc_sync_pair;
use lc3_traits::control::rpc::*;

use lc3_baseline_sim::interp::{
    InstructionInterpreter, Interpreter, InterpreterBuilder, PeripheralInterruptFlags,
};
use lc3_baseline_sim::sim::Simulator;

use lc3_shims::memory::FileBackedMemoryShim;
use lc3_shims::peripherals::PeripheralsShim;

use std::convert::TryInto;

use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

use std::sync::mpsc::{Receiver, Sender};
use std::time;

use std::time::Duration;

// use serde::{Deserialize, Serialize};
use log::{info, warn};
extern crate flexi_logger;

use flexi_logger::{opt_format, Logger};

// use std::fs::File;

// use std::borrow::Cow::Borrowed;

use core::num::NonZeroU8;

// use std::process;

use std::collections::HashMap;

enum Event<I> {
    Input(I),
    Tick,
}

struct Cli {
    tick_rate: u64,
    log: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TuiState {
    CONT,
    IN,
    GPIO,
    ADC,
    PWM,
    TMR,
    CLK,
    REG,
    PC,
    MEM,
}

lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE_CONTROLLER: SyncEventFutureSharedState = SyncEventFutureSharedState::new();
}

lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE_SIM: SyncEventFutureSharedState = SyncEventFutureSharedState::new();
}

fn main() -> Result<(), failure::Error> {
    let file: String = format!("test_prog.mem");

    let memory = FileBackedMemoryShim::from_existing_file(&file).unwrap();
    let gpio_shim = Arc::new(RwLock::new(GpioShim::default()));
    let adc_shim = Arc::new(RwLock::new(AdcShim::default()));
    let pwm_shim = Arc::new(RwLock::new(PwmShim::default()));
    let timer_shim = Arc::new(RwLock::new(TimersShim::default()));
    let clock_shim = Arc::new(RwLock::new(ClockShim::default()));

    let source_shim = Box::new(SourceShim::new());
    let source_shim = Box::leak(source_shim);
    let input_shim = Arc::new(Mutex::new(InputShim::with_ref(source_shim)));

    let mut console_output_string: String = String::new();
    let mut last_idx = 0;

    let console_output = Box::new(Mutex::new(Vec::new()));
    let console_output = Box::leak(console_output);
    // let console_output_sink: &(dyn lc3_shims::peripherals::Sink + Send + Sync) = &console_output;
    let output_shim = Arc::new(Mutex::new(OutputShim::with_ref(console_output)));

    let mut iteratively_collect_into_console_output = || {
        let vec = console_output.lock().unwrap();

        // if console_output_string.len() > 5000 {
        //     let _ = console_output_string.drain(0..(console_output_string.len() - 2000));
        //     // Only keep the last 2000 characters
        // }

        if vec.len() > last_idx {
            vec[last_idx..].iter().for_each(|c| {
                console_output_string.push(*c as char);
            });

            last_idx = vec.len();
        }
        console_output_string.clone()
    };

    let (controller, mut device) = mpsc_sync_pair::<
        RequestMessage,
        ResponseMessage,
        Transparent<_>,
        Transparent<_>,
        Transparent<_>,
        Transparent<_>,
        _,
    >(&EVENT_FUTURE_SHARED_STATE_CONTROLLER);

    let gshim = gpio_shim.clone();
    let ashim = adc_shim.clone();
    let pshim = pwm_shim.clone();
    let ishim = input_shim.clone();
    let oshim = output_shim.clone();

    use std::thread::Builder as ThreadBuilder;

    ThreadBuilder::new()
        .name("Device Thread".to_string())
        .stack_size(1024 * 1024 * 8)
        .spawn(move || {
            let peripherals =
                PeripheralSet::new(gshim, ashim, pshim, timer_shim, clock_shim, ishim, oshim);

            // TODO: this shouldn't have to be `'static`!
            // let _flags: PeripheralInterruptFlags = PeripheralInterruptFlags::new();
            static _flags: PeripheralInterruptFlags = PeripheralInterruptFlags::new();

            let mut interp: Interpreter<'_, _, _> = InterpreterBuilder::new() //.build();
                .with_defaults()
                .with_peripherals(peripherals)
                .with_memory(memory)
                //.with_interrupt_flags_by_ref(&_flags)
                .build();

            interp.reset();
            interp.init(&_flags);

            let mut sim = Simulator::new_with_state(interp, &*EVENT_FUTURE_SHARED_STATE_SIM);
            // sim.get_interpreter().init(&_flags);
            sim.reset();

            loop {
                // (*counter).lock().unwrap().step(&mut sim);
                device.step(&mut sim);
                // thread::sleep(time::Duration::from_millis(10));
            }
        });

    // thread::sleep(time::Duration::from_millis(100000));

    let screen = AlternateScreen::to_alternate(true)?;
    let backend = CrosstermBackend::with_alternate_screen(screen)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let cli = Cli {
        tick_rate: 250,
        log: true,
    };

    if cli.log {
        Logger::with_env_or_str("")
            .log_to_file()
            .directory("log_files")
            .format(opt_format)
            .start()
            .unwrap();
    }

    let mut input_mode = TuiState::CONT;
    let mut pin_flag = 0;
    let mut gpio_pin = GpioPin::G0;
    let mut adc_pin = AdcPin::A0;
    let mut pwm_pin = PwmPin::P0;
    let mut timer_id = TimerId::T0;
    let mut reg_id = Reg::R0;
    let mut mem_addr: Addr = 1;

    let mut input_out = String::from("");
    let mut set_val_out = String::from("");

    let mut active: bool = true;
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

    let mut offset: u16 = 2;
    let mut sim = controller;
    let mut bp = HashMap::new();
    let mut wp = HashMap::new();

    while active {
        //let bp = sim.get_breakpoints();

        match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent::Insert => {
                    set_val_out = String::from("");
                    if input_mode == TuiState::IN {
                        input_mode = TuiState::CONT;
                    } else {
                        input_mode = TuiState::IN;
                    }
                }
                KeyEvent::Up => offset = offset.wrapping_add(1),
                KeyEvent::Down => offset = offset.wrapping_sub(1),
                KeyEvent::ShiftUp => offset = offset.wrapping_add(10),
                KeyEvent::ShiftDown => offset = offset.wrapping_sub(10),
                KeyEvent::CtrlUp => offset = offset.wrapping_add(100),
                KeyEvent::CtrlDown => offset = offset.wrapping_sub(100),
                KeyEvent::Char(c) => {
                    if input_mode == TuiState::CONT {
                        match c {
                            's' => {
                                sim.step();
                                offset = 2;
                            }
                            'p' => {
                                sim.pause();
                                offset = 2;
                            }
                            'r' => {
                                sim.run_until_event();
                                offset = 2;
                            }
                            'b' => {
                                let cur_addr = sim.get_pc().wrapping_sub(offset).wrapping_add(2);
                                match bp.remove(&cur_addr) {
                                    Some(val) => {sim.unset_breakpoint(val);},
                                    None => {match sim.set_breakpoint(cur_addr) {
                                        Ok(val) => {bp.insert(cur_addr, val);},
                                        Err(_e) => {},
                                    }},
                                };
                                offset = sim.get_pc().wrapping_sub(cur_addr - 2);
                            }
                            'w' => {
                                let cur_addr = sim.get_pc().wrapping_sub(offset).wrapping_add(2);
                                match wp.remove(&cur_addr) {
                                    Some(val) => {sim.unset_memory_watchpoint(val);},
                                    None => {match sim.set_memory_watchpoint(cur_addr) {
                                        Ok(val) => {wp.insert(cur_addr, val);},
                                        Err(_e) => {},
                                    }},
                                };
                            }
                            'q' => active = false,
                            _ => {}
                        }
                    } else if input_mode == TuiState::IN {
                        source_shim.push(c);
                        match c {
                            '\n' => {
                                input_out = String::from("");
                            }
                            _ => {
                                let x = format!("{}", c);
                                input_out.push_str(&x);
                            }
                        }
                    } else if input_mode == TuiState::GPIO {
                        if pin_flag == 0 {
                            pin_flag = 1;
                            match c {
                                '0' => gpio_pin = GpioPin::G0,
                                '1' => gpio_pin = GpioPin::G1,
                                '2' => gpio_pin = GpioPin::G2,
                                '3' => gpio_pin = GpioPin::G3,
                                '4' => gpio_pin = GpioPin::G4,
                                '5' => gpio_pin = GpioPin::G5,
                                '6' => gpio_pin = GpioPin::G6,
                                '7' => gpio_pin = GpioPin::G7,
                                _ => pin_flag = 0,
                            }
                        } else {
                            match c {
                                '\n' => {
                                    match set_val_out.parse::<bool>() {
                                        Ok(b) => RwLock::write(&gpio_shim)
                                            .unwrap()
                                            .set_pin(gpio_pin, b)
                                            .unwrap(),
                                        Err(_e) => {}
                                    }
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        }
                    } else if input_mode == TuiState::ADC {
                        if pin_flag == 0 {
                            pin_flag = 1;
                            match c {
                                '0' => adc_pin = AdcPin::A0,
                                '1' => adc_pin = AdcPin::A1,
                                '2' => adc_pin = AdcPin::A2,
                                '3' => adc_pin = AdcPin::A3,
                                '4' => adc_pin = AdcPin::A4,
                                '5' => adc_pin = AdcPin::A5,
                                _ => pin_flag = 0,
                            }
                        } else {
                            match c {
                                '\n' => {
                                    match set_val_out.parse::<u8>() {
                                        Ok(n) => RwLock::write(&adc_shim)
                                            .unwrap()
                                            .set_value(adc_pin, n)
                                            .unwrap(),
                                        Err(_e) => {}
                                    }
                                    if set_val_out.len() > 2 {
                                        let val = set_val_out.split_off(2);
                                        if set_val_out == "0x" {
                                            match u8::from_str_radix(&val, 16) {
                                                Ok(n) => RwLock::write(&adc_shim)
                                                    .unwrap()
                                                    .set_value(adc_pin, n)
                                                    .unwrap(),
                                                Err(_e) => {}
                                            }
                                        } else if set_val_out == "0b" {
                                            match u8::from_str_radix(&val, 2) {
                                                Ok(n) => RwLock::write(&adc_shim)
                                                    .unwrap()
                                                    .set_value(adc_pin, n)
                                                    .unwrap(),
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        }
                    } else if input_mode == TuiState::PWM {
                        if pin_flag == 0 {
                            pin_flag = 1;
                            match c {
                                '0' => pwm_pin = PwmPin::P0,
                                '1' => pwm_pin = PwmPin::P1,
                                _ => pin_flag = 0,
                            }
                        } else {
                            match c {
                                '\n' => {
                                    match set_val_out.parse::<NonZeroU8>() {
                                        Ok(n) => RwLock::write(&pwm_shim)
                                            .unwrap()
                                            .set_duty_cycle_helper(pwm_pin, n),
                                        Err(_e) => {}
                                    }
                                    if set_val_out.len() > 2 {
                                        let val = set_val_out.split_off(2);
                                        if set_val_out == "0x" {
                                            match u8::from_str_radix(&val, 16) {
                                                Ok(n) => {
                                                    if n > 0 {
                                                        RwLock::write(&pwm_shim)
                                                            .unwrap()
                                                            .set_duty_cycle_helper(
                                                                pwm_pin,
                                                                NonZeroU8::new(n).unwrap(),
                                                            );
                                                    }
                                                }
                                                Err(_e) => {}
                                            }
                                        } else if set_val_out == "0b" {
                                            match u8::from_str_radix(&val, 2) {
                                                Ok(n) => {
                                                    if n > 0 {
                                                        RwLock::write(&pwm_shim)
                                                            .unwrap()
                                                            .set_duty_cycle_helper(
                                                                pwm_pin,
                                                                NonZeroU8::new(n).unwrap(),
                                                            );
                                                    }
                                                }
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        }
                    } else if input_mode == TuiState::TMR {
                        if pin_flag == 0 {
                            pin_flag = 1;
                            match c {
                                '0' => timer_id = TimerId::T0,
                                '1' => timer_id = TimerId::T1,
                                _ => pin_flag = 0,
                            }
                        } else {
                            match c {
                                '\n' => {
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        }
                    } else if input_mode == TuiState::REG {
                        if pin_flag == 0 {
                            pin_flag = 1;
                            match c {
                                '0' => reg_id = Reg::R0,
                                '1' => reg_id = Reg::R1,
                                '2' => reg_id = Reg::R2,
                                '3' => reg_id = Reg::R3,
                                '4' => reg_id = Reg::R4,
                                '5' => reg_id = Reg::R5,
                                '6' => reg_id = Reg::R6,
                                '7' => reg_id = Reg::R7,
                                _ => pin_flag = 0,
                            }
                        } else {
                            match c {
                                '\n' => {
                                    match set_val_out.parse::<Word>() {
                                        Ok(w) => sim.set_register(reg_id, w),
                                        Err(_e) => {}
                                    }
                                    if set_val_out.len() > 2 {
                                        let val = set_val_out.split_off(2);
                                        if set_val_out == "0x" {
                                            match Word::from_str_radix(&val, 16) {
                                                Ok(w) => sim.set_register(reg_id, w),
                                                Err(_e) => {}
                                            }
                                        } else if set_val_out == "0b" {
                                            match Word::from_str_radix(&val, 2) {
                                                Ok(w) => sim.set_register(reg_id, w),
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        }
                    } else if input_mode == TuiState::MEM {
                        if pin_flag == 0 {
                            match c {
                                '\n' => {
                                    match set_val_out.parse::<Addr>() {
                                        Ok(a) => {
                                            pin_flag = 1;
                                            mem_addr = a;
                                        }
                                        Err(_e) => {}
                                    }
                                    if set_val_out.len() > 2 {
                                        let val = set_val_out.split_off(2);
                                        if set_val_out == "0x" {
                                            match Addr::from_str_radix(&val, 16) {
                                                Ok(a) => {
                                                    pin_flag = 1;
                                                    mem_addr = a;
                                                }
                                                Err(_e) => {}
                                            }
                                        } else if set_val_out == "0b" {
                                            match Addr::from_str_radix(&val, 2) {
                                                Ok(a) => {
                                                    pin_flag = 1;
                                                    mem_addr = a;
                                                }
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        } else {
                            match c {
                                '\n' => {
                                    if set_val_out == "b" {
                                        match sim.set_breakpoint(mem_addr) {
                                            Ok(val) => {bp.insert(mem_addr, val); pin_flag = 0;},
                                            Err(_e) => {},
                                        }
                                    } else if set_val_out == "w" {
                                        match sim.set_memory_watchpoint(mem_addr) {
                                            Ok(val) => {wp.insert(mem_addr, val); pin_flag = 0;},
                                            Err(_e) => {},
                                        }
                                        
                                    } else if set_val_out == "rb" {
                                        match bp.remove(&mem_addr) {
                                            Some(val) => {sim.unset_breakpoint(val); pin_flag = 0;},
                                            None => {},
                                        };
                                    } else if set_val_out == "rw" {
                                        match wp.remove(&mem_addr) {
                                            Some(val) =>  {sim.unset_memory_watchpoint(val); pin_flag = 0;},
                                            None => {},
                                        };
                                    } else if set_val_out == "j" {
                                        offset = sim.get_pc().wrapping_sub(mem_addr - 2);
                                        pin_flag = 0;
                                    } else {
                                        match set_val_out.parse::<Word>() {
                                            Ok(w) => {
                                                sim.write_word(mem_addr, w);
                                                pin_flag = 0;
                                            }
                                            Err(_e) => {}
                                        }
                                        if set_val_out.len() > 2 {
                                            let val = set_val_out.split_off(2);
                                            if set_val_out == "0x" {
                                                match Word::from_str_radix(&val, 16) {
                                                    Ok(w) => {
                                                        sim.write_word(mem_addr, w);
                                                        pin_flag = 0;
                                                    }
                                                    Err(_e) => {}
                                                }
                                            } else if set_val_out == "0b" {
                                                match Word::from_str_radix(&val, 2) {
                                                    Ok(w) => {
                                                        sim.write_word(mem_addr, w);
                                                        pin_flag = 0;
                                                    }
                                                    Err(_e) => {}
                                                }
                                            }
                                        }
                                    }
                                    set_val_out = String::from("");
                                }
                                _ => {
                                    let x = format!("{}", c);
                                    set_val_out.push_str(&x);
                                }
                            }
                        }
                    } else if input_mode == TuiState::CLK {
                        match c {
                            '\n' => {
                                set_val_out = String::from("");
                            }
                            _ => {
                                let x = format!("{}", c);
                                set_val_out.push_str(&x);
                            }
                        }
                    } else if input_mode == TuiState::PC {
                        match c {
                            '\n' => {
                                match set_val_out.parse::<Addr>() {
                                    Ok(a) => sim.set_pc(a),
                                    Err(_e) => {}
                                }
                                if set_val_out.len() > 2 {
                                    let val = set_val_out.split_off(2);
                                    if set_val_out == "0x" {
                                        match Addr::from_str_radix(&val, 16) {
                                            Ok(a) => sim.set_pc(a),
                                            Err(_e) => {}
                                        }
                                    } else if set_val_out == "0b" {
                                        match Addr::from_str_radix(&val, 2) {
                                            Ok(a) => sim.set_pc(a),
                                            Err(_e) => {}
                                        }
                                    }
                                }
                                set_val_out = String::from("");
                            }
                            _ => {
                                let x = format!("{}", c);
                                set_val_out.push_str(&x);
                            }
                        }
                    }
                }
                KeyEvent::Ctrl(c) => {
                    set_val_out = String::from("");
                    match c {
                        'g' => {
                            if input_mode == TuiState::GPIO {
                                input_mode = TuiState::CONT;
                            } else {
                                pin_flag = 0;
                                input_mode = TuiState::GPIO;
                            }
                        }
                        'a' => {
                            if input_mode == TuiState::ADC {
                                input_mode = TuiState::CONT;
                            } else {
                                pin_flag = 0;
                                input_mode = TuiState::ADC;
                            }
                        }
                        'p' => {
                            if input_mode == TuiState::PWM {
                                input_mode = TuiState::CONT;
                            } else {
                                pin_flag = 0;
                                input_mode = TuiState::PWM;
                            }
                        }
                        't' => {
                            if input_mode == TuiState::TMR {
                                input_mode = TuiState::CONT;
                            } else {
                                pin_flag = 0;
                                input_mode = TuiState::TMR;
                            }
                        }
                        'c' => {
                            if input_mode == TuiState::CLK {
                                input_mode = TuiState::CONT;
                            } else {
                                pin_flag = 1;
                                input_mode = TuiState::CLK;
                            }
                        }
                        'r' => {
                            if input_mode == TuiState::REG {
                                input_mode = TuiState::CONT;
                            } else {
                                pin_flag = 0;
                                input_mode = TuiState::REG;
                            }
                        }
                        'h' => offset = 2,
                        _ => {}
                    }
                }
                KeyEvent::Alt(c) => match c {
                    'r' => {
                        pin_flag = 0;
                        input_mode = TuiState::CONT;
                        set_val_out = String::from("");
                        input_out = String::from("");
                        sim.reset();
                    }
                    'm' => {
                        if input_mode == TuiState::MEM {
                            input_mode = TuiState::CONT;
                        } else {
                            pin_flag = 0;
                            input_mode = TuiState::MEM;
                        }
                    }
                    'p' => {
                        if input_mode == TuiState::PC {
                            input_mode = TuiState::CONT;
                        } else {
                            pin_flag = 1;
                            input_mode = TuiState::PC;
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::Tick => {}
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(10), Constraint::Length(8)].as_ref())
                .split(f.size());

            let footer = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(20),
                        Constraint::Length(80),
                    ]
                    .as_ref(),
                )
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

            let console_watch = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(66), Constraint::Percentage(34)].as_ref())
                .split(right_pane[0]);

            let console = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
                .split(console_watch[0]);

            Block::default()
                .title("> ")
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .render(&mut f, console[1]);

            Block::default()
                .title("IO")
                .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))
                .borders(Borders::ALL)
                .render(&mut f, right_pane[1]);

            //Further breakdown of IO
            let io_pane = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(5),
                        Constraint::Length(3),
                        Constraint::Length(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(right_pane[1]);

            let timers_n_clock = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
                .split(io_pane[3]);

            //TEXT BELOW HERE

            //Footer Text
            let text = [
                Text::styled(
                    "To control the TUI, you can use ",
                    Style::default().fg(Color::LightGreen),
                ),
                Text::styled("S to Step ", Style::default().fg(Color::LightCyan)),
                Text::styled(
                    "through instructions, ",
                    Style::default().fg(Color::LightGreen),
                ),
                Text::styled("P to Pause, ", Style::default().fg(Color::LightRed)),
                Text::styled("R to Run, ", Style::default().fg(Color::LightYellow)),
                Text::styled("and ", Style::default().fg(Color::LightGreen)),
                Text::styled("Q to Quit\n", Style::default().fg(Color::Gray)),
                Text::styled(
                    "To set the peripherals use ",
                    Style::default().fg(Color::LightGreen),
                ),
                Text::styled("Ctrl + ", Style::default().fg(Color::White)),
                Text::styled(
                    "g for GPIO, ",
                    Style::default().fg(Color::Rgb(0xee, 0xee, 0xee)),
                ),
                Text::styled(
                    "a for ADC, ",
                    Style::default().fg(Color::Rgb(0xdd, 0xdd, 0xdd)),
                ),
                Text::styled(
                    "p for PWM, ",
                    Style::default().fg(Color::Rgb(0xcc, 0xcc, 0xcc)),
                ),
                Text::styled(
                    "t for Timers, ",
                    Style::default().fg(Color::Rgb(0xbb, 0xbb, 0xbb)),
                ),
                Text::styled("and ", Style::default().fg(Color::LightGreen)),
                Text::styled(
                    "c for CLK\n",
                    Style::default().fg(Color::Rgb(0xaa, 0xaa, 0xaa)),
                ),
                Text::styled(
                    "To affect the simulator, use ",
                    Style::default().fg(Color::LightGreen),
                ),
                Text::styled("Alt + ", Style::default().fg(Color::White)),
                Text::styled(
                    "p for PC, ",
                    Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
                ),
                Text::styled("m for Memory, ", Style::default().fg(Color::LightCyan)),
                Text::styled("and ", Style::default().fg(Color::LightGreen)),
                Text::styled("r to reset\n", Style::default().fg(Color::Gray)),
                Text::styled(
                    "To control memory, use ",
                    Style::default().fg(Color::LightGreen),
                ),
                Text::styled("UP and DOWN ", Style::default().fg(Color::Gray)),
                Text::styled("arrow keys. ", Style::default().fg(Color::LightGreen)),
                Text::styled("Shift + arrow ", Style::default().fg(Color::Gray)),
                Text::styled("jumps 10, ", Style::default().fg(Color::LightGreen)),
                Text::styled("Control + arrow ", Style::default().fg(Color::Gray)),
                Text::styled("jumps 100. ", Style::default().fg(Color::LightGreen)),
                Text::styled(
                    "Ctrl + h returns to PC\n",
                    Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
                ),
            ];

            Paragraph::new(text.iter())
                .block(
                    Block::default().borders(Borders::ALL)
                    .title("Footer")
                    .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, footer[0]);

            //Shim Input

            let mut cur_pin = Text::styled("\n", Style::default());
            if input_mode == TuiState::MEM {
                if pin_flag == 0 {
                    cur_pin = Text::styled(
                        "INPUT ADDRESS\n",
                        Style::default().fg(Color::Red).modifier(Modifier::BOLD),
                    );
                } else {
                    cur_pin = Text::styled(
                        format!("Current Addr: {:#06x}\n", mem_addr),
                        Style::default().fg(Color::Gray),
                    );
                }
            } else if input_mode != TuiState::CONT && input_mode != TuiState::IN {
                if pin_flag == 0 {
                    cur_pin = Text::styled(
                        "SELECT TARGET (Type an integer)\n",
                        Style::default().fg(Color::Red).modifier(Modifier::BOLD),
                    );
                } else {
                    cur_pin = Text::styled(
                        format!(
                            "Current Selection: {}\n",
                            get_pin_string(
                                input_mode, gpio_pin, adc_pin, pwm_pin, timer_id, reg_id
                            )
                        ),
                        Style::default().fg(Color::Gray),
                    );
                }
            };

            let info = sim.get_info();
            let mut proxies = String::new();

            for p in info.proxies.iter().filter_map(|p| *p) {
                proxies += format!("-> {}", p).as_str();
            }

            let text = [
                Text::styled(
                    format!("Input Mode: {}\n", input_mode_string(input_mode)),
                    Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
                ),
                cur_pin,
                Text::raw(set_val_out.clone()),
            ];

            let status = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(30),
                        Constraint::Length(50),
                    ]
                    .as_ref(),
                )
                .split(footer[1]);

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, status[0]);

            //Metadata

            let text = [
                Text::styled(
                    format!(
                        "\n\n\nProg: {:?}\nSource: {} | Proxies: {}",
                        info.current_program_metadata, info.source_name, proxies,
                    ),
                    Style::default().fg(Color::Rgb(0xA6, 0x97, 0xB7)),
                )
            ];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::ALL)
                    .title("Status Window")
                    .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))))
                .wrap(true)
                .render(&mut f, footer[1]);

            //Register Status Text
            let regs_psr_pc = sim.get_registers_psr_and_pc();
            let (regs, psr, pc) = regs_psr_pc;

            let text = [
                Text::styled("R0:\nR1:\nR2:\nR3:\n", Style::default().fg(Color::Gray)),
                Text::styled("PSR:\n", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
            ];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Registers + PC + PSR")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, left_pane[1]);

            let regs_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(5),
                        Constraint::Length(40),
                        Constraint::Length(5),
                        Constraint::Length(40),
                    ]
                    .as_ref(),
                )
                .split(left_pane[1]);

            let mut s = String::from("");
            for i in 0..4 {
                s.push_str(&format!(
                    "{:#018b} {:#06x} {:#05}\n",
                    regs[i], regs[i], regs[i]
                ));
            }
            s.push_str(&format!("{:#018b} {:#06x} {:#05}\n", psr, psr, psr));

            let text = [Text::styled(s, Style::default().fg(Color::LightGreen))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, regs_partitions[1]);

            let text = [
                Text::styled("R4:\nR5:\nR6:\nR7:\n", Style::default().fg(Color::Gray)),
                Text::styled("PC:\n", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
            ];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, regs_partitions[2]);

            s = String::from("");
            for i in 4..8 {
                s.push_str(&format!(
                    "{:#018b} {:#06x} {:#05}\n",
                    regs[i], regs[i], regs[i]
                ));
            }
            s.push_str(&format!("{:#018b} {:#06x} {:#05}\n", pc, pc, pc));

            let text = [Text::styled(s, Style::default().fg(Color::LightGreen))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, regs_partitions[3]);

            //Memory
            let mut mem: [Word; 50] = [0; 50];
            let mut x: u16 = 0;
            while x != 50 {
                mem[x as usize] = sim.read_word(pc.wrapping_sub(offset).wrapping_add(x));
                x = x + 1;
            }

            let mut pc_arrow = String::from("");
            let mut bp_locs = String::from("");
            let mut wp_locs = String::from("");
            let mut addresses = String::from("");
            s = String::from("");
            let mut insts = String::from("");
            x = 0;
            while x != 50 {
                let inst: Instruction = match mem[x as usize].try_into() {
                    Ok(data) => data,
                    Err(_e) => Instruction::AddReg {
                        dr: Reg::R0,
                        sr1: Reg::R0,
                        sr2: Reg::R0,
                    },
                };

                let addr = pc.wrapping_sub(offset).wrapping_add(x);
                if x == offset {
                    pc_arrow.push_str("-->\n");
                } else {
                    pc_arrow.push_str("\n");
                }

                if bp.contains_key(&addr) {
                    bp_locs.push_str("<b>\n");
                } else {
                    bp_locs.push_str("\n");
                }

                if wp.contains_key(&addr) {
                    wp_locs.push_str("<w>\n");
                } else {
                    wp_locs.push_str("\n");
                }

                addresses.push_str(&format!(
                    "{:#06x}\n",
                    addr
                ));
                s.push_str(&format!(
                    "{:#018b} {:#06x} {:#05}\n",
                    mem[x as usize], mem[x as usize], mem[x as usize]
                ));
                insts.push_str(&format!("{}\n", inst));
                x = x + 1;
            }

            let text = [Text::styled(
                "\n\n-->",
                Style::default().fg(Color::Rgb(0x73, 0xB7, 0xE8)),
            )];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                )
                .wrap(true)
                .render(&mut f, left_pane[0]);

            let text = [Text::styled(
                pc_arrow,
                Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
            )];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Memory")
                        .title_style(
                            Style::default()
                                .fg(Color::Rgb(0xFF, 0x97, 0x40))
                        ),
                )
                .wrap(true)
                .render(&mut f, left_pane[0]);

            let mem_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(5),
                        Constraint::Length(5),
                        Constraint::Length(5),
                        Constraint::Length(10),
                        Constraint::Length(40),
                        Constraint::Min(10),
                    ]
                    .as_ref(),
                )
                .split(left_pane[0]);

            let text = [Text::styled(
                bp_locs,
                Style::default().fg(Color::Rgb(0xCC, 0x02, 0x02)),
            )];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[1]);

            let text = [Text::styled(
                wp_locs,
                Style::default().fg(Color::Rgb(0x30, 0x49, 0xDE)),
            )];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[2]);

            let text = [Text::styled(addresses, Style::default().fg(Color::Gray))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[3]);

            let text = [Text::styled(s, Style::default().fg(Color::LightGreen))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[4]);

            let text = [Text::styled(insts, Style::default().fg(Color::LightCyan))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[5]);

            //Console

            let console_height = console[0].height;
            let output_string = iteratively_collect_into_console_output();
            let num_lines = output_string.split('\n').count();

            // output_string.split('\n').map(|s| Text::raw(s))

            // //Console
            // let text = [
            //     Text::raw(iteratively_collect_into_console_output())
            // ];

            // let text: Vec<_> = output_string.split('\n').map(|s| Text::raw(s)).collect();

            Paragraph::new(
                [Text::styled(
                    output_string,
                    Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
                )]
                .iter(),
            )
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                    .title("Console")
                    .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
            )
            .wrap(false)
            .scroll((num_lines.saturating_sub(console_height as usize)) as u16)
            .render(&mut f, console[0]);

            let text = [Text::raw(input_out.clone())];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                        .title(">")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, console[1]);

            //IO

            //GPIO
            let GPIO_states = sim.get_gpio_states();
            let gpioin = sim.get_gpio_readings();

            let text = [Text::styled(
                "GPIO 0:\nGPIO 1:\nGPIO 2:\nGPIO 3:\n",
                Style::default().fg(Color::Gray),
            )];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                        .title("GPIO")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, io_pane[0]);

            let left_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(io_pane[0]);

            let gpio = match gpioin[GpioPin::G0] {
                Ok(val) => format!("{}\n", val),
                Err(_e) => format!("-\n"),
            };

            let t0 = match GPIO_states[GpioPin::G0] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let gpio = match gpioin[GpioPin::G1] {
                Ok(val) => format!("{}\n", val),
                Err(_e) => format!("-\n"),
            };

            let t1 = match GPIO_states[GpioPin::G1] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let gpio = match gpioin[GpioPin::G2] {
                Ok(val) => format!("{}\n", val),
                Err(_e) => format!("-\n"),
            };

            let t2 = match GPIO_states[GpioPin::G2] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let gpio = match gpioin[GpioPin::G3] {
                Ok(val) => format!("{}\n", val),
                Err(_e) => format!("-\n"),
            };

            let t3 = match GPIO_states[GpioPin::G3] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let text = [t0, t1, t2, t3];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, left_partitions[1]);

            let gpio_half = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(io_pane[0]);

            let right_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(gpio_half[1]);

            let text = [Text::styled(
                "GPIO 4:\nGPIO 5:\nGPIO 6:\nGPIO 7:\n",
                Style::default().fg(Color::Gray),
            )];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, right_partitions[0]);

            let gpio = match gpioin[GpioPin::G4] {
                Ok(val) => format!("GPIO 4:  {}\n", val),
                Err(_e) => format!("GPIO 4:  -\n"),
            };

            let t0 = match GPIO_states[GpioPin::G4] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let gpio = match gpioin[GpioPin::G5] {
                Ok(val) => format!("GPIO 5:  {}\n", val),
                Err(_e) => format!("GPIO 5:  -\n"),
            };

            let t1 = match GPIO_states[GpioPin::G5] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let gpio = match gpioin[GpioPin::G6] {
                Ok(val) => format!("GPIO 6:  {}\n", val),
                Err(_e) => format!("GPIO 6:  -\n"),
            };

            let t2 = match GPIO_states[GpioPin::G6] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let gpio = match gpioin[GpioPin::G7] {
                Ok(val) => format!("GPIO 7:  {}\n", val),
                Err(_e) => format!("GPIO 7:  -\n"),
            };

            let t3 = match GPIO_states[GpioPin::G7] {
                GpioState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(gpio, Style::default().fg(Color::LightGreen)),
            };

            let text = [t0, t1, t2, t3];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP | Borders::RIGHT))
                .wrap(true)
                .render(&mut f, right_partitions[1]);

            //ADC
            let ADC_states = sim.get_adc_states();
            let adcin = sim.get_adc_readings();

            let text = [Text::styled(
                "ADC 0:\nADC 1:\n",
                Style::default().fg(Color::Gray),
            )];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                        .title("ADC")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, io_pane[1]);

            let left_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(io_pane[1]);

            let adc = match adcin[AdcPin::A0] {
                Ok(number) => format!("{:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(_e) => format!("-                  -      -\n"),
            };

            let t0 = match ADC_states[AdcPin::A0] {
                AdcState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(adc, Style::default().fg(Color::LightGreen)),
            };

            let adc = match adcin[AdcPin::A1] {
                Ok(number) => format!("{:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(_e) => format!("-                  -      -\n"),
            };

            let t1 = match ADC_states[AdcPin::A1] {
                AdcState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(adc, Style::default().fg(Color::LightGreen)),
            };

            let adc = match adcin[AdcPin::A2] {
                Ok(number) => format!("{:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(_e) => format!("-                  -      -\n"),
            };

            let t2 = match ADC_states[AdcPin::A2] {
                AdcState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(adc, Style::default().fg(Color::LightGreen)),
            };

            let text = [t0, t1, t2];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, left_partitions[1]);

            let right_ADC = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(io_pane[1]);

            let right_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(right_ADC[1]);

            let text = [Text::styled(
                "ADC 2:\nADC 3:\n",
                Style::default().fg(Color::Gray),
            )];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, right_ADC[1]);

            let adc = match adcin[AdcPin::A3] {
                Ok(number) => format!("{:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(_e) => format!("-                  -      -\n"),
            };

            let t0 = match ADC_states[AdcPin::A3] {
                AdcState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(adc, Style::default().fg(Color::LightGreen)),
            };

            let adc = match adcin[AdcPin::A4] {
                Ok(number) => format!("{:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(_e) => format!("-                  -      -\n"),
            };

            let t1 = match ADC_states[AdcPin::A4] {
                AdcState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(adc, Style::default().fg(Color::LightGreen)),
            };

            let adc = match adcin[AdcPin::A5] {
                Ok(number) => format!("{:#018b} {:#06x} {:#05}\n", number, number, number),
                Err(_e) => format!("-                  -      -\n"),
            };

            let t2 = match ADC_states[AdcPin::A5] {
                AdcState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                _ => Text::styled(adc, Style::default().fg(Color::LightGreen)),
            };

            let text = [t0, t1, t2];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, right_partitions[1]);

            //PWM
            let PWM_states = sim.get_pwm_states();
            let PWM = sim.get_pwm_config();

            let text = [Text::styled("PWM 0:\n", Style::default().fg(Color::Gray))];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                        .title("PWM")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, io_pane[2]);

            let left_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(io_pane[2]);

            let text = match PWM_states[PwmPin::P0] {
                PwmState::Disabled => [Text::styled(
                    format!("Disabled"),
                    Style::default().fg(Color::LightRed),
                )],
                PwmState::Enabled(_) => [Text::styled(
                    format!(
                        "{:#018b} {:#06x} {:#05}\n",
                        PWM[PwmPin::P0],
                        PWM[PwmPin::P0],
                        PWM[PwmPin::P0]
                    ),
                    Style::default().fg(Color::LightGreen),
                )],
            };

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, left_partitions[1]);

            let right_PWM = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(io_pane[2]);

            let right_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(right_PWM[1]);

            let text = [Text::styled("PWM 1:\n", Style::default().fg(Color::Gray))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, right_partitions[0]);

            let text = match PWM_states[PwmPin::P1] {
                PwmState::Disabled => [Text::styled(
                    format!("Disabled"),
                    Style::default().fg(Color::LightRed),
                )],
                PwmState::Enabled(_) => [Text::styled(
                    format!(
                        "{:#018b} {:#06x} {:#05}\n",
                        PWM[PwmPin::P0],
                        PWM[PwmPin::P0],
                        PWM[PwmPin::P0]
                    ),
                    Style::default().fg(Color::LightGreen),
                )],
            };

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP | Borders::RIGHT))
                .wrap(true)
                .render(&mut f, right_partitions[1]);

            //Timers
            let timer_state = sim.get_timer_states();
            let timer = sim.get_timer_config();

            let text = [Text::styled(
                "Timer 1:\nTimer 2:\n",
                Style::default().fg(Color::Gray),
            )];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL & (!Borders::RIGHT))
                        .title("Timers")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, timers_n_clock[0]);

            let left_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(timers_n_clock[0]);

            let t0 = match timer_state[TimerId::T0] {
                TimerState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                TimerState::Repeated => Text::styled(
                    format!(
                        "Repeat:  {:#018b} {:#06x} {:#05}\n",
                        timer[TimerId::T0],
                        timer[TimerId::T0],
                        timer[TimerId::T0]
                    ),
                    Style::default().fg(Color::LightGreen),
                ),
                TimerState::SingleShot => Text::styled(
                    format!(
                        "Single:  {:#018b} {:#06x} {:#05}\n",
                        timer[TimerId::T0],
                        timer[TimerId::T0],
                        timer[TimerId::T0]
                    ),
                    Style::default().fg(Color::LightGreen),
                ),
            };

            let t1 = match timer_state[TimerId::T1] {
                TimerState::Disabled => {
                    Text::styled(format!("Disabled\n"), Style::default().fg(Color::LightRed))
                }
                TimerState::Repeated => Text::styled(
                    format!(
                        "Repeat:  {:#018b} {:#06x} {:#05}\n",
                        timer[TimerId::T1],
                        timer[TimerId::T1],
                        timer[TimerId::T1]
                    ),
                    Style::default().fg(Color::LightGreen),
                ),
                TimerState::SingleShot => Text::styled(
                    format!(
                        "Single:  {:#018b} {:#06x} {:#05}\n",
                        timer[TimerId::T1],
                        timer[TimerId::T1],
                        timer[TimerId::T1]
                    ),
                    Style::default().fg(Color::LightGreen),
                ),
            };

            let text = [t0, t1];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .wrap(true)
                .render(&mut f, left_partitions[1]);

            //Clock
            let clock = sim.get_clock();

            let text = [Text::raw(format!(
                "{:#018b} {:#06x} {:#05}\n",
                clock, clock, clock
            ))];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Clock")
                        .title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
                )
                .wrap(true)
                .render(&mut f, timers_n_clock[1]);

            //Watchpoints

            let mut wp_indices = String::from("");
            addresses = String::from("");
            s = String::from("");

            for (wp_addr, index) in wp.iter() {
                wp_indices.push_str(&format!(
                    "{}\n\n",
                    index
                ));

                addresses.push_str(&format!(
                    "{:#06x}\n\n",
                    wp_addr
                ));

                s.push_str(&format!(
                    "{:#018b} {:#06x} {:#05}\n",
                    sim.read_word(*wp_addr), sim.read_word(*wp_addr), sim.read_word(*wp_addr)
                ));
            }

            let text = [Text::styled(
                wp_indices,
                Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
            )];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Watch Window")
                        .title_style(
                            Style::default()
                                .fg(Color::Rgb(0xFF, 0x97, 0x40))
                        ),
                )
                .wrap(true)
                .render(&mut f, console_watch[1]);

            let mem_partitions = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(8),
                        Constraint::Length(40),
                        Constraint::Min(10),
                    ]
                    .as_ref(),
                )
                .split(console_watch[1]);

            let text = [Text::styled(addresses, Style::default().fg(Color::Gray))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[1]);

            let text = [Text::styled(s, Style::default().fg(Color::LightGreen))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, mem_partitions[2]);

            //Breakpoints
            let mut bp_title = String::from("Breakpoints:");
            bp_title.push_str(&format!(
                    "{}/{} ", bp.len(), MAX_BREAKPOINTS));

            let mut wp_title = String::from("");
            wp_title.push_str(&format!("    Watchpoints: {}/{}\n",wp.len(), MAX_MEMORY_WATCHPOINTS));

            s = String::from("");

            let mut i = 0;

            for (bp_addr, index) in bp.iter() {
                if i == 5 {
                    s.push_str(",\n");
                    i = i + 1;
                } else if i > 0 {
                    s.push_str(",");
                    i = i + 1;
                } else {
                    i = i + 1;
                }

                s.push_str(&format!(
                    " {:#06x}",
                    bp_addr
                ));

            }

            let text = [Text::styled(bp_title, Style::default().fg(Color::Rgb(0xCC, 0x02, 0x02))),
                Text::styled(wp_title, Style::default().fg(Color::Rgb(0x30, 0x49, 0xDE))),
                Text::styled(s, Style::default().fg(Color::Rgb(0xCC, 0x02, 0x02)))];

            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::NONE))
                .wrap(true)
                .render(&mut f, status[1]);


        })?;
        //  loop{}

       


    }

    Ok(())
}

fn input_mode_string(s: TuiState) -> String {
    use TuiState::*;

    match s {
        CONT => format!("Control"),
        IN => format!("Input"),
        GPIO => format!("GPIO"),
        ADC => format!("ADC"),
        PWM => format!("PWM"),
        TMR => format!("Timer"),
        CLK => format!("Clock"),
        REG => format!("Registers"),
        PC => format!("Program Counter (PC)"),
        MEM => format!("Memory"),
    }
}

fn get_pin_string(s: TuiState, g: GpioPin, a: AdcPin, p: PwmPin, t: TimerId, r: Reg) -> String {
    use TuiState::*;

    match s {
        GPIO => g.to_string(),
        ADC => a.to_string(),
        PWM => p.to_string(),
        TMR => t.to_string(),
        REG => r.to_string(),
        CLK => format!("CLK"),
        PC => format!("PC"),
        _ => return format!(""),
    }
}
