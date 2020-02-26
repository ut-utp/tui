//! TODO!

use crate::tui::TuiData;
use crate::tui::events::WidgetEvent;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::Frame;
use tui::widgets::Widget as TuiWidget;
use tui::layout::Rect;

pub trait Widget: TuiWidget + Sized {
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
