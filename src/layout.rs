//! Module defining the layout of the widgets used by the TUI.

use crate::tui::widget::Widgets;
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
pub fn layout<'a, 'int: 'a, C, I, O, B: 'a>() -> Widgets<'a, 'int, C, I, O, B>
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

    let mut root = Widgets::<'a, 'int, C, I, O, B>::new(horz.clone());

    let mut left = Widgets::<'a, 'int, C, I, O, B>::new(vert.clone());
    let _ = left.add_widget(Constraint::Percentage(50), Empty, Some(b.clone().title("Top Left")))
        .add_widget(Constraint::Percentage(50), Empty, Some(b.clone().title("Bottom Left")));

    let mut right = Widgets::new(vert.clone());
    let _ = right.add_widget(Constraint::Percentage(50), Empty, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Top Right")))
        .add_widget(Constraint::Percentage(50), Empty, Some(b.clone().title("Bottom Right")));

    let _ = root.add_widget(Constraint::Percentage(40), left, None)
        .add_widget(Constraint::Percentage(20), Empty, None)
        .add_widget(Constraint::Percentage(40), right, None);

    root
}
