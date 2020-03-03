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

    let mut root = Widgets::new(horz.clone());

    let mut left = Widgets::new(vert.clone());
    let mut top_left = Widgets::new(vert.clone());

    let _ = top_left
        .add_widget(Constraint::Percentage(33), empty.focusable(true), Some(b.clone().title("Top Left One")))
        .add_widget(Constraint::Percentage(33), empty.focusable(false), Some(b.clone().title("Top Left Two -- Unfocusable")))
        .add_widget(Constraint::Percentage(34), empty.focusable(true), Some(b.clone().title("Top Left Three")));

    let _ = left.add_widget(Constraint::Percentage(50), top_left, Some(b.clone().title("Top Left")))
        .add_widget(Constraint::Percentage(50), empty.focusable(false), Some(b.clone().title("Bottom Left -- Unfocusable")));

    let mut right = Widgets::new(vert.clone());
    let _ = right.add_widget(Constraint::Percentage(50), empty.focusable(true), Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Top Right")))
        .add_widget(Constraint::Percentage(50), empty.focusable(true), Some(b.clone().title("Bottom Right")));

    let _ = root.add_widget(Constraint::Percentage(40), left, None)
        .add_widget(Constraint::Percentage(10), empty, None)
        .add_widget(Constraint::Percentage(10), empty, None)
        .add_widget(Constraint::Percentage(40), right, None);

    let mut tabs = Tabs::new(root, "Root")
        .add(empty, "Foo")
        .add(empty, "Bar")
        .add(empty, "Baz")
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
