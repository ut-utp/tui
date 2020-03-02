//! TODO!

use super::{Widget, TuiWidget};

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::buffer::Buffer;
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::widgets::Block;


#[allow(explicit_outlives_requirements)]
pub(in super) struct SingleWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub(super) widget: Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>,
    pub(super) block: Option<Block<'a>>,
    pub(super) constraint: Constraint,
    pub(super) area: Rect,
}

impl<'a, 'int, C, I, O, B> SingleWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    pub(super) fn new(constraint: Constraint, widget: Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>, block: Option<Block<'a>>) -> Self {
        Self {
            widget,
            block,
            constraint,
            area: Rect::default(),
        }
    }

    pub(super) fn draw(&mut self, sim: &C, buf: &mut Buffer, focused: bool) {
        // If we have a block, draw it.
        let area = if let Some(ref mut block) = self.block {
            // Change the border colour of the block if we're focused.
            if focused {
                let mut block = block
                    .clone()
                    .border_style(Style::default().fg(Color::Red));

                block.draw(self.area, buf);
            } else {
                block.draw(self.area, buf);
            }

            block.inner(self.area)
        } else {
            self.area
        };

        Widget::draw(&mut *self.widget, sim, area, buf)
    }
}
