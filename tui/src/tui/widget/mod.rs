//! TODO!

use std::marker::PhantomData;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::Frame;
use tui::layout::Rect;
use tui::terminal::Terminal;
use tui::widgets::StatefulWidget;

use crate::tui::TuiData;
use crate::tui::events::WidgetEvent;
use super::TuiTypes;

pub use tui::widgets::Widget as TuiWidget;

mod fake;
use fake::FakeWidget;

mod single;
mod grouped;
pub use grouped::Widgets;


pub mod util;

// we'd like for `B: Backend` to be a param on `render` and `update` but then
// this trait isn't object safe.
pub trait WidgetTypes {
    type TuiTypes: TuiTypes;
    type Backend: Backend;
}

#[derive(Debug)]
pub struct WidgetTypesSpec<T: TuiTypes, B: Backend>(PhantomData<(T, B)>);
impl<T: TuiTypes, B: Backend> WidgetTypes for WidgetTypesSpec<T, B> {
    type TuiTypes = T;
    type Backend = B;
}

#[allow(type_alias_bounds)]
pub type ControlTy<Wt: WidgetTypes> = <<Wt as WidgetTypes>::TuiTypes as TuiTypes>::Control;
#[allow(type_alias_bounds)]
pub type InputTy  <Wt: WidgetTypes> = <<Wt as WidgetTypes>::TuiTypes as TuiTypes>::Input;
#[allow(type_alias_bounds)]
pub type OutputTy <Wt: WidgetTypes> = <<Wt as WidgetTypes>::TuiTypes as TuiTypes>::Output;
#[allow(type_alias_bounds)]
pub type BackendTy<Wt: WidgetTypes> = <Wt as WidgetTypes>::Backend;

#[allow(type_alias_bounds)]
pub type Data<'a, Wt: WidgetTypes> = TuiData<'a, <Wt as WidgetTypes>::TuiTypes>;

pub trait Widget<Wt: WidgetTypes> {
    /// For functions that don't hold their own state and need a reference to
    /// the [`Control`] impl to redraw themselves.
    ///
    /// By default, this just ignores the [`Control`] entirely and just calls
    /// the regular draw function on
    /// [the `tui` `Widget` trait](tui::widgets::Widget). Functions that don't
    /// need a [`Control`] instance need not override the default impl.
    ///
    /// [`Control`]: `lc3_traits::control::Control`
    fn draw(&mut self, _data: &TuiData<'_, Wt::TuiTypes>, area: Rect, buf: &mut Buffer);

    fn render(&mut self, data: &TuiData<'_, Wt::TuiTypes>, f: &mut Frame<'_, Wt::Backend>, area: Rect) {
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
        //
        // Note: the above used to be true, but now we just do the below so that
        // we can draw this widget instance without it being _consumed_.

        // TODO: do we still need this?
        // TODO: bump TUI version

        let fw = FakeWidget::<'_, '_, _, _>(data, self);
        f.render_widget(fw, area);
    }

    // Return true or false indicating whether you (a widget) or your children
    // handled the event.
    //
    // This is useful for events that must be handled only once (i.e. changing
    // which widget is currently focused).
    fn update(
        &mut self,
        event: WidgetEvent,
        data: &mut TuiData<'_, Wt::TuiTypes>,
        terminal: &mut Terminal<Wt::Backend>,
    ) -> bool;
}
