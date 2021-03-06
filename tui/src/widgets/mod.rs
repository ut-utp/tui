//! Widgets for the Tui ([`Widget`] implementations).
//!
//! [`Widget`]: crate::tui::widget::Widget

pub(in self) mod widget_impl_support {
    pub use crate::tui::widget::{TuiWidget, Widget};
    pub use crate::tui::widget::util::{trim_to_width, trim_to_rect, increment, Axis};
    pub use crate::tui::widget::util::ConditionalSendBound;
    pub use crate::tui::ansi::AnsiTextContainer;
    pub use crate::tui::TuiData;
    pub use crate::tui::events::{WidgetEvent, FocusEvent};

    pub use crate::debug::{in_debug_mode, run_if_debugging};
    pub use crate::colours::c;
    pub use crate::strings::*;

    pub use lc3_application_support::io_peripherals::{InputSink, OutputSource};
    pub use lc3_traits::control::Control;

    pub use tui::backend::Backend;
    pub use tui::buffer::Buffer;
    pub use tui::layout::{Rect, Layout, Direction, Constraint};
    pub use tui::terminal::Terminal;
    pub use tui::widgets::{Text as TuiText, Paragraph, Gauge, Block, Borders};
    pub use tui::style::{Color as Colour, Style, Modifier};
    pub use tui::layout::Alignment;

    //pub use TuiColour;

    pub use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, MouseEvent, MouseButton};
}

mod empty;
pub use empty::*;

mod tabs;
pub use tabs::*;

mod text;
pub use text::*;

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

mod load_button;
pub use load_button::*;

mod console_peripherals;
pub use console_peripherals::*;

mod watch_window;
pub use watch_window::*;

mod break_window;
pub use break_window::*;

mod call_stack;
pub use call_stack::*;

mod modeline;
pub use modeline::*;

mod mem_reg_interface;
pub use mem_reg_interface::*;

mod root_widget;
pub use root_widget::*;
