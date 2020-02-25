//! TODO!

use crate::tui::Tui;
use crate::tui::events::Event;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::Frame;
use tui::widgets::Widget as TuiWidget;
use tui::layout::Rect;

pub trait Widget: TuiWidget + Sized {
    fn draw<C: Control, B: Backend>(&mut self, _sim: &mut C, f: &mut Frame<'_, B>, area: Rect) {
        self.render(f, area)
    }

    fn update<'a, 'int, C, I, O>(&mut self, event: Event, data: Tui<'a, 'int, C, I, O>)
    where
        C: Control + ?Sized + 'a,
        I: InputSink + ?Sized + 'a,
        O: OutputSource + ?Sized + 'a;
}

pub type Widgets = Vec<Box<dyn Widget>>;
