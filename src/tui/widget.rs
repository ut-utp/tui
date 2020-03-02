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
    fn draw(&mut self, _sim: &C, rect: Rect, buffer: &mut Buffer) {
        TuiWidget::draw(self, rect, buffer)
    }

    // fn render(&mut self, _sim: &C, f: &mut Frame<'_, B>, area: Rect) where Self: Sized {
    //     TuiWidget::render(self, f, area)
    // }

    fn render<'s>(&'s mut self, sim: &'s C, f: &mut Frame<'_, B>, area: Rect) {
        // TuiWidget::render(self, f, area)

        // This is tricky.
        //
        // We can't just call render on ourself because we can't guarantee that
        // we're Sized (if we try to, this trait is no longer object safe). So,
        // we get to do some fun gymnastics.
        //
        // What we do is pass ourselves into a wrapper widget that is Sized.
        // We exploit the fact that `TuiWidget::render` goes and passes
        // `TuiWidget::draw(self, ...)` the buffer. (TODO, finish comment)

        let mut fw = FakeWidget::<'s, 'a, 'int, _, _, _, _, _>(sim, self, PhantomData);
        <FakeWidget<'s, 'a, 'int, _, _, _, _, _> as TuiWidget>::render::<B>(&mut fw, f, area);

        // fw.rend(f, area)
        // TuiWidget::render(&mut &mut fw, f, area)
    }

    // This would be an associated const or a function if we didn't need to care
    // about object safety.
    //
    // Implementors (like Widgets) that contain other widgets should override
    // this to return false.
    fn is_single_widget(&self) -> bool { true }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>);
}

    where
        W: Widget<'a, 'int, C, I, O, B> + 'a

// This exists to circumvent the `Sized` requirement on `TuiWidget::render`.
#[allow(explicit_outlives_requirements)]
// struct FakeWidget<'s, 'a: 's, 'int: 's, C, I, O, B, W>
struct FakeWidget<'s, 'a, 'int, C, I, O, B, W>
(&'s C, &'s mut W, PhantomData<(&'a I, &'a O, B, &'int ())>)
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    W: Widget<'a, 'int, C, I, O, B> + ?Sized;

// trait Rats: Sized {
//     fn bar(&self) where Self: Sized {
//         println!("yui");
//     }
// }

// impl<'s, 'a, 'int, C, I, O, B, W> Rats for &'s mut FakeWidget<'s, 'a, 'int, C, I, O, B, W>
// where
//     C: Control + ?Sized + 'a,
//     I: InputSink + ?Sized + 'a,
//     O: OutputSource + ?Sized + 'a,
//     B: Backend,
//     W: Widget<'a, 'int, C, I, O, B>
// {}

// impl<'s, 'a: 's, 'int: 's, C, I, O, B, W> TuiWidget for FakeWidget<'s, 'a, 'int, C, I, O, B, W>
impl<'s, 'a, 'int, C, I, O, B, W> TuiWidget for /*&'s mut */FakeWidget<'s, 'a, 'int, C, I, O, B, W>
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

impl<'s, 'a, 'int, C, I, O, B, W> FakeWidget<'s, 'a, 'int, C, I, O, B, W>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    W: Widget<'a, 'int, C, I, O, B> + ?Sized
{
    fn rend(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        Self::render(self, f, area)
    }
}

}

// pub type Widgets = Vec<Box<dyn Widget>>;
