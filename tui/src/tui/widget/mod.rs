//! TODO!

use crate::tui::TuiData;
use crate::tui::events::WidgetEvent;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::Frame;
use tui::layout::Rect;
use tui::terminal::Terminal;

use std::marker::PhantomData;


pub use tui::widgets::Widget as TuiWidget;

mod fake;
use fake::FakeWidget;

mod single;
mod grouped;
pub use grouped::Widgets;

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
    fn draw(&mut self, _data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        TuiWidget::draw(self, area, buf)
    }

    fn render<'s>(&'s mut self, data: &'s TuiData<'a, 'int, C, I, O>, f: &mut Frame<'_, B>, area: Rect) {
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

        let mut fw = FakeWidget::<'s, 'a, 'int, _, _, _, _, _>(data, self, PhantomData);
        <FakeWidget<'s, 'a, 'int, _, _, _, _, _> as TuiWidget>::render::<B>(&mut fw, f, area);
    }

    // Return true or false indicating whether you (a widget) or your children
    // handled the event.
    //
    // This is useful for events that must be handled only once (i.e. changing
    // which widget is currently focused).
    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool;
}

// TODO: should this actually be pub?
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
}

// TODO: should this actually be pub?
pub fn increment(offset: u16, axis: Axis, area: Rect) -> Rect {
    let mut offset = offset;
    if axis == Axis::X {
        if offset > area.width {
            offset = area.width;
        }
        return Rect::new(area.x+offset, area.y, area.width.saturating_sub(offset), area.height);
    } else {
        if offset > area.height {
            offset = area.height;
        }
        return Rect::new(area.x, area.y+offset, area.width, area.height.saturating_sub(offset));
    }
}

/// Takes an string with ANSI escape sequences and appends them to a `Vec` of
/// `Vec`s of `tui::widgets::Text` items, separated by line.
///
/// Only display attributes are supported; a full list of ANSI/VT100 escape
/// sequences is available [here](http://www.termsys.demon.co.uk/vtansi.htm).
pub fn ansi_string_to_tui_text<'a>(
    inp: &'a str,
    current_style: &mut Style,
    out: &mut Vec<Vec<TuiText<'a>>
) -> Result<(), &'a str> {
    if out.is_empty() {
        out.push(vec![]);
    }

    let mut iter = inp.chars_indices();
    let mut segment_start_idx = 0;

    const ESC: char = (0x1B as u8) as char;
    const NEW: char = '\n';

    loop {
        match iter.next() {
            Some((idx, NEW)) => {
                // The current line is finished.

                // First, make the `Text` instance that corresponds to this part
                // of the line:
                let (s, _) = inp.split_at(idx + 1);
                let (_, s) = s.split_at(segment_start_idx);
                segment_start_idx = idx + 1;

                let text = Text::styled(s, current_style);

                // Then, finish off this line:
                lines.last_mut().unwrap().push(text);
                lines.push(vec![]);
            }

            Some((idx, ESC)) => {
                // An escape code!

                // In case we don't yet have all the characters we need to
                // process this escape code fully, let's hold on to a reference
                // to this part of the input so we can tell the caller to try
                // again starting here.
                let remaining = s.split_at(idx);

                // If this the start of a new segment, we need to finish off the
                // last segment first:
                //
                // Note that because we adjust the segment start index at the
                // end of this block, this only runs if the escape code follows
                // a run of normal non-escape characters. Put another way, we
                // won't be making empty `Text` instances out of back-to-back
                // escape codes.
                if segment_start_idx != idx {
                    // Same offset math as above:
                    let (s, _) = inp.split_at(idx);
                    let (_, s) = s.split_at(segment_start_idx);

                    let text = Text::styled(s, current_style);
                    lines.last_mut().unwrap().push(text);
                }

                let mut consumed_characters = 1;
                let (_, next_char) = iter.next();

                match next_char {
                    // Reset
                    'c' => { },

                    // Default font
                    '(' => { },

                    // Alternate font
                    ')' => { },

                    // Save current cursor position
                    '7' => { }

                    // Restore cursor position
                    '8' => { }

                    // Scroll down one line
                    'D' => { },

                    // Scroll up one line
                    'M' => { }

                    // Sets a tab at the current position
                    'H' => { }

                    //
                    '[' => {

                    }
                }

                // Finally, we don't want the escape code to appear in the
                // printed text so we'd better adjust the segment start index
                // accordingly:
                segment_start_idx = idx + consumed_characters + 1;
            }

            Some((_, _)) => {
                // Same segment, same line. Nothing to do here.
            }

            None => {
                // We're done! Push any characters that are in this segment and
                // return.

                let (_, s) = inp.split_at(segment_start_idx);
                let text = Text::styled(s, current_style);

                lines.last_mut().unwrap().push(text);

                return lines
            },
        }
    }
}
