//! Widgets for the Tui ([`Widget`] implementations).
//!
//! [`Widget`]: crate::tui::widget::Widget

pub(in self) mod widget_impl_support {
    pub use crate::tui::widget::{TuiWidget, Widget, increment, Axis};
    pub use crate::tui::TuiData;
    pub use crate::tui::events::{WidgetEvent, FocusEvent};
    pub use crate::debug::in_debug_mode;

    pub use lc3_application_support::io_peripherals::{InputSink, OutputSource};
    pub use lc3_traits::control::Control;

    pub use tui::backend::Backend;
    pub use tui::buffer::Buffer;
    pub use tui::layout::{Rect, Layout, Direction, Constraint};
    pub use tui::terminal::Terminal;
    pub use tui::widgets::{Text as TuiText, Paragraph};
    pub use tui::style::{Color as Colour, Style};
    pub use tui::layout::Alignment;

    pub use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, MouseEvent, MouseButton};
}

mod empty;
pub use empty::*;

mod tabs;
pub use tabs::*;

mod text;
pub use text::*;

mod footer;
pub use footer::*;

mod help;
pub use help::*;

mod mem;
pub use mem::*;

mod regs;
pub use regs::*;

mod console;
pub use console::*;

mod gpio;
pub use gpio::*;

mod adc;
pub use adc::*;

mod pwm;
pub use pwm::*;

mod timers;
pub use timers::*;

mod clock;
pub use clock::*;

mod button;
pub use button::*;

use tui::layout::Rect;

