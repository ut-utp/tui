//! TODO!

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use std::marker::PhantomData;

use lc3_isa::{Addr, Instruction, Reg, Word};

#[allow(explicit_outlives_requirement)]
#[derive(Debug, Clone, PartialEq)]
pub struct Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>),
{
    title: String,
    func: F,
    colour: Color,
    _p: PhantomData<(&'int (), &'a I, &'a O, C)>,
}

impl<'a, 'int, C, I, O, F> Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>),
{
    fn new_from_func(func:F) -> Self{
        Self {
            title: String::from(""),
            func,
            colour: Color::Red,
            _p: PhantomData,
        }
    }

    fn new_sans_colour(title:String, func:F) -> Self {
        Self {
            title,
            func,
            colour: Color::Red,
            _p: PhantomData,
        }
    }

    fn new_sans_title(colour:Color, func:F) -> Self {
        Self {
            title: String::from(""),
            func,
            colour,
            _p: PhantomData,
        }
    }

    fn new(title:String, colour:Color, func:F) -> Self {
        Self {
            title,
            func,
            colour,
            _p: PhantomData,
        }
    }
}

impl<'a, 'int, C, I, O, F> TuiWidget for Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>),
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B, F> Widget<'a, 'int, C, I, O, B> for Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r TuiData<'a, 'int, C, I, O>),
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let text = [
            TuiText::styled(self.title.clone(), Style::default().fg(self.colour)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,
            _ => false,
        }
    }
}
