//! TODO!

use super::widget_impl_support::*;

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

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Clock
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {

        let text = [
            TuiText::styled("Clock: \n", Style::default().fg(c!(Name))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf);


        let clock_val = data.sim.get_clock();

        let mut s0 = String::from("");
        s0.push_str(&format!("{}ms\n", clock_val));

        let text = [TuiText::styled(s0, Style::default().fg(c!(Data)))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.render(area, buf);
    }


    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
