//! Simple widget that tries to display a string.

use super::widget_impl_support::*;

use tui::widgets::{Text, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextBlock<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r String,
{
    func: F,
    offset: u16,
    _p: PhantomData<(&'int (), &'a I, &'a O, C)>,
}

impl<'a, 'int, C, I, O, F> TextBlock<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r String,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            offset: 0,
            _p: PhantomData,
        }
    }
}

impl<'a, 'int, C, I, O, F> TuiWidget for TextBlock<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r String,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, F, B> Widget<'a, 'int, C, I, O, B> for TextBlock<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>) -> &'r String,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let text = [Text::raw((self.func)(data))];

        // TODO: allow parameterization of this in the usual way.
        let para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .scroll(self.offset)
            .wrap(true);

        para.draw(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;

        match event {
            // Accept focus!
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            // Mouse(MouseEvent::Up(_, _, _, _)) => true,
            // Mouse(MouseEvent::Down(_, _, _, _)) => true,

            Mouse(MouseEvent::ScrollUp(_, _, _)) => {
                self.offset = self.offset.saturating_sub(1);
                true
            }
            Mouse(MouseEvent::ScrollDown(_, _, _)) => {
                self.offset = self.offset.saturating_add(1);
                true
            }

            _ => false,
        }
    }
}
