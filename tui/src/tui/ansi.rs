//! Utilities for dealing with strings containing ANSI escape codes.

use tui::widgets::Text as TuiText;
use tui::style::Style;

use std::ops::{Bound, Deref, RangeBounds};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct AnsiTextContainer<'a> {
    lines: Vec<Vec<TuiText<'a>>>,
    pending: String,
    style: Style,
}

impl<'a> AnsiTextContainer<'a> {
    pub fn new() -> Self {
        let mut lines = Vec::with_capacity(1024);
        lines.push(vec![]);

        Self {
            lines,
            pending: String::new(),
            style: Style::default(),
        }
    }

    pub fn push_string(&mut self, new: String) {
        if self.pending.is_empty() {
            self.pending = new;
        } else {
            self.pending.push_str(new.as_str());
        }

        match ansi_string_to_tui_text(
            &self.pending.as_str(),
            &mut self.style,
            &mut self.lines
        ) {
            Ok(()) => self.pending.clear(),
            Err(remaining) => self.pending = String::from(remaining),
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear()
    }

    /// Get an iterator over [`Text`] instances for a given range of lines.
    ///
    /// [`Text`]: tui::widgets::Text
    pub fn get_lines(&self, lines: impl RangeBounds<usize>) -> impl Iterator<Item = &TuiText<'a>> {
        use Bound::*;

        // Inclusive of lower.
        let lower = match lines.start_bound() {
            Included(i) => *i,
            Excluded(e) => *e + 1,
            Unbounded => 0,
        };

        // Exclusive of upper.
        let upper = match lines.end_bound() {
            Included(i) => *i + 1,
            Excluded(e) => *e,
            Unbounded => self.lines.len(),
        };

        let lower = lower.max(self.lines.len() - 1);
        let upper = upper.max(self.lines.len());

        let num_to_take = upper - lower;

        self.lines.iter()
            .skip(lower)
            .take(num_to_take)
            .flatten()
    }

    /// Gets a `String` containing all the lines in the container.
    ///
    /// Note that this elides any ANSI escape sequences (including display
    /// attributes).
    pub fn as_string(&self) -> String {
        self.get_lines(..)
            .map(|t| match t {
                TuiText::Raw(s) => s,
                TuiText::Styled(s, _) => s,
            }.deref())
            .collect()
    }
}
