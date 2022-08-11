//! A widget that does nothing but occupy space.
//!
//! Useful for testing and for blank spaces.

use super::widget_impl_support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Empty {
    pub focusable: bool,
}

impl Default for Empty {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Empty {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl<Wt: WidgetTypes> Widget<Wt> for Empty {
    fn draw(&mut self, _data: &Data<Wt>, _area: Rect, _buf: &mut Buffer) {
        // Do nothing!
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
