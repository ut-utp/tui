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

/// Takes an string with ANSI escape sequences and appends them to a `Vec` of
/// `Vec`s of `tui::widgets::Text` items, separated by line.
///
/// Only display attributes are supported; a full list of ANSI/VT100 escape
/// sequences is available [here](http://www.termsys.demon.co.uk/vtansi.htm).
///
/// Returns an `Err` containing any characters that were not able to be
/// processed because there aren't currently enough characters to do so. This
/// happens when the input ends in the middle of an escape sequence. When this
/// happens, callers should be sure to include the unprocessed characters on the
/// next invocation of this function.
pub fn ansi_string_to_tui_text<'s, 't>(
    inp: &'s str,
    current_style: &'s mut Style,
    out: &mut Vec<Vec<TuiText<'t>>>,
) -> Result<(), &'s str> {
    if out.is_empty() {
        out.push(vec![]);
    }

    let mut iter = inp.char_indices();
    let mut segment_start_idx = 0;

    const ESC: char = 0x1B as char;
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

                let text = TuiText::<'t>::styled(
                    Cow::Borrowed(s).into_owned(),
                    current_style.clone()
                );

                // Then, finish off this line:
                out.last_mut().unwrap().push(text);
                out.push(vec![]);
            }

            Some((idx, ESC)) => {
                // An escape code!

                // In case we don't yet have all the characters we need to
                // process this escape code fully, let's hold on to a reference
                // to this part of the input so we can tell the caller to try
                // again starting here.
                let (_, remaining) = inp.split_at(idx);

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

                    let text = TuiText::<'t>::styled(
                        Cow::Borrowed(s).into_owned(),
                        current_style.clone()
                    );

                    out.last_mut().unwrap().push(text);
                }

                let mut consumed_characters = 1;

                macro_rules! next {
                    () => {
                        if let Some((_, next_char)) = iter.next() {
                            consumed_characters += 1;
                            next_char
                        } else {
                            return Err(remaining)
                        }
                    };
                }

                // Things that are unsupported are processed (i.e. _consumed_)
                // but unused; we don't do anything in these cases.
                match next!() {
                    // Reset
                    'c' => { current_style.reset() }

                    // Default font
                    '(' => { /* Unsupported! */ }

                    // Alternate font
                    ')' => { /* Unsupported! */ }

                    // Save current cursor position
                    '7' => { /* Unsupported! */ }

                    // Restore cursor position
                    '8' => { /* Unsupported! */ }

                    // Scroll down one line
                    'D' => { /* Unsupported! */ }

                    // Scroll up one line
                    'M' => { /* Unsupported! */ }

                    // Sets a tab at the current position
                    'H' => { /* Unsupported! */ }

                    // Lots of things..
                    '[' => match next!() {
                        // Query Device Code
                        'c' => { /* Unsupported */ }

                        // Save current cursor position
                        's' => { /* Unsupported */ }

                        // Restore cursor position
                        'u' => { /* Unsupported */ }

                        // Enable scrolling
                        'r' => { /* Unsupported */ }

                        // Clears a tab at the current position
                        'g' => { /* Unsupported */ }

                        // Erases til the end of the line
                        'K' => { /* Unsupported */ }

                        // Erase til the bottom of the screen
                        'J' => { /* Unsupported */ }

                        // Print the current screen
                        'i' => { /* Unsupported */ }

                        // Lots of things..
                        c @ '0'..='9' => match (c, next!()) {
                            // Query Device Status
                            ('5', 'n') => { /* Unsupported */ }

                            // Report Device OK
                            ('0', 'n') => { /* Unsupported */ }

                            // Report Device Failure
                            ('3', 'n') => { /* Unsupported */ }

                            // Query Cursor Position
                            ('6', 'n') => { /*  Unsupported */ }

                            // Enable Line Wrap
                            ('7', 'h') => { /* Unsupported */ }

                            // Disable Line Wrap
                            ('7', 'l') => { /* Unsupported */ }

                            // Clear All Tabs
                            ('3', 'g') => { /* Unsupported */ }

                            // Erase Start of Line
                            ('1', 'K') => { /* Unsupported */ }

                            // Erase Line
                            ('2', 'K') => { /* Unsupported */ }

                            // Erase Up
                            ('1', 'J') => { /* Unsupported */ }

                            // Erase Screen
                            ('2', 'J') => { /* Unsupported */ }

                            // Print Line
                            ('1', 'i') => { /* Unsupported */ }

                            // Stop Print Log
                            ('4', 'i') => { /* Unsupported */ }

                            // Start Print Log
                            ('5', 'i') => { /* Unsupported */ }

                            // The rest are variable length codes:
                            (a, b) => {
                                let mut nums: Vec<u16> = Vec::new();
                                let mut s = String::new();
                                s.push(a);

                                let mut last = b;

                                'c: loop {
                                    match last {
                                        n @ '0'..='9' => {
                                            s.push(n);
                                        }

                                        // Note: ITU T.416 (colon delimited) not supported.
                                        ';' => {
                                            // End of a number.
                                            if let Ok(num) = s.parse() {
                                                nums.push(num);
                                                s.clear();
                                            } else {
                                                // Assume the escape was a
                                                // mistake, etc.
                                                consumed_characters = 1;
                                                break;
                                            }
                                        }

                                        // Cursor Up
                                        'A' if s.parse::<u16>().is_ok() && nums.len() == 0 => {
                                            /* Unsupported */
                                            break;
                                        }

                                        // Cursor Down
                                        'B' if s.parse::<u16>().is_ok() && nums.len() == 0 => {
                                            /* Unsupported */
                                            break;
                                        }

                                        // Cursor Forward
                                        'C' if s.parse::<u16>().is_ok() && nums.len() == 0 => {
                                            /* Unsupported */
                                            break;
                                        }

                                        // Cursor Backward
                                        'D' if s.parse::<u16>().is_ok() && nums.len() == 0 => {
                                            /* Unsupported */
                                            break;
                                        }

                                        // Report Device Code
                                        'c' if s.chars().last() == Some('0') && s.len() >= 2 => {
                                            let _ = s.pop();

                                            if let Ok(dev_code) = s.parse::<u16>() {
                                                /* Unsupported */
                                                break;
                                            } else {
                                                // On error, assume the escape
                                                // was a mistake, etc.
                                                consumed_characters = 1;
                                                break;
                                            }
                                        }

                                        // Report Cursor Position
                                        'R' if s.parse::<u16>().is_ok() && nums.len() == 1 => {
                                            let _row = nums[0];
                                            let _col = s.parse::<u16>().unwrap();

                                            /* Unsupported */
                                            break;
                                        }

                                        // Cursor Home
                                        'H' if s.parse::<u16>().is_ok() && nums.len() == 1 => {
                                            let _row = nums[0];
                                            let _col = s.parse::<u16>().unwrap();

                                            /* Unsupported */
                                            break;
                                        }

                                        // Force Cursor Position
                                        'f' if s.parse::<u16>().is_ok() && nums.len() == 1 => {
                                            let _row = nums[0];
                                            let _col = s.parse::<u16>().unwrap();

                                            /* Unsupported */
                                            break;
                                        }

                                        // Scroll part of the screen
                                        'r' if s.parse::<u16>().is_ok() && nums.len() == 1 => {
                                            let _start_row = nums[0];
                                            let _end_row = s.parse::<u16>().unwrap();

                                            /* Unsupported */
                                            break;
                                        }

                                        // Set Key Definition
                                        '"' if nums.len() == 1 => loop {
                                            let mut string = String::new();
                                            let mut last = next!();

                                            while last != '"' {
                                                string.push(last);
                                                last = next!();
                                            }

                                            // The next character must be a p
                                            // otherwise this is all invalid.
                                            if next!() != 'p' {
                                                // Assume this was all a
                                                // mistake, etc.
                                                consumed_characters = 1;
                                                break;
                                            }

                                            let _key = nums[0];
                                            let _string = string;

                                            /* Unsupported */
                                            break;
                                        }

                                        // Set Attribute Mode (SGR)
                                        // Details for this section are taken
                                        // from [here](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)
                                        'm' if s.parse::<u16>().is_ok() => {
                                            use super::Color::*;
                                            use tui::style::Modifier;
                                            nums.push(s.parse::<u16>().unwrap());

                                            let to_color = |num: u16, sub: u16| {
                                                match num - sub {
                                                    0 => Black,
                                                    1 => Red,
                                                    2 => Green,
                                                    3 => Yellow,
                                                    4 => Blue,
                                                    5 => Magenta,
                                                    6 => Cyan,
                                                    7 => Gray,

                                                    60 => DarkGray,
                                                    61 => LightRed,
                                                    62 => LightGreen,
                                                    63 => LightYellow,
                                                    64 => LightBlue,
                                                    65 => LightMagenta,
                                                    66 => LightCyan,
                                                    67 => White,

                                                    _ => unreachable!(),
                                                    // _ => Indexed(num as u8), // TODO: don't as cast
                                                }
                                            };

                                            let mut iter = nums.iter().peekable();
                                            let mut s = current_style.clone();
                                            loop {
                                                if let Some(attr) = iter.next() {
                                                    macro_rules! at {
                                                        ($a:expr) => {s = s.modifier(s.modifier | $a);};
                                                    }

                                                    macro_rules! atc {
                                                        ($a:expr) => {s = s.modifier(s.modifier & !($a));};
                                                    }

                                                    match attr {
                                                        0 => s = s.modifier(Modifier::empty()),
                                                        1 => at!(Modifier::BOLD),
                                                        2 => at!(Modifier::DIM),
                                                        3 => at!(Modifier::ITALIC),
                                                        4 => at!(Modifier::UNDERLINED),
                                                        5 => at!(Modifier::SLOW_BLINK),
                                                        6 => at!(Modifier::RAPID_BLINK),
                                                        7 => at!(Modifier::REVERSED),
                                                        8 => {},
                                                        9 => at!(Modifier::CROSSED_OUT),

                                                        21 => atc!(!Modifier::BOLD),
                                                        22 => atc!(!Modifier::DIM),
                                                        23 => atc!(!Modifier::ITALIC),
                                                        24 => atc!(!Modifier::UNDERLINED),
                                                        25 => {
                                                            atc!(!Modifier::SLOW_BLINK);
                                                            atc!(!Modifier::RAPID_BLINK);
                                                        },
                                                        26 => {},
                                                        27 => atc!(Modifier::REVERSED),
                                                        28 => {},
                                                        29 => atc!(Modifier::CROSSED_OUT),

                                                        30..=37 |   90..=97 => s = s.fg(to_color(*attr, 30)),
                                                        40..=47 | 100..=107 => s = s.bg(to_color(*attr, 40)),

                                                        // 8-bit lookup:
                                                        38 if Some(&&5) == iter.peek() && {
                                                            assert_eq!(Some(&5), iter.next());
                                                            iter.peek().is_some()
                                                        } => {
                                                            // TODO: don't as cast here.
                                                            s = s.fg(Indexed(*iter.next().unwrap() as u8));
                                                        },

                                                        48 if Some(&&5) == iter.peek() && {
                                                            assert_eq!(Some(&5), iter.next());
                                                            iter.peek().is_some()
                                                        } => {
                                                            // TODO: don't as cast here.
                                                            s = s.bg(Indexed(*iter.next().unwrap() as u8));
                                                        },

                                                        // 24-bit lookup:
                                                        38 if Some(&&2) == iter.peek() && {
                                                            assert_eq!(Some(&2), iter.next());
                                                            iter.size_hint().0 >= 3
                                                        } => {
                                                            let (r, g, b) = (*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap());
                                                            // TODO: don't as cast here.
                                                            s = s.fg(Rgb(r as u8, g as u8, b as u8));
                                                        },

                                                        48 if Some(&&2) == iter.peek() && {
                                                            assert_eq!(Some(&2), iter.next());
                                                            iter.size_hint().0 >= 3
                                                        } => {
                                                            let (r, g, b) = (*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap());
                                                            // TODO: don't as cast here.
                                                            s = s.bg(Rgb(r as u8, g as u8, b as u8));
                                                        },

                                                        _ => {
                                                            // Assume this was all a mistake.
                                                            consumed_characters = 1;
                                                            break 'c;
                                                        }
                                                    }
                                                } else {
                                                    break;
                                                }
                                            }

                                            *current_style = s;
                                        }

                                        // Note: this misses CSI m (TODO).
                                        // fix by adding a check for 'm' at the '[' level

                                        _ => {
                                            // If we get anything else, assume
                                            // this was all a mistake.
                                            consumed_characters = 1;
                                            break;
                                        }
                                    }

                                    last = next!();
                                }
                            }
                        }

                        _ => {
                            // If we got something that isn't valid in this
                            // position, we'll assume that the escape + '['
                            // was here by mistake and that the '[' and the
                            // character following it were meant to be printed.
                            consumed_characters -= 2;
                        }
                    }

                    _ => {
                        // If we got something else, we'll assume that the
                        // escape was here by mistake and that the character
                        // after it was meant to be printed.
                        consumed_characters -= 1;
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
                let text = TuiText::<'t>::styled(
                    Cow::Borrowed(s).into_owned(),
                    current_style.clone(),
                );

                out.last_mut().unwrap().push(text);

                return Ok(())
            },
        }
    }
}

#[cfg(test)]
mod ansi_tests {
    use super::*;
    use pretty_assertions::assert_eq as eq;

    macro_rules! l {
        (
            $((

                $($l:literal:$s:expr),* $(,)?
            )),* $(,)?
        ) => {
            vec![$(
                vec![$(
                    TuiText::Styled(Cow::Owned($l.to_string()), $s)

                ),*]
            ),*]
        };
    }

    mod no_escapes {
        use super::*;

        #[test]
        fn simple() {
            let mut s = Style::default();
            let mut out = Vec::new();

            ansi_string_to_tui_text("hello\nfriends", &mut s, &mut out).unwrap();

            eq!(out, l![
                ("hello\n": s),
                ("friends": s),
            ]);
        }

        #[test]
        fn with_interruptions() {
            let mut s = Style::default();
            let mut out = Vec::new();

            ansi_string_to_tui_text("hey ", &mut s, &mut out).unwrap();
            ansi_string_to_tui_text(" there", &mut s, &mut out).unwrap();
            ansi_string_to_tui_text("\n\n\nyo\n👋", &mut s, &mut out).unwrap();
            ansi_string_to_tui_text("\n", &mut s, &mut out).unwrap();
            ansi_string_to_tui_text("\n.\n", &mut s, &mut out).unwrap();
            ansi_string_to_tui_text("Sphinx of black quartz, judge my vow.", &mut s, &mut out).unwrap();

            eq!(out, l![
                ("hey ": s, " there": s, "\n": s),
                ("\n": s),
                ("\n": s),
                ("yo\n": s),
                ("👋": s, "\n": s),
                ("": s, "\n": s),
                (".\n": s),
                ("": s, "Sphinx of black quartz, judge my vow.": s),
            ]);
        }
    }

    mod colors {
        use super::*;

        #[test]
        fn red() {
            let mut s = Style::default();
            let mut out = Vec::new();

            ansi_string_to_tui_text("\x1B[1;31mhello", &mut s, &mut out).unwrap();

            eq!(out, l![("hello": s)]);
            eq!(s, Style::default().fg(Color::Red).modifier(Modifier::BOLD));
        }
    }
}
