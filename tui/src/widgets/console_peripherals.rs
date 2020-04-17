//! TODO!

use super::widget_impl_support::*;

use core::num::NonZeroU8;
use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;
use lc3_traits::peripherals::gpio::{GpioPin, GpioState};
use lc3_traits::peripherals::adc::{AdcPin, AdcState};
use lc3_traits::peripherals::pwm::{PwmPin, PwmState};
use lc3_traits::peripherals::timers::{TimerId, TimerState};
use std::sync::{mpsc, Arc, Mutex, RwLock};

use lc3_isa::{Addr, Instruction, Reg, Word};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Console_peripherals
{
    history: String,
    input: String,
}

impl Default for Console_peripherals {
    fn default() -> Self {
        Self {
            history: String::from(""),
            input: String::from(""),
        }
    }
}

impl TuiWidget for Console_peripherals
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Console_peripherals
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let Console_psr_pc = data.sim.get_registers_psr_and_pc();
        let (Console, psr, pc) = Console_psr_pc;

        let peripheral_help = format!("Hello! To write to ADC or GPIO from the peripheral console,\nyou must write a line below separated by colons (:) then press ENTER to submit!\nex. gpio:1:1 will set GPIO pin 1 to True");
        let text_help = [TuiText::styled(peripheral_help, Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];
        
        let mut para_help = Paragraph::new(text_help.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let text = [TuiText::styled(">", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);


        if area.height <= 1 {
            para.draw(area, buf);
        } else if area.height <= 4 {
            let area = Rect::new(area.x, area.y+area.height/2, area.width, 3);
            para.draw(area, buf);

            let text = [TuiText::styled(self.input.clone(), Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(1, Axis::Y, area);
            if area.height < 2 {
                return;
            }
            para.draw(area,buf);
        } else {

            para_help.draw(area, buf);

            let area = Rect::new(area.x, area.y+area.height-3, area.width, 3);
            para.draw(area, buf);

            let text = [TuiText::styled(self.input.clone(), Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(1, Axis::Y, area);
            if area.height < 2 {
                return;
            }
            para.draw(area,buf);
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
                        self.input.remove(self.input.len()-1);

                        true

                    }

                    
                    Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {

                        let mut x = format!("{}", c);
                        self.input.push_str(&x);
                        true
                        
                    }
        
                    Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                        let x = self.input.clone();
                        let vec: Vec<&str> = x.split(":").collect();

                        self.input = String::from("");
                        
                        if vec.len() > 2 {
                            match vec[0] {
                                "adc" => {
                                    let lock = RwLock::write(&shim.adc);
                                    let adc_states = _data.sim.get_adc_states();
                                    match vec[1] {
                                        "0" => {
                                            match adc_states[AdcPin::A0] {
                                                AdcState::Enabled => {
                                                    let value = vec[2].parse::<u8>().unwrap();
                                                    lock.unwrap().set_value(AdcPin::A0, value);
                                                },
                                                AdcState::Disabled => {
                                                }

                                            }
                                            
                                        },
                                        "1" => {
                                            match adc_states[AdcPin::A1] {
                                                AdcState::Enabled => {
                                                    let value = vec[2].parse::<u8>().unwrap();
                                                    lock.unwrap().set_value(AdcPin::A1, value);
                                                },
                                                AdcState::Disabled => {
                                                    
                                                }

                                            }
                                        },
                                        "2" => {
                                            match adc_states[AdcPin::A2] {
                                                AdcState::Enabled => {
                                                    let value = vec[2].parse::<u8>().unwrap();
                                                    lock.unwrap().set_value(AdcPin::A2, value);
                                                },
                                                AdcState::Disabled => {
                                                    
                                                }

                                            }
                                        },
                                        "3" => {
                                            match adc_states[AdcPin::A3] {
                                                AdcState::Enabled => {
                                                    let value = vec[2].parse::<u8>().unwrap();
                                                    lock.unwrap().set_value(AdcPin::A3, value);
                                                },
                                                AdcState::Disabled => {
                                                    
                                                }

                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                "gpio" => {
                                    let lock = RwLock::write(&shim.gpio);
                                    let gpio_states = _data.sim.get_gpio_states();
                                    match vec[1] { // gpio:pin:{true(1)/false(0)}
                                        "0" => {
                                            match gpio_states[GpioPin::G0] {
                                                
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G0, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G0, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                            
                                            
                                        },
                                        "1" => {
                                            match gpio_states[GpioPin::G1] {
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G1, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G1, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                            
                                        },
                                        "2" => {
                                            match gpio_states[GpioPin::G2] {
                                                
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G2, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G2, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                            
                                        },
                                        "3" => {
                                            match gpio_states[GpioPin::G3] {
                                                
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G3, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G3, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                            
                                        },
                                        "4" => {
                                            match gpio_states[GpioPin::G4] {
                                                
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G4, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G4, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        "5" => {
                                            match gpio_states[GpioPin::G5] {
                                                
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G5, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G5, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                            
                                        },
                                        "6" => {
                                            match gpio_states[GpioPin::G6] {
                                                
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G6, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G6, true); 
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                            
                                        },
                                        "7" => {
                                            match gpio_states[GpioPin::G7] {
                                                GpioState::Input => {
                                                    match vec[2] {
                                                        "0" => {
                                                            lock.unwrap().set_pin(GpioPin::G7, false); 
                                                        },
                                                        "1" => {
                                                            lock.unwrap().set_pin(GpioPin::G7, true); 
                                                        },
                                                        _ => {}
                                                    }
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
