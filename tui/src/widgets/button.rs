//! TODO!

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use std::marker::PhantomData;

use lc3_isa::{Addr, Instruction, Reg, Word};

#[allow(explicit_outlives_requirements)]
#[derive(Debug, Clone, PartialEq)]
pub struct Sim_Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r mut C),
{
    title: String,
    func: F,
    colour: Color,
    _p: PhantomData<(&'int (), &'a I, &'a O, C)>,
}

impl<'a, 'int, C, I, O, F> Sim_Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r mut C),
{
    pub fn new_from_func(func:F) -> Self{
        Self {
            title: String::from(""),
            func,
            colour: Color::Red,
            _p: PhantomData,
        }
    }

    pub fn new_sans_colour(title:String, func:F) -> Self {
        Self {
            title,
            func,
            colour: Color::Red,
            _p: PhantomData,
        }
    }

    pub fn new_sans_title(colour:Color, func:F) -> Self {
        Self {
            title: String::from(""),
            func,
            colour,
            _p: PhantomData,
        }
    }

    pub fn new(title:String, colour:Color, func:F) -> Self {
        Self {
            title,
            func,
            colour,
            _p: PhantomData,
        }
    }
}

impl<'a, 'int, C, I, O, F> TuiWidget for Sim_Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r mut C),
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B, F> Widget<'a, 'int, C, I, O, B> for Sim_Button<'a, 'int, C, I, O, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    F: for<'r> Fn(&'r mut C),
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let text = [
            TuiText::styled(self.title.clone(), Style::default().fg(self.colour)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Center)
            .wrap(true);

        para.draw(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => {
                (self.func)(data.sim);
                true
            }
            _ => false,
        }
    }
}
