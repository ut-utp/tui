//! Module defining the layout of the widgets used by the TUI.

use crate::tui::widget::{Widgets, Widget};
use crate::widgets::*;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::layout::{Layout, Direction, Constraint};
use tui::widgets::{Block, Borders};
use tui::style::{Style, Color};

// Returns the root widget for our layout.
//
// This is currently 'static' (i.e. doesn't change based on the inputs given)
// but that could change in the future.
// TODO: potentially parameterize this from with user configurable options!
pub fn layout<'a, 'int: 'a, C, I, O, B: 'a>() -> impl Widget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    layout_tabs()
}

pub fn layout_tabs<'a, 'int: 'a, C, I, O, B: 'a>() -> Tabs<'a, 'int, C, I, O, B, impl Fn() -> TabsBar<'a, String>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    let horz = Layout::default().direction(Direction::Horizontal);
    let vert = Layout::default().direction(Direction::Vertical);
    let b = Block::default()
        .title_style(Style::default().fg(Color::Red))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Reset));
    let empty = Empty::default();
    
    let mut root = Widgets::new(vert.clone());
    let mut top = Widgets::new(horz.clone());

    let mut left = Widgets::new(vert.clone());
    let footer = Footer::default();
    let mem = Mem::default();
    let regs = Regs::default();
    let console = Console::default();
    let mut io = Widgets::new(vert.clone());
    //let mut top_left = Widgets::new(vert.clone());

    let _ = left.add_widget(Constraint::Percentage(80), mem, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Memory")))
        .add_widget(Constraint::Percentage(20), regs, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Registers + PC+ PSR").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))));

    let mut right = Widgets::new(vert.clone());

    let _ = io.add_widget(Constraint::Percentage(45), empty.focusable(true), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("GPIO").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(20), empty.focusable(true), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("ADC").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(15), empty.focusable(true), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("PWM").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(20), empty.focusable(true), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("Timers").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))));

    let _ = right.add_widget(Constraint::Percentage(60), console, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Console")))
        .add_widget(Constraint::Percentage(40), io, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("IO")));

    let _ = top.add_widget(Constraint::Percentage(50), left, None)
        .add_widget(Constraint::Percentage(50), right, None);

    let _ = root.add_widget(Constraint::Percentage(85), top, None)
        .add_widget(Constraint::Percentage(15), footer, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Footer")));

    let mut help = Widgets::new(horz.clone());
    let mut middle = Widgets::new(vert.clone());
    let help_text = Help::default();

    let _ = middle.add_widget(Constraint::Percentage(20), empty.focusable(false), None)
        .add_widget(Constraint::Percentage(60), help_text, Some(b.clone().border_style(Style::default().fg(Color::Yellow)).title("Help")))
        .add_widget(Constraint::Percentage(20), empty.focusable(false), None);

    let _ = help.add_widget(Constraint::Percentage(20), empty.focusable(false), None)
        .add_widget(Constraint::Percentage(60), middle, None)
        .add_widget(Constraint::Percentage(20), empty.focusable(false), None);

    let mut tabs = Tabs::new(root, "Root")
        .add(empty, "Foo")
        .add(empty, "Bar")
        .add(empty, "Baz")
        .add(help, "Help")
        .with_tabs_bar(|| {
            TabsBar::default()
                .block(Block::default().title("Tabs").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Cyan))
                // .divider(tui::symbols::DOT)
        });

    if crate::debug::in_debug_mode() {
        let events = Text::new(|t| t.log.as_ref().unwrap());

        let mut debug = Widgets::new(vert.clone());
        let _ = debug
            .add_widget(Constraint::Percentage(100), events, Some(b.clone().border_style(Style::default().fg(Color::Green)).title("Event Log")));

        tabs = tabs
            .add(debug, "Debug Info");
    }

    tabs
}
