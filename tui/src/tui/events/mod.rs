//! TODO!

use super::Res as Result;

pub use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use crossterm::ErrorKind as CrosstermError;

/// All events that our event threads produce.
#[derive(Debug)]
#[non_exhaustive]
pub(in crate::tui) enum Event {
    Error(CrosstermError),
    Tick,
    ActualEvent(CrosstermEvent),
    #[doc(hidden)]
    FlushEventsBarrier(u8),
}

/// The only events that actually make their to Widgets.
///
/// All other events (i.e. the others in the [`Event`] enum) are handled
/// "internally".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum WidgetEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Focus(FocusEvent),
    Update, // Just another name for Tick.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FocusEvent {
    GotFocus,
    LostFocus
}

impl From<CrosstermEvent> for WidgetEvent {
    fn from(event: CrosstermEvent) -> Self {
        use CrosstermEvent::*;
        match event {
            Key(k) => WidgetEvent::Key(k),
            Mouse(m) => WidgetEvent::Mouse(m),
            Resize(x, y) => WidgetEvent::Resize(x, y),
        }
    }
}

