//! TODO!

use super::widget_impl_support::*;

use lc3_traits::peripherals::gpio::{GpioPin, GpioState};
use lc3_traits::peripherals::adc::{AdcPin, AdcState};
use lc3_traits::peripherals::pwm::{PwmPin, PwmState};
use lc3_traits::peripherals::timers::{TimerId, TimerState};

use std::sync::RwLock;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsolePeripherals
{
    history: String,
    input: String,
}

impl Default for ConsolePeripherals {
    fn default() -> Self {
        Self {
            history: String::from(""),
            input: String::from(""),
        }
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for ConsolePeripherals
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let peripheral_help = format!("Hello! To write to ADC or GPIO from the peripheral console,\nyou must write a line below separated by colons (:) then press ENTER to submit!\nex. gpio:1:1 will set GPIO pin 1 to True");
        let text_help = [TuiText::styled(peripheral_help, Style::default().fg(c!(ConsoleHelp)))];

        let para_help = Paragraph::new(text_help.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let text = [TuiText::styled(">", Style::default().fg(c!(Title)))];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);


        if area.height <= 1 {
            para.render(area, buf);
        } else if area.height <= 4 {
            let area = Rect::new(area.x, area.y+area.height/2, area.width, 3);
            para.render(area, buf);

            let text = [TuiText::styled(self.input.clone(), Style::default().fg(c!(ConsoleIn)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(1, Axis::Y, area);
            if area.height < 2 {
                return;
            }
            para.render(area,buf);
        } else {

            para_help.render(area, buf);

            let area = Rect::new(area.x, area.y+area.height-3, area.width, 3);
            para.render(area, buf);

            let text = [TuiText::styled(self.input.clone(), Style::default().fg(c!(ConsoleIn)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(1, Axis::Y, area);
            if area.height < 2 {
                return;
            }
            para.render(area,buf);
        }

    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();
        match &_data.shims{
            Some(shim) => {
                match event {
                    Focus(FocusEvent::GotFocus) => true,
                    Focus(FocusEvent::LostFocus) => true,
                    Mouse(MouseEvent::Up(_, _, _, _)) => true,
                    Mouse(MouseEvent::Down(_, _, _, _)) => true,


                    Key(KeyEvent { code: KeyCode::Backspace, modifiers: EMPTY }) => {
                        if self.input.len() > 0 {
                            self.input.remove(self.input.len()-1);
                        }

                        true
                    }


                    Key(KeyEvent { code: KeyCode::Char(c), modifiers: _ }) => {
                        let x = format!("{}", c);
                        self.input.push_str(&x);
                        true
                    }

                    Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                        let x = self.input.clone();
                        let vec: Vec<&str> = x.split(":").collect();

                        self.input = String::from("");
                        let adc_pins = [AdcPin::A0, AdcPin::A1, AdcPin::A2, AdcPin::A3];
                        let gpio_pins = [GpioPin::G0, GpioPin::G1, GpioPin::G2, GpioPin::G3, GpioPin::G4, GpioPin::G5, GpioPin::G6, GpioPin::G7];
                        if vec.len() > 2 {
                            match vec[0] {
                                "adc" => {
                                    let adc_states = _data.sim.get_adc_states();
                                    let lock = RwLock::write(&shim.adc);
                                    let adc_pin = adc_pins[vec[1].parse::<u8>().unwrap() as usize];
                                         match adc_states[adc_pin] {
                                                AdcState::Enabled => {
                                                    let value = vec[2].parse::<u8>().unwrap();
                                                    lock.unwrap().set_value(adc_pin, value);
                                                },
                                                AdcState::Disabled => {

                                                }

                                            }
                                    },

                                "gpio" => {
                                    let gpio_states = _data.sim.get_gpio_states();
                                    let lock = RwLock::write(&shim.gpio);

                                    let gpio_pin = gpio_pins[vec[1].parse::<u8>().unwrap() as usize];
                                    match gpio_states[gpio_pin] {

                                                GpioState::Input | GpioState::Interrupt => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(gpio_pin, false);
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(gpio_pin, true);
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }

                                     },
                                        _ => {}

                                }


                            }

                        true
                    }
                     _ => false,
                }

            }

            _ => false,

        }

    }
}
