//! TODO!

use crate::tui::TuiData;
use crate::tui::events::WidgetEvent;
use super::single::SingleWidget;
use super::{TuiWidget, Widget};

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::layout::{Layout, Constraint, Rect};
use tui::terminal::Frame;


/// A bunch of Widgets that split the are they are given in *one* direction. In
/// other words, a horizontal or vertical set of widgets.
///
/// Nest these like you'd nest [`Layout`]s for more complicated arrangements.
///
/// [`Layout`]: tui::layout::Layout
#[allow(explicit_outlives_requirements)]
pub struct Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    /// The widgets within.
    widgets: Vec<SingleWidget<'a, 'int, C, I, O, B>>,
    /// Overall `Layout` for the Widgets. This is used to set the margins and
    /// direction of the Widgets; any constraints given will be ignored.
    layout: Layout,
    /// Whether or not the cached `Rect` in each `SingleWidget` is still valid.
    rects_valid: bool,
    /// The index of the widget to dispatch events to.
    focused: Option<usize>,
}

impl<'a, 'int, C, I, O, B> Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            widgets: Vec::new(),
            rects_valid: false,
            focused: None,
        }
    }

    pub fn add_widget<W>(&mut self, constraint: Constraint, widget: W) -> &mut Self
    where
        W: Widget<'a, 'int, C, I, O, B> + 'a
    {
        self.widgets.push(SingleWidget::new(constraint, Box::new(widget)));
        self.rects_valid = false; // We need to recalculate positions now!

        self
    }

    fn update_rects(&mut self, area: Rect) {
        if !self.rects_valid {
            let layout = self.layout.clone();

            let constraints: Vec<_> = self.widgets
                .iter()
                .map(|w| w.constraint)
                .collect();

            let rects = layout
                .constraints(constraints)
                .split(area);

            assert_eq!(self.widgets.len(), rects.len());

            for (idx, rect) in rects.iter().enumerate() {
                self.widgets[idx].rect = *rect;
            }

            self.rects_valid = true;
        }
    }
}


impl<'a, 'int, C, I, O, B> TuiWidget for Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, _rect: Rect, _buffer: &mut Buffer) {
        // self.update_rects(rect);

        // for sw in self.widgets {
        //     TuiWidget::draw(&mut *sw.widget, sw.rect, buffer);
        // }

        unreachable!("This should never be called. Call `lc3_tui::Widget::draw` instead.")
    }
}



impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, sim: &C, rect: Rect, buffer: &mut Buffer) {
        self.update_rects(rect);

        for sw in self.widgets.iter_mut() {
            Widget::draw(&mut *sw.widget, sim, sw.rect, buffer)
        }
    }

    fn is_single_widget(&self) -> bool { false } // TODO: remove from trait, probably

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) {
        todo!()

        // invalidate (recursively) on resize events (i.e. propagate the resize
        // event)

        // use clicked events to update the currently focused thing
        // (propagate these as well since what's under us might not be a single
        // widget)

        // dispatch key events to the currently focused thing

        // The intention behind `Widget::is_single_widget` was to know when to
        // propagate events to the thing under us, but really this is
        // unnecessary; we can just always propagate, I think.
    }
}
