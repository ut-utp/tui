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

        const fn with_control(code: KeyCode) -> KeyEvent {
            KeyEvent { code, modifiers: KeyModifiers::CONTROL }
        }

        const UP: KeyEvent = with_control(KeyCode::Up);
        const DOWN: KeyEvent = with_control(KeyCode::Down);
        const LEFT: KeyEvent = with_control(KeyCode::Left);
        const RIGHT: KeyEvent = with_control(KeyCode::Right);

        // const UP: KeyEvent = KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::CONTROL };
        // const DOWN: KeyEvent = KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::CONTROL };
        // const LEFT: KeyEvent = KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::CONTROL };
        // const RIGHT: KeyEvent = KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::CONTROL };

        match event {
            r @ Resize(_, _) => {
                self.areas_valid = false;
                self.propagate_to_all(r, data)
            },

            // // We'll use the Key events to tell when we're focused.
            // Focus(FocusEvent::GotFocus) => { },

            Focus(FocusEvent::GotFocus) => {
                // The only time we should be told that we've gotten focus
                // without getting key or mouse events that confirm this is the
                // first time we're focused. In this case if we have elements,
                // we'll set our first one as focused:
                self.focused = if !self.widgets.is_empty() {
                    Some(0)
                } else {
                    None
                };

                self.propagate_to_focused(event, data)
            },
            Focus(FocusEvent::LostFocus) => {
                drop(self.focused.take());
                self.propagate_to_focused(event, data);

                true
            },

            Mouse(e) => {
                use MouseEvent::*;
                use MouseButton::*;
                // match e {

                // }
                todo!()
            }

            Key(e) => match e {
                UP | DOWN | LEFT | RIGHT => {
                    let dir = extract_direction_from_layout(&self.layout);

                    use Direction::*;
                    // We currently don't wrap because we can't; if we were to
                    // wrap our children would never _not_ handle these events
                    // and we'd never be able to break out of nested Widgets.
                    match (dir, e) {
                        (Vertical, UP) | (Vertical, DOWN) |
                        (Horizontal, LEFT) | (Horizontal, RIGHT) => {
                            // First let's check if our currently focused thing
                            // can handle this:
                            if self.propagate_to_focused(event, data) {
                                true
                            } else {
                                // If it couldn't, we're up:

                                // (Note: we do nothing if we don't have a
                                // currently focused widget; because of the
                                // handling of FocusEvent::GotFocus above we
                                // take that to mean that we just don't have
                                // any widgets)
                                if let Some(focused_idx) = self.focused {
                                    if let Some(new_idx) = match e {
                                        UP | LEFT => focused_idx.checked_sub(1),
                                        DOWN | RIGHT => focused_idx.checked_add(1),
                                        _ => unreachable!(), // Alas..
                                    }
                                    .filter(|i| (0..self.widgets.len()).contains(i)) {/* {
                                        if (0..self.widgets.len()).contains(&new_idx) {
                                            // If we can handle the event (i.e.
                                            // if we're not already at an edge),
                                            // do so:
                                            self.propagate_to_focused(Focus(FocusEvent::LostFocus), data);
                                            self.focused = Some(new_idx);
                                            self.propagate_to_focused(Focus(FocusEvent::GotFocus), data);
                                            true
                                        } else {
                                            false
                                        }
                                    } */
                                        // If we can handle the event (i.e. if
                                        // we're not already at an edge), do so:
                                        self.propagate_to_focused(Focus(FocusEvent::LostFocus), data);
                                        self.focused = Some(new_idx);
                                        self.propagate_to_focused(Focus(FocusEvent::GotFocus), data);
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    // Test our assumptions:
                                    assert!(self.widgets.is_empty());

                                    // Parents should actually handle the event,
                                    // so we return false.
                                    false
                                }

                            }
                        },
                        // (Vertical, DOWN) => {

                        // },
                        // (Horizontal, LEFT) => {

                        // },
                        // (Horizontal, RIGHT) => {

                        // },

                        // If the key event doesn't match us, send it below
                        // and return.
                        (Vertical, LEFT) | (Vertical, RIGHT) |
                        (Horizontal, UP) | (Horizontal, DOWN) => {
                            self.propagate_to_focused(event, data)
                        }
                        _ => unreachable!(), // Unnamed union types.. we long for ye
                    }
                },

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
