//! Simple widget that tries to display a string.

use super::widget_impl_support::*;

use std::marker::PhantomData;

// TODO: handle lines properly...
// right now we assume each Text element is its own line which is Not True (for
// good reasons too).

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text<Wt: WidgetTypes, F>
where
    F: for<'r> Fn(&'r Data<Wt>) -> &'r [TuiText<'r>],
{
    func: F,
    offset: u16,
    follow: bool,
    _p: PhantomData<Wt>,
}

impl<Wt: WidgetTypes, F> Text<Wt, F>
where
    F: for<'r> Fn(&'r Data<Wt>) -> &'r [TuiText<'r>],
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

impl<'a, 'int, Wt: WidgetTypes, F> Widget<Wt> for Text<Wt, F>
where
    F: for<'r> Fn(&'r Data<Wt>) -> &'r [TuiText<'r>],
{

    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        let text = (self.func)(data);

        if self.follow {
            self.offset = (self.func)(data).len() as u16;
            self.offset = self.offset.saturating_sub(area.height - 1);
        }

        // TODO: allow parameterization of this in the usual way.
        let para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .scroll(self.offset)
            .wrap(true);

        para.render(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
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
