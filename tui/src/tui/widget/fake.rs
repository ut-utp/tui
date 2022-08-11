//! Home of [`FakeWidget`]: a trick.

use super::WidgetTypes;
use super::{Widget, TuiWidget};
use crate::tui::TuiData;

use tui::buffer::Buffer;
use tui::layout::Rect;

// This used to exist to circumvent the `Sized` requirement on
// `TuiWidget::render`, but now serves as kind of a bridge between our `Widget`
// trait and `TuiWidget`.
pub(super) struct FakeWidget<'s, 'a, Wt, W>
(
    pub(super) &'s TuiData<'a, Wt::TuiTypes>,
    pub(super) &'s mut W,
)
where
    Wt: WidgetTypes,
    W: Widget<Wt> + ?Sized;

impl<'s, 'a, Wt, W> TuiWidget for FakeWidget<'s, 'a, Wt, W>
where
    Wt: WidgetTypes,
    W: Widget<Wt> + ?Sized
{
    fn render(self, rect: Rect, buffer: &mut Buffer) {
        debug_assert!(&buffer.area().union(rect) == buffer.area(), "drew {:?} on a {:?}", rect, buffer.area());
        Widget::draw(self.1, self.0, rect, buffer)
    }
}
