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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

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
    fn draw(&mut self, _data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        TuiWidget::draw(self, area, buf)
    }

    fn render<'s>(&'s mut self, data: &'s TuiData<'a, 'int, C, I, O>, f: &mut Frame<'_, B>, area: Rect) {
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

        let mut fw = FakeWidget::<'s, 'a, 'int, _, _, _, _, _>(data, self, PhantomData);
        <FakeWidget<'s, 'a, 'int, _, _, _, _, _> as TuiWidget>::render::<B>(&mut fw, f, area);
    }

    // Return true or false indicating whether you (a widget) or your children
    // handled the event.
    //
    // This is useful for events that must be handled only once (i.e. changing
    // which widget is currently focused).
    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool;
}

pub fn increment(offset: u16, axis: Axis, area: Rect) -> Rect {
    if axis == Axis::X {
        if offset > area.width {
            let offset = area.width-1;
        }
        return Rect::new(area.x+offset, area.y, area.width.saturating_sub(offset), area.height);
    } else {
        if offset > area.height {
            let offset = area.height-1;
        }
        return Rect::new(area.x, area.y+offset, area.width, area.height.saturating_sub(offset));
    }   
}