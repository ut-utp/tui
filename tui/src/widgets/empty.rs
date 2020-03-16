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

impl TuiWidget for Empty {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        // Do nothing!
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Empty
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
