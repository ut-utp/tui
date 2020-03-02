//! TODO!

use crate::tui::TuiData;
use crate::tui::events::WidgetEvent;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::Frame;
use tui::layout::Rect;

use std::marker::PhantomData;


pub use tui::widgets::Widget as TuiWidget;

mod fake;
use fake::FakeWidget;

mod single;
mod grouped;
pub use grouped::Widgets;

pub trait Widget<'a, 'int, C, I, O, B>: TuiWidget
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    /// For functions that don't hold their own state and need a reference to
    /// the [`Control`] impl to redraw themselves.
    ///
    /// By default, this just ignores the [`Control`] entirely and just calls
    /// the regular draw function on
    /// [the `tui` `Widget` trait](tui::widgets::Widget). Functions that don't
    /// need a [`Control`] instance need not override the default impl.
    ///
    /// [`Control`]: `lc3_traits::control::Control`
    fn draw(&mut self, _sim: &C, rect: Rect, buffer: &mut Buffer) {
        TuiWidget::draw(self, rect, buffer)
    }

    fn render<'s>(&'s mut self, sim: &'s C, f: &mut Frame<'_, B>, area: Rect) {
        // This is tricky.
        //
        // We can't just call render on ourself because we can't guarantee that
        // we're Sized (if we try to, this trait is no longer object safe). So,
        // we get to do some fun gymnastics.
        //
        // What we do is pass ourselves into a wrapper widget that is Sized.
        // We exploit the fact that `TuiWidget::render` goes and passes
        // `TuiWidget::draw(self, ...)` the buffer; our impl of `TuiWidget` on
        // `FakeWidget` goes and passes this buffer to the wrapped widget's
        // `TuiWidget::draw` function.

        let mut fw = FakeWidget::<'s, 'a, 'int, _, _, _, _, _>(sim, self, PhantomData);
        <FakeWidget<'s, 'a, 'int, _, _, _, _, _> as TuiWidget>::render::<B>(&mut fw, f, area);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>);
}
