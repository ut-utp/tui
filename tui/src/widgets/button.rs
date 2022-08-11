//! TODO!

use super::widget_impl_support::*;

use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
pub struct SimButton<Wt: WidgetTypes, F>
where
    F: for<'r> Fn(&'r mut ControlTy<Wt>),
{
    title: String,
    func: F,
    colour: Colour,
    _p: PhantomData<Wt>,
}

impl<Wt: WidgetTypes, F> SimButton<Wt, F>
where
    F: for<'r> Fn(&'r mut ControlTy<Wt>),
{
    pub fn new_from_func(func:F) -> Self{
        Self {
            title: String::from(""),
            func,
            colour: Colour::Red,
            _p: PhantomData,
        }
    }

    pub fn new_sans_colour(title:String, func:F) -> Self {
        Self {
            title,
            func,
            colour: Colour::Red,
            _p: PhantomData,
        }
    }

    pub fn new_sans_title(colour:Colour, func:F) -> Self {
        Self {
            title: String::from(""),
            func,
            colour,
            _p: PhantomData,
        }
    }

    pub fn new(title:String, colour:Colour, func:F) -> Self {
        Self {
            title,
            func,
            colour,
            _p: PhantomData,
        }
    }
}

impl<Wt: WidgetTypes, F> Widget<Wt> for SimButton<Wt, F>
where
    F: for<'r> Fn(&'r mut ControlTy<Wt>),
{
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        let text = [
            TuiText::styled(self.title.clone(), Style::default().fg(self.colour)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Center)
            .wrap(true);

        para.render(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut Data<Wt>, terminal: &mut Terminal<Wt::Backend>) -> bool {
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
