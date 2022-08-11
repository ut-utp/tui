//! TODO!

use super::widget_impl_support::*;

use lc3_traits::peripherals::timers::{TimerId, TimerState, TimerMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timers {
    pub focusable: bool,
}

impl Default for Timers {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Timers {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl<Wt: WidgetTypes> Widget<Wt> for Timers {
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        let timer_state = data.sim.get_timer_states();
        let timer_modes = data.sim.get_timer_modes();

        let text = [
            TuiText::styled("Timer 0: \n", Style::default().fg(c!(Name))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf);


        let mut s0 = String::from("");

        let t0 = match timer_state[TimerId::T0] {
            TimerState::Disabled => {
                let disabled_string = "Disabled";
                s0.push_str(&format!("{}\n", disabled_string))
            }
            TimerState::WithPeriod(period) => {
                match timer_modes[TimerId::T0]{
                        TimerMode::Repeated => {
                            s0.push_str(&format!(
                                "Repeated: {:#018b} {:#06x} {:#05}\n",
                                period,
                                period,
                                period,
                            ))
                    },
                        TimerMode::SingleShot => {
                            s0.push_str(&format!(
                                "Single: {:#018b} {:#06x} {:#05}\n",
                                period,
                                period,
                                period,
                            ))
                        }

                };


            },

        };

        let text = [TuiText::styled(s0, Style::default().fg(c!(Data)))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.render(area, buf);

        let text = [
            TuiText::styled("Timer 1: \n", Style::default().fg(c!(Name))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(40, Axis::X, area);
        para.render(area, buf);

        let mut s1 = String::from("");

        let t1 = match timer_state[TimerId::T1] {
            TimerState::Disabled => {
                let disabled_string = "Disabled";
                s1.push_str(&format!("{}\n", disabled_string))
            }
            TimerState::WithPeriod(period) => {
                match timer_modes[TimerId::T0]{
                        TimerMode::Repeated => {
                            s1.push_str(&format!(
                                "Repeated: {:#018b} {:#06x} {:#05}\n",
                                period,
                                period,
                                period,
                            ))
                    },
                        TimerMode::SingleShot => {
                            s1.push_str(&format!(
                                "Single: {:#018b} {:#06x} {:#05}\n",
                                period,
                                period,
                                period,
                            ))
                        }

                };

            },

        };



        let text = [TuiText::styled(s1, Style::default().fg(c!(Data)))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.render(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
