//! A widget that does nothing but occupy space.
//!
//! Useful for testing and for blank spaces.

use super::widget_impl_support::*;
use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Clock {
    pub focusable: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Clock {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl TuiWidget for Clock {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        // Do nothing!
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Clock
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
       
        let text = [
            TuiText::styled("Clock: \n", Style::default().fg(Color::Gray)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);


        let clock_val = data.sim.get_clock();

        let mut s0 = String::from("");
        s0.push_str(&format!("{}ms\n", clock_val));

        let text = [TuiText::styled(s0, Style::default().fg(Color::LightGreen))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.draw(area, buf);








    }


    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
