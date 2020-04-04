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
        
                    Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {

                        let mut x = format!("{}", c);
                        


                        self.input.push_str(&x);
        
                        let mut vec: Vec<&str> = x.split(":").collect();
                        
                        
                        if vec.len() > 2 {
                        match vec[0] {

                            // "pwm" => {
                            //     let lock = RwLock::write(&shim.pwm);

                            //     match vec[1] {
                            //         "0" => {
                            //             let duty_cycle = vec[2].parse::<NonZeroU8>().unwrap();
                            //              lock.unwrap().set_duty_cycle_helper(PwmPin::P0, duty_cycle);




                            //         },
                            //         "1" => {
                            //             let duty_cycle = vec[2].parse::<NonZeroU8>().unwrap();
                            //             lock.unwrap().set_duty_cycle_helper(PwmPin::P1, duty_cycle);
                            //         },

                            //         _ => {}


                            //     }


                            // },
                            "adc" => {
                                let lock = RwLock::write(&shim.adc);

                                match vec[1] {
                                    "0" => {
                                        let value = vec[2].parse::<u8>().unwrap();
                                         lock.unwrap().set_value(AdcPin::A0, value);
                                    },
                                    "1" => {
                                        let value = vec[2].parse::<u8>().unwrap();
                                        lock.unwrap().set_value(AdcPin::A1, value);
                                    },
                                    "2" => {
                                        let value = vec[2].parse::<u8>().unwrap();
                                        lock.unwrap().set_value(AdcPin::A2, value);
                                    },
                                    "3" => {
                                        let value = vec[2].parse::<u8>().unwrap();
                                        lock.unwrap().set_value(AdcPin::A3, value);
                                    },

                                    _ => {}
                                }
                            },
                            "gpio" => {
                                let lock = RwLock::write(&shim.gpio);
                                
                                match vec[1] {
                                    "0" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G0, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G0, true); 
                                            },
                                            _ => {}
                                        }
                                        
                                    },
                                    "1" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G1, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G1, true); 
                                            },
                                            _ => {}
                                        }
                                        
                                    },
                                    "2" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G2, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G2, true); 
                                            },
                                            _ => {}
                                        }
                                        
                                    },
                                    "3" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G3, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G3, true); 
                                            },
                                            _ => {}
                                        }
                                        
                                    },
                                    "4" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G4, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G4, true); 
                                            },
                                            _ => {}
                                        }
                                        
                                    },
                                    "5" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G5, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G5, true); 
                                            },
                                            _ => {}
                                        }
                                         
                                    },
                                    "6" => {
                                        match vec[3] {
                                            "0" => {
                                                lock.unwrap().set_pin(GpioPin::G6, false); 
                                            },
                                            "1" => {
                                                lock.unwrap().set_pin(GpioPin::G6, true); 
                                            },
                                            _ => {}
                                        }
                                        
                                    },
                                    "7" => {
                                        match vec[3] {
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
                            // "timers" => {
                            //     let lock = RwLock::write(&shim.timers);
                                
                            //     match vec[1] {
                            //         "0" => {
                            //             let milliseconds = vec[2].parse::<Word>().unwrap();
                            //             //  lock.unwrap().set_period(TimerId::T0, milliseconds);
                            //         },
                            //         "1" => {
                            //             let milliseconds = vec[2].parse::<Word>().unwrap();
                            //             //  lock.unwrap().set_period(TimerId::T0, milliseconds);

                            //         },

                            //         _ => {}


                            //     }


                            // },
                            _ => {}


                        }
                    }
        
        
                        true
                    }
        
                    Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                        self.input = String::from("");
                        true
                    }
                     _ => false,
                }

            }

            _ => false,

        }
        
    }
}