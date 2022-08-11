//! TODO!

use super::widget_impl_support::*;

use lc3_traits::peripherals::gpio::{GpioPin, GpioState};

use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Gpio {
    pub focusable: bool,
}

impl Default for Gpio {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Gpio {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Gpio
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let gpio_states = data.sim.get_gpio_states();
        let gpioin = data.sim.get_gpio_readings();


        let text = [
            TuiText::styled("GPIO 0: \nGPIO 1: \nGPIO 2: \nGPIO 3: \n", Style::default().fg(c!(Name))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf);


        let mut s1 = String::from("");

        let gpio_pins_1 = [GpioPin::G0, GpioPin::G1, GpioPin::G2, GpioPin::G3];
        let gpio_pins_2 = [GpioPin::G4, GpioPin::G5, GpioPin::G6, GpioPin::G7];
        //let gpio_pins_2 = [GpioPin::G0, GpioPin::G1, GpioPin::G2, GpioPin::G3, GpioPin::G4, GpioPin::G5, GpioPin::G6, GpioPin::G7];
        for i in 0..4 {
            match gpio_states[gpio_pins_1[i]]{
                GpioState::Disabled => {
                    let disabled_string = "Disabled";
                    s1.push_str(&format!("{}\n",
                    disabled_string, ));
                }
                GpioState::Output => {
                    let mut output_string = String::from("Output");

                    if let Some(shims) = &data.shims {
                        match RwLock::read(&shims.gpio).unwrap().get_pin(gpio_pins_1[i]).unwrap() {
                            true => output_string.push_str(": 1"),
                            false => output_string.push_str(": 0"),
                        }
                    }

                    s1.push_str(&format!("{}\n",
                    output_string, ));
                },


                GpioState::Input => {
                    match gpioin[gpio_pins_1[i]] {
                        Ok(val) => {
                                s1.push_str(&format!(
                                "Input: {}\n",
                                val,

                                ));
                            }
                        _ => {
                            let err_string = "-";
                            s1.push_str(&format!(
                                "Input: {}\n",
                                err_string,

                                ));
                        }
                        }
                },

                GpioState::Interrupt => {
                    match gpioin[gpio_pins_1[i]] {
                        Ok(val) => {
                                s1.push_str(&format!(
                                "Interrupt: {}\n",
                                val,

                                ));
                            }
                        _ => {
                            let err_string = "-";
                            s1.push_str(&format!(
                                "Interrupt: {}\n",
                                err_string,

                                ));
                        }
                        }
                }
            }
        }

        let text = [TuiText::styled(s1, Style::default().fg(c!(Data)))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.render(area, buf);



        let text = [
            TuiText::styled("GPIO 4: \nGPIO 5: \nGPIO 6: \nGPIO 7: \n", Style::default().fg(c!(Name))),
            ];

        let mut para = Paragraph::new(text.iter())
        .style(Style::default().fg(Colour::White).bg(Colour::Reset))
        .alignment(Alignment::Left)
        .wrap(true);
        let area = increment(40, Axis::X, area);
        para.render(area, buf);

        let mut s2 = String::from("");



            for i in 0..4 {
                match gpio_states[gpio_pins_2[i]]{
                    GpioState::Disabled => {
                        let disabled_string = "Disabled";
                        s2.push_str(&format!("{}\n",
                        disabled_string, ));
                    },
                    GpioState::Output => {
                        let mut output_string = String::from("Output");

                        if let Some(shims) = &data.shims {
                            match RwLock::read(&shims.gpio).unwrap().get_pin(gpio_pins_2[i]).unwrap() {
                                true => output_string.push_str(": 1"),
                                false => output_string.push_str(": 0"),
                            }
                        }

                        s2.push_str(&format!("{}\n",
                        output_string, ));
                    },
                    _ => {
                        match gpioin[gpio_pins_2[i]] {
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

            let text = [TuiText::styled(s2, Style::default().fg(c!(Data)))];
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
