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

use tui::widgets::{Widget, Block, Borders, Text, Paragraph};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Modifier, Style};
use tui::backend::{Backend};

use std::io::stdout;

use lc3_isa::{Addr, Word, Instruction,  Reg};
use lc3_traits::control::Control;

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
    let screen = AlternateScreen::to_alternate(true)?;
    let backend = CrosstermBackend::with_alternate_screen(screen)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let cli = Cli{
        tick_rate: 250,
        log: true,
    };

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

    let mut z = 0;
    loop {
        //println!("Console out: {}", z);
        z = z + 1;

        let x = terminal.get_cursor();
        let x = match x {
            Ok(data) => data,
            Err(error) => (0,0),
        };

        /*match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent::Char(c) => {
                    match c{
                        's' => Control::step(),
                        'p' => Control::pause(),
                        'r' => Control::run_until_event(),
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::Tick => {
                println!("z");
            }
        }*/


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
                .constraints([Constraint::Min(13), Constraint::Length(12)].as_ref())
                .split(panes[1]);

            let console = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
                .split(right_pane[0]);

            Block::default()
                 .title("Output")
                 .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                 .render(&mut f, console[0]);

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
                .constraints([Constraint::Length(3), Constraint::Length(2), Constraint::Length(2), Constraint::Length(2)].as_ref())
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
            //let regs = Control::get_registers_and_pc();
            let regsPC: ([Word; 9], Word) = ([1,2,3,4,5,6,7,8,9], 10);
            let (regs, pc) = regsPC;

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
                Text::raw(format!("PSR: {:#018b} {:#06x} {:#05}\n", regs[8], regs[8], regs[8])),
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
                //mem[x as usize] = Control::read_word(pc-2+x);
                mem[x as usize] = 0xA000 + (x+1) * 2;
                x = x + 1;
            }

            
            let mut s =  String::from("");
            x = 0;
            while x != 50 {
                let inst: Instruction = match mem[x as usize].try_into(){
                    Ok(data) => data,
                    Err(error) => Instruction::AddReg{dr: Reg::R0, sr1: Reg::R0, sr2: Reg::R0,},
                };
                s.push_str(&format!("{:#06x} {:#018b} {:#06x} {:#05}    {}\n", pc-2+x, mem[x as usize], mem[x as usize], mem[x as usize], inst));
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
            
            //IO

            //GPIO
            let GPIO: [Word; 4] = [100; 4];

            let text = [
                Text::raw(format!("GPIO 0:  {:#018b} {:#06x} {:#05}\n", GPIO[0], GPIO[0], GPIO[0])),
                Text::raw(format!("GPIO 1:  {:#018b} {:#06x} {:#05}\n", GPIO[1], GPIO[1], GPIO[1]))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                            .title("GPIO")
                            .title_style(Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
                )
                .wrap(true)
                .render(&mut f, io_panel[0]);

            let text = [
                Text::raw(format!("GPIO 2:  {:#018b} {:#06x} {:#05}\n", GPIO[2], GPIO[2], GPIO[2])),
                Text::raw(format!("GPIO 3:  {:#018b} {:#06x} {:#05}\n", GPIO[3], GPIO[3], GPIO[3]))
            ];

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

            let ADC: [Word; 2] = [200, 300];

            let text = [
                Text::raw(format!("ADC 0:   {:#018b} {:#06x} {:#05}\n", ADC[0], ADC[0], ADC[0]))
            ];

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

            let text = [
                Text::raw(format!("ADC 1:   {:#018b} {:#06x} {:#05}\n", ADC[1], ADC[1], ADC[1]))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::TOP | Borders::RIGHT)
                )
                .wrap(true)
                .render(&mut f, right_ADC[1]);

            //PWM
            let PWM: [Word; 2] = [5000, 3000];
            let text = [
                Text::raw(format!("PWM 0:   {:#018b} {:#06x} {:#05}\n", PWM[0], PWM[0], PWM[0]))
            ];

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

            let text = [
                Text::raw(format!("PWM 1:   {:#018b} {:#06x} {:#05}\n", PWM[1], PWM[1], PWM[1]))
            ];

            Paragraph::new(text.iter())
                .block(
                        Block::default()
                            .borders(Borders::TOP | Borders::RIGHT)
                )
                .wrap(true)
                .render(&mut f, right_PWM[1]);

            //Timers
            let timer = 30000;

            let text = [
                Text::raw(format!("_        {:#018b} {:#06x} {:#05}\n", timer, timer, timer))
            ];

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

            let clock = 20000;
            
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
