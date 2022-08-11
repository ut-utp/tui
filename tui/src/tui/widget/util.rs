//! Miscellaneous helper functions and types for Widget implementors.

use crate::strings::{s, DotDotDot};

use tui::backend::Backend;
use tui::layout::Rect;
use tui::terminal::Terminal;

specialize! {
    desktop => {
        pub trait ConditionalSendBound: Send {}
        impl<B: Backend> ConditionalSendBound for Terminal<B> where Terminal<B>: Send { }
    }

    web => {
        pub trait ConditionalSendBound {}
        impl<B: Backend> ConditionalSendBound for Terminal<B> { }
    }

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

trait OddEven {
    fn is_odd(&self) -> bool;

    fn is_even(&self) -> bool { !self.is_odd() }
}

impl OddEven for usize {
    fn is_odd(&self) -> bool { self % 1 == 1 }
}

pub fn trim_to_rect(inp: &str, Rect { width, .. }: Rect) -> String {
    trim_to_width(inp, width)
}

pub fn trim_to_width(inp: &str, width: u16) -> String {
    let mut s = String::from(inp);

    let width = width as usize;
    let num_chars = s.chars().count();

    if num_chars <= width {
        s
    } else {
        // We're assuming the width is _at least_ 2 here; otherwise this isn't
        // really going to help.

        // If we have N characters and a width of W, we're going to remove the
        // 'middle' (N - W) + 2 characters and replace them with dots.
        //
        // We'll take ceil((w - 2) / 2) chars from the front and floor((w - 2) /
        // 2) chars from the back.

        let prefix = ((width - 2) / 2) + if width.is_odd() { 1 } else { 0 };
        let prefix: String = s.chars().take(prefix).collect();

        let suffix = num_chars - ((width - 2 ) / 2);
        let suffix: String = s.chars().skip(suffix).collect();

        format!("{}{}{}", prefix, s!(DotDotDot), suffix)
    }
}
