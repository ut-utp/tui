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

fn main() -> Result<(), failure::Error> {
    let screen = AlternateScreen::to_alternate(true)?;
    let backend = CrosstermBackend::with_alternate_screen(screen)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    /*let cursor_screen = AlternateScreen::to_alternate(true)?;
    let mut cursor_backend = CrosstermBackend::with_alternate_screen(cursor_screen)?;
*/
    let mut z = 0;
    loop {
        z = z + 1;
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        // Constraint::Percentage(50),
                        // Constraint::Percentage(20),
                        // Constraint::Percentage(20),
                        // Constraint::Percentage(20),
                        // Constraint::Percentage(10),
                        Constraint::Min(10),
                        Constraint::Length(4),
                    ].as_ref()
                )
                .split(f.size());

            Block::default()
                 .title("Footer")
                 .borders(Borders::ALL)
                 .render(&mut f, chunks[1]);

            let body = chunks[0];

            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(body);

            let left_pane = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3), Constraint::Length(10)].as_ref())
                .split(panes[0]);

            Block::default()
                 .title("Memory")
                 .borders(Borders::ALL)
                 .render(&mut f, left_pane[0]);

            Block::default()
                 .title("Registers + PC + PSR")
                 .borders(Borders::ALL)
                 .render(&mut f, left_pane[1]);

            let right_pane = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(13), Constraint::Length(15)].as_ref())
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

            let io_panel = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(4), Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(right_pane[1]);

            
            Block::default()
                 .title("GPIO")
                 .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                 .render(&mut f, io_panel[0]);

            Block::default()
                 .title("ADC")
                 .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                 .render(&mut f, io_panel[1]);

            Block::default()
                 .title("PWM")
                 .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                 .render(&mut f, io_panel[2]);

            let timers_n_clock = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
                .split(io_panel[3]);

            Block::default()
                 .title("Timers")
                 .borders(Borders::ALL & !(Borders::RIGHT))
                 .render(&mut f, timers_n_clock[0]);

            Block::default()
                 .title("Clock")
                 .borders(Borders::ALL)
                 .render(&mut f, timers_n_clock[1]);


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
    
            /*let x = cursor_backend.get_cursor();
            let x = match x {
                Ok(data) => data,
                Err(error) => (0,0),
            };*/
            let text = [
                //Text::styled(format!("X position: {}\n", x.0), Style::default().modifier(Modifier::BOLD)),
                //Text::styled(format!("Y position: {}\n", x.1), Style::default().modifier(Modifier::BOLD)),
                Text::styled(format!("{}", z), Style::default().modifier(Modifier::BOLD))
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
            
            
        })?;
    }
    

    
    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}

// fn main() {
//     println!("Hello, world!");
// }
