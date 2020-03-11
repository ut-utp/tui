//! Simple widget that tries to display a string.

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use std::marker::PhantomData;

// TODO: handle lines properly...
// right now we assume each Text element is its own line which is Not True (for
// good reasons too).

#[allow(explicit_outlives_requirements)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r [TuiText<'r>],
{
    func: F,
    offset: u16,
    follow: bool,
    _p: PhantomData<(&'int (), &'a I, &'a O, C)>,
}

impl<'a, 'int, C, I, O, F> Text<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r [TuiText<'r>],
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            offset: 0,
            follow: false,
            _p: PhantomData,
        }
    }
}

impl<'a, 'int, C, I, O, F> TuiWidget for Text<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r [TuiText<'r>],
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, F, B> Widget<'a, 'int, C, I, O, B> for Text<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r [TuiText<'r>],
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let text = (self.func)(data);

        if self.follow {
            self.offset = (self.func)(data).len() as u16;
            self.offset = self.offset.saturating_sub(area.height - 1);
        }

        // TODO: allow parameterization of this in the usual way.
        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .scroll(self.offset)
            .wrap(true);

        para.draw(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            // Accept focus!
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,

            Key(KeyEvent { code: KeyCode::Up, modifiers: EMPTY }) |
            Mouse(MouseEvent::ScrollUp(_, _, _)) => {
                self.follow = false;
                self.offset = self.offset.saturating_sub(1);
                true
            }
            Key(KeyEvent { code: KeyCode::Down, modifiers: EMPTY }) |
            Mouse(MouseEvent::ScrollDown(_, _, _)) => {
                self.follow = false;
                self.offset = self.offset.saturating_add(1);
                true
            }

            Key(KeyEvent { code: KeyCode::PageUp, modifiers: EMPTY }) => {
                // TODO: actually use the current page size (i.e. height) for this
                self.follow = false;
                self.offset = self.offset.saturating_sub(50);
                true
            }
            Key(KeyEvent { code: KeyCode::PageDown, modifiers: EMPTY }) => {
                // TODO: actually use the current page size (i.e. height) for this
                self.follow = false;
                self.offset = self.offset.saturating_add(50);
                true
            }

            Key(KeyEvent { code: KeyCode::Home, modifiers: EMPTY }) => {
                self.follow = false;
                self.offset = 0;
                true
            }
            Key(KeyEvent { code: KeyCode::End, modifiers: EMPTY }) => {
                self.follow = true;
                self.offset = (self.func)(data).len() as u16;
                true
            }

            _ => false,
        }
    }
}
