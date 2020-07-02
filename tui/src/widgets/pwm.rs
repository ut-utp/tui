//! TODO!

use super::widget_impl_support::*;

use lc3_traits::peripherals::pwm::{PwmPin, PwmState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pwm {
    pub focusable: bool,
}

impl Default for Pwm {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Pwm {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Pwm
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let pwm_state = data.sim.get_pwm_states();
        let pwm_configs = data.sim.get_pwm_config();


        let text = [
            TuiText::styled("PWM 0: \n", Style::default().fg(c!(Name))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf);

        let mut s0 = String::from("");

        let p0 = match pwm_state[PwmPin::P0]{
            PwmState::Disabled => {
                let disabled_string = "Disabled";
                s0.push_str(&format!("{}\n", disabled_string));
            },
            PwmState::Enabled(time) => {
                s0.push_str(&format!(
                    "{:#018b} {:#06x} {:#05}\n", pwm_configs[PwmPin::P0], pwm_configs[PwmPin::P0], pwm_configs[PwmPin::P0]
                ));

            }

        };


        let text = [TuiText::styled(s0, Style::default().fg(c!(Data)))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.render(area, buf);




        let text = [
            TuiText::styled("PWM 1: \n", Style::default().fg(c!(Name))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area1 = increment(40, Axis::X, area);
        para.render(area1, buf);



        let mut s1 = String::from("");
        let p1 = match pwm_state[PwmPin::P1]{
            PwmState::Disabled => {
                let disabled_string = "Disabled";
                s1.push_str(&format!("{}\n", disabled_string));
            },
            PwmState::Enabled(time) => {
                s1.push_str(&format!(
                    "{:#018b} {:#06x} {:#05}\n", pwm_configs[PwmPin::P1], pwm_configs[PwmPin::P1], pwm_configs[PwmPin::P1]
                ));

            }

        };

        let text = [TuiText::styled(s1, Style::default().fg(c!(Data)))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area2 = increment(10, Axis::X, area1);
        para.render(area2, buf);



    }



    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
