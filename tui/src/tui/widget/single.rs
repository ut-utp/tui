//! TODO!

use super::{WidgetTypes, Data};
use super::{Widget, TuiWidget};

use tui::buffer::Buffer;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::widgets::Block;


pub(in super) struct SingleWidget<'a, Wt: WidgetTypes> {
    pub(super) widget: Box<dyn Widget<Wt> + 'a>,
    pub(super) block: Option<Block<'a>>,
    pub(super) constraint: Constraint,
    pub(super) area: Rect,
}

impl<'a, Wt: WidgetTypes> SingleWidget<'a, Wt> {
    pub(super) fn new(constraint: Constraint, widget: Box<dyn Widget<Wt> + 'a>, block: Option<Block<'a>>) -> Self {
        Self {
            widget,
            block,
            constraint,
            area: Rect::default(),
        }
    }

    pub(super) fn draw(&mut self, data: &Data<Wt>, buf: &mut Buffer, focused: bool) {
        // If we have a block, draw it.
        let area = if let Some(ref mut block) = self.block {
            // Change the border colour of the block if we're focused.
            if focused {
                let mut block = block
                    .clone()
                    .border_style(Style::default().fg(Color::Red));

                block.render(self.area, buf);
            } else {
                block.clone().render(self.area, buf);
            }

            block.inner(self.area)
        } else {
            self.area
        };

        if buf.area().union(area) == *buf.area() {
            Widget::draw(&mut *self.widget, data, area, buf)
        } else {
            log::debug!("tried to draw {:?} on a {:?}", area, buf.area());
        }
    }
}
