//! Widgets for the Tui ([`Widget`] implementations).
//!
//! [`Widget`]: crate::tui::widget::Widget

pub(in self) mod widget_impl_support {
    pub use crate::tui::widget::{TuiWidget, Widget};
    pub use crate::tui::TuiData;
    pub use crate::tui::events::WidgetEvent;

    pub use lc3_application_support::io_peripherals::{InputSink, OutputSource};
    pub use lc3_traits::control::Control;

    pub use tui::backend::Backend;
    pub use tui::buffer::Buffer;
    pub use tui::layout::Rect;
}

mod empty;
pub use empty::*;
