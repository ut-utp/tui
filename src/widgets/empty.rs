//! A widget that does nothing but occupy space.
//!
//! Useful for testing and for blank spaces.

use super::widget_impl_support::*;

pub struct Empty;

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
    fn update(&mut self, _event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) {
        // Do nothing!
    }
}
