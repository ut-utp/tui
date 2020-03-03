//! TODO!

use crate::tui::TuiData;
use crate::tui::events::{WidgetEvent, FocusEvent};
use super::single::SingleWidget;
use super::{TuiWidget, Widget};

use lc3_application_support::io_peripherals::{InputSink, OutputSource};
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::buffer::Buffer;
use tui::layout::{Layout, Direction, Constraint, Rect};
use tui::widgets::Block;
use crossterm::event::{MouseEvent, MouseButton, KeyEvent, KeyCode, KeyModifiers};


/// A bunch of Widgets that split the are they are given in *one* direction. In
/// other words, a horizontal or vertical set of widgets.
///
/// Nest these like you'd nest [`Layout`]s for more complicated arrangements.
///
/// [`Layout`]: tui::layout::Layout
#[allow(explicit_outlives_requirements)]
pub struct Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    /// The widgets within.
    widgets: Vec<SingleWidget<'a, 'int, C, I, O, B>>,
    /// Overall `Layout` for the Widgets. This is used to set the margins and
    /// direction of the Widgets; any constraints given will be ignored.
    layout: Layout,
    /// Whether or not the cached `Rect` in each `SingleWidget` is still valid.
    areas_valid: bool,
    /// The index of the widget to dispatch events to.
    focused: Option<usize>,
    previously_focused: Option<usize>,
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
            areas_valid: false,
            focused: None,
            previously_focused: None,
        }
    }

    // `block` is optional; widgets that wish to manage their block themselves
    // (or don't want a `Block`) are free to not use this.
    //
    // Blocks that we manage will have their borders change color when focused.
    //
    // We also send the appropriate event when widgets are focused so widgets
    // that choose to manage their own `Block` can provide similar
    // functionality.
    pub fn add_widget<W>(&mut self, constraint: Constraint, widget: W, block: Option<Block<'a>>) -> &mut Self
    where
        W: Widget<'a, 'int, C, I, O, B> + 'a
    {
        self.widgets.push(SingleWidget::new(constraint, Box::new(widget), block));
        self.areas_valid = false; // We need to recalculate positions now!

        self
    }

    fn update_areas(&mut self, area: Rect) {
        if !self.areas_valid {
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
                self.widgets[idx].area = *rect;
            }

            self.areas_valid = true;
        }
    }

    // Returns whether *any* widget handled the event.
    //
    // With this function it is possible that more than one widget handles the
    // event.
    fn propagate_to_all(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        self.widgets.iter_mut().fold(false, |b, w| b | w.widget.update(event, data))
    }

    // Returns whether *any* widget handled the event.
    //
    // With this function at most one widget will handle the event.
    fn propagate_until_handled(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        self.widgets.iter_mut().any(|w| w.widget.update(event, data))
    }

    // Returns whether or not the focused Widget handled the event.
    //
    // If there is no focused widget, this returns false.
    fn propagate_to_focused(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        if let Some(idx) = self.focused {
            self.widgets[idx].widget.update(event, data)
        } else {
            false
        }
    }
}

const fn with_control(code: KeyCode) -> KeyEvent {
    KeyEvent { code, modifiers: KeyModifiers::CONTROL }
}

const UP: KeyEvent = with_control(KeyCode::Up);
const DOWN: KeyEvent = with_control(KeyCode::Down);
const LEFT: KeyEvent = with_control(KeyCode::Left);
const RIGHT: KeyEvent = with_control(KeyCode::Right);

impl<'a, 'int, C, I, O, B> Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn handle_focus_key_event(&mut self, event: KeyEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::{Focus, Key};

        if let UP | DOWN | LEFT | RIGHT = event { } else {
            panic!("Called the focus key event handler without a focus key event!")
        }

        let dir = extract_direction_from_layout(&self.layout);

        // We currently don't wrap because we can't; if we were to wrap our
        // children would never _not_ handle these events and we'd never be able
        // to break out of nested Widgets.
        use Direction::*;
        match (dir, event) {
            (Vertical, UP) | (Vertical, DOWN) |
            (Horizontal, LEFT) | (Horizontal, RIGHT) => {
                // First let's check if our focused thing can handle this:
                if self.propagate_to_focused(Key(event), data) {
                    true
                } else {
                    // If it couldn't, we're up:

                    // (Note: we do nothing if we don't have a currently focused
                    // widget; because of the handling of FocusEvent::GotFocus
                    // we take that to mean that we just don't have any widgets)
                    if let Some(focused_idx) = self.focused {
                        let mut focused_idx = focused_idx;
                        let new_idx = loop {
                            if let Some(n) = match event {
                                UP | LEFT => focused_idx.checked_sub(1),
                                DOWN | RIGHT => focused_idx.checked_add(1),
                                _ => unreachable!(), // Obvious to us; not rustc :-/
                            }
                            .filter(|i| (0..self.widgets.len()).contains(i)) {
                                focused_idx = n;

                                let accepted = self.widgets[n].widget.update(Focus(FocusEvent::GotFocus), data);
                                if accepted {
                                    break Some(n)
                                } else {
                                    self.widgets[n].widget.update(Focus(FocusEvent::LostFocus), data);
                                    continue; // try again
                                }
                            } else {
                                // Out of bounds? We've run out.
                                break None
                            }
                        };

                        match new_idx {
                            Some(i) => {
                                self.propagate_to_focused(Focus(FocusEvent::LostFocus), data);
                                // Already focused so no need to sent the focused
                                // event.
                                self.focused = Some(i);
                                self.propagate_to_focused(Key(event), data)
                            }
                            None => false,
                        }
                    } else {
                        // Test our assumption:
                        assert!(self.widgets.is_empty());

                        // Parents should actually handle the event, so we
                        // return false.
                        false
                    }
                }
            },

            // If the key event doesn't match us, send it below
            // and return.
            (Vertical, LEFT) | (Vertical, RIGHT) |
            (Horizontal, UP) | (Horizontal, DOWN) => {
                self.propagate_to_focused(Key(event), data)
            }
            _ => unreachable!(), // Unnamed union types.. we long for ye
        }
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
        unreachable!("This should never be called. Call `lc3_tui::Widget::draw` instead.")
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Widgets<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, sim: &C, rect: Rect, buf: &mut Buffer) {
        self.update_areas(rect);

        for (idx, sw) in self.widgets.iter_mut().enumerate() {
            sw.draw(sim, buf, self.focused.map(|f| f == idx).unwrap_or(false))
        }
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        // todo!()

        use WidgetEvent::*;

        match event {
            r @ Resize(_, _) => {
                self.areas_valid = false;
                self.propagate_to_all(r, data)
            },

            Focus(FocusEvent::GotFocus) => {
                // The only time we should be told that we've gotten focus
                // without getting key or mouse events that confirm this is the
                // first time we're focused. In this case if we have elements,
                // we'll set our first one as focused:
                self.focused = if !self.widgets.is_empty() {
                    // TODO: is this more or less confusing to users (than just
                    // starting at 0)?
                    Some(self.previously_focused)
                } else {
                    None
                };

                self.propagate_to_focused(event, data)
            },
            Focus(FocusEvent::LostFocus) => {
                let _ = self.propagate_to_focused(event, data);

                if let Some(idx) = self.focused.take() {
                    self.previously_focused = Some(idx);
                }

                true
            },

            // TODO: should we allow us to 'focus' on things that don't actually
            // accept focus when clicking on them? For now, let's say yes.
            Mouse(e) => {
                use MouseEvent::*;

                match e {
                    // We don't care about buttons, up or down, or modifiers for
                    // focus purposes; all can change the currently focused
                    // widget.
                    Down(_, col, row, _) | Up(_, col, row, _) => {
                        // It's possible that we use an outdated area set here
                        // (as in, we don't call `update_areas` here), which is
                        // actually fine: we assume that the user choose a place
                        // to click based on the last drawn frame anyways.
                        let new_focused_idx = self.widgets.iter()
                            .enumerate()
                            .filter(|(_, w)| w.area.contains(col, row))
                            .map(|(idx, _)| idx)
                            .next();

                        if self.focused == new_focused_idx {
                            // If there isn't a change in focus, propagate the
                            // event and carry on.
                            self.propagate_to_focused(event, data)
                        } else {
                            if let Some(_) = new_focused_idx {
                                let _ = self.propagate_to_focused(Focus(FocusEvent::LostFocus), data);

                                // Switch the focused widget and give it the event:
                                self.focused = new_focused_idx;
                                let _ = self.propagate_to_focused(Focus(FocusEvent::GotFocus), data);
                                self.propagate_to_focused(event, data)
                            } else {
                                // If we don't have a focused valid new focused
                                // widget, keep the current focused widget and
                                // drop the event.
                                false
                            }
                        }
                    },
                    Drag(_, _, _, _) => { /* ignore drag events! */ false },
                    ScrollDown(_, _, _) | ScrollUp(_, _, _) => {
                        // Just propagate scroll events:
                        self.propagate_to_focused(event, data)
                    }
                }
            }

            Key(e) => match e {
                UP | DOWN | LEFT | RIGHT => self.handle_focus_key_event(e, data),

                // For events that don't change the focus, just propagate:
                _ => self.propagate_to_focused(event, data),
            }
        }

        // invalidate (recursively) on resize events (i.e. propagate the resize
        // event)

        // use clicked events to update the currently focused thing
        // (propagate these as well since what's under us might not be a single
        // widget)
        // additionally, send out focused/lost focus events on changes to the
        // currently focused thing

        // dispatch key events to the currently focused thing
    }
}

/// `Layout.direction` is private and we'd rather not clutter out API by making
/// users tell us the [`Direction`] they want _and_ specify a [`Layout`]
/// (there's still a need to specify a layout because layouts include other
/// things like margins).
///
/// So, we do this trick (leveraging the fact that [`Eq`] is implemented for
/// [`Layout`]) to extract the direction.
///
/// ['Direction`]: tui::layout::Direction
/// ['Layout`]: tui:layout::Layout
/// ['Eq`]: std::cmp::Eq
#[inline]
fn extract_direction_from_layout(l: &Layout) -> Direction {
    let guess = l.clone().direction(Direction::Vertical);

    if guess == *l {
        Direction::Vertical
    } else {
        Direction::Horizontal
    }
}

trait Contains {
    fn contains(&self, col: u16, row: u16) -> bool;
}

impl Contains for Rect {
    fn contains(&self, col: u16, row: u16) -> bool {
        (self.left()..=self.right()).contains(&col) &&
        (self.top()..=self.bottom()).contains(&row)
    }
}
