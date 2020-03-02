//! TODO!

use crate::tui::TuiData;
use crate::tui::events::WidgetEvent;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Layout};
use tui::Frame;
use tui::widgets::Widget as TuiWidget;
use tui::layout::Rect;

use std::marker::PhantomData;

// pub trait Widget: TuiWidget {
//     /// For functions that don't hold their own state and need a reference to
//     /// the [`Control`] impl to redraw themselves.
//     ///
//     /// By default, this just ignores the [`Control`] entirely and just calls
//     /// the regular draw function on
//     /// [the `tui` `Widget` trait](tui::widgets::Widget). Functions that don't
//     /// need a [`Control`] instance need not override the default impl.
//     ///
//     /// [`Control`]: `lc3_traits::control::Control`
//     fn draw<C: Control, B: Backend>(&mut self, _sim: &C, f: &mut Frame<'_, B>, area: Rect) {
//         self.render(f, area)
//     }

//     fn update<'a, 'int, C, I, O>(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>)
//     where
//         C: Control + ?Sized + 'a,
//         I: InputSink + ?Sized + 'a,
//         O: OutputSource + ?Sized + 'a;
// }

pub trait Widget<'a, 'int, C, I, O, B>: TuiWidget
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    // for<'s> FakeWidget<'s, 'a, 'int, C, I, O, B, Self>: Sized
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
    fn draw<C: Control, B: Backend>(&mut self, _sim: &C, f: &mut Frame<'_, B>, area: Rect) {
        self.render(f, area)
    }

    fn update<'a, 'int, C, I, O>(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>)
    where
        C: Control + ?Sized + 'a,
        I: InputSink + ?Sized + 'a,
        O: OutputSource + ?Sized + 'a;
}

pub type Widgets = Vec<Box<dyn Widget>>;
