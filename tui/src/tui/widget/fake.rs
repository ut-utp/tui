//! Home of [`FakeWidget`]: a trick.

use super::{Widget, TuiWidget};
use crate::tui::TuiData;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::Frame;
use tui::layout::Rect;

use std::marker::PhantomData;

#[allow(explicit_outlives_requirements)]
// This exists to circumvent the `Sized` requirement on `TuiWidget::render`.
pub(in super) struct FakeWidget<'s, 'a, 'int, C, I, O, B, W>
(pub(super)&'s TuiData<'a, 'int, C, I, O>, pub(super)&'s mut W, pub(super)PhantomData<(&'a I, &'a O, B, &'int ())>)
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    W: Widget<'a, 'int, C, I, O, B> + ?Sized;

impl<'s, 'a, 'int, C, I, O, B, W> TuiWidget for FakeWidget<'s, 'a, 'int, C, I, O, B, W>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    W: Widget<'a, 'int, C, I, O, B> + ?Sized
{
    fn draw(&mut self, rect: Rect, buffer: &mut Buffer) {
        Widget::draw(self.1, self.0, rect, buffer)
    }

    // The default impl is fine; it'll dispatch to `draw`.
    //
    // Listed here anyways for clarity.
    fn render<BB: Backend>(&mut self, f: &mut Frame<'_, BB>, area: Rect) {
        f.render(self, area);

        // Goes to this:
        // pub fn render<W>(&mut self, widget: &mut W, area: Rect)
        // where
        //     W: Widget,
        // {
        //     widget.draw(area, self.terminal.current_buffer_mut());
        // }
    }
}
