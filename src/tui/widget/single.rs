//! TODO!

use super::Widget;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::layout::Constraint;
use tui::layout::Rect;

#[allow(explicit_outlives_requirements)]
pub(in super) struct SingleWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub(super) widget: Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>,
    pub(super) constraint: Constraint,
    pub(super) rect: Rect,
}

impl<'a, 'int, C, I, O, B> SingleWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub(super) fn new(constraint: Constraint, widget: Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>) -> Self {
        Self {
            widget,
            constraint,
            rect: Rect::default(),
        }
    }
}
