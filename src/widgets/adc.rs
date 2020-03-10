//! TODO!

use super::widget_impl_support::*;
use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;
use lc3_traits::peripherals::adc::{AdcPin, AdcState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Adc {
    pub focusable: bool,
}

impl Default for Adc {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Adc {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl TuiWidget for Adc {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Adc
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let adc_states = data.sim.get_adc_states();
        let adcin = data.sim.get_adc_readings();


        let text = [
            TuiText::styled("ADC 0: \nADC 1: \nADC 2: \n", Style::default().fg(Color::Gray)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);


        let mut s1 = String::from("");

        let adc_pins_1 = [AdcPin::A0, AdcPin::A1, AdcPin::A2];
        let adc_pins_2 = [AdcPin::A3, AdcPin::A4, AdcPin::A5];
        for i in 0..3 {
            match adc_states[adc_pins_1[i]]{
                AdcState::Disabled => {
                    let disabled_string = "Disabled";
                    s1.push_str(&format!("{}\n",
                    disabled_string, ));
                }
                _ => {
                    match adcin[adc_pins_1[i]] {
                        Ok(val) => {
                                s1.push_str(&format!(
                                "{}\n",
                                val,

                                ));
                            }
                        _ => {
                            let err_string = "-";
                            s1.push_str(&format!(
                                "{}\n",
                                err_string,

                                ));
                        }
                        }
                }
            }
        }

        let text = [TuiText::styled(s1, Style::default().fg(Color::LightGreen))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.draw(area, buf);



        let text = [
            TuiText::styled("ADC 3: \nADC 4: \nADC 5: \n", Style::default().fg(Color::Gray)),
            ];

        let mut para = Paragraph::new(text.iter())
        .style(Style::default().fg(Color::White).bg(Color::Reset))
        .alignment(Alignment::Left)
        .wrap(true);
        let area = increment(40, Axis::X, area);
        para.draw(area, buf);

        let mut s2 = String::from("");



            for i in 0..3 {
                match adc_states[adc_pins_2[i]]{
                    AdcState::Disabled => {
                        let disabled_string = "Disabled";
                        s2.push_str(&format!("{}\n",
                        disabled_string, ));
                    }
                    _ => {
                        match adcin[adc_pins_2[i]] {
                            Ok(val) => {
                                    s2.push_str(&format!(
                                    "{}\n",
                                    val,

                                    ));
                                }
                            _ => {
                                let err_string = "-";
                                s2.push_str(&format!(
                                    "{}\n",
                                    err_string,

                                    ));
                            }
                            }
                    }
                }
            }

            let text = [TuiText::styled(s2, Style::default().fg(Color::LightGreen))];
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
