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

    // This would be an associated const or a function if we didn't need to care
    // about object safety.
    //
    // Implementors (like Widgets) that contain other widgets should override
    // this to return false.
    fn is_single_widget(&self) -> bool { true }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>);
}

#[allow(explicit_outlives_requirements)]
struct SingleWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    widget: Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>,
    constraint: Constraint,
    // cached_rect: Option<Rect>,
    rect: Rect,
}

impl<'a, 'int, C, I, O, B> SingleWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn new(constraint: Constraint, widget: Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>) -> Self {
        Self {
            widget,
            constraint,
            // cached_rect: None,
            rect: Rect::default(),
        }
    }

    // fn invalidate_cached_rect(&mut self) {
    //     self.cached_rect = None;
    // }
}


#[allow(explicit_outlives_requirements)]
pub struct Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    widgets: Vec<SingleWidget<'a, 'int, C, I, O, B>>,
    layout: Layout,
    rects_valid: bool,
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
        // self.widgets.iter_mut().for_each(|w| w.invalidate_cached_rect());
        self.widgets.push(SingleWidget::new(constraint, Box::new(widget)));
        self.rects_valid = false;

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
        // if !self.widgets.iter().all(|w| w.cached_rect.is_some()) {
        //     // If any don't exist, update all:

        // }
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
        // if !self.widgets.iter().all(|w| w.cached_rect.is_some()) {
        //     //
        // }

        // self.update_rects(rect);

        // for sw in self.widgets {
        //     TuiWidget::draw(&mut *sw.widget, sw.rect, buffer);
        // }


        // Widget::draw(self, self.)
        // unreachable!("This should never be called. Call lc3_tui::Widget::draw instead.")


    }
}

// This exists to circumvent the `Sized` requirement on `TuiWidget::render`.
#[allow(explicit_outlives_requirements)]
struct FakeWidget<'s, 'a, 'int, C, I, O, B, W>
(&'s C, &'s mut W, PhantomData<(&'a I, &'a O, B, &'int ())>)
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

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    // fn draw(&mut self, sim: &C, f: &mut Frame<'_, B>, area: Rect) where Self: Sized {
    //     self.update_rects(area);

    //     for sw in self.widgets {
    //         Widget::draw(&mut *sw.widget, sim, f, sw.rect)
    //     }
    // }

    fn draw(&mut self, sim: &C, rect: Rect, buffer: &mut Buffer) {
        self.update_rects(rect);

        for sw in self.widgets.iter_mut() {
            Widget::draw(&mut *sw.widget, sim, sw.rect, buffer)
        }
    }

    fn render(&mut self, _sim: &C, f: &mut Frame<'_, B>, area: Rect) where Self: Sized {
        // This is tricky.
        //
        // We can't just call render on our children widgets because we can't
        // guarentee that they're Sized. So instead what we go and do is pass
        // ourselves into a wrapper widget that is Sized that just calls draw
        // on us with the buffer it gets from We exploit the fact that `TuiWidget::render` goes and
        // passes `TuiWidget::draw(self, ...)` the buffer
        TuiWidget::render(self, f, area)
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
        // unecessary; we can just always propagate, I think.
    }
}

// pub type Widgets = Vec<Box<dyn Widget>>;
