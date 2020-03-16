//! TODO!

use super::widget_impl_support::*;
use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;
use lc3_traits::peripherals::gpio::{GpioPin, GpioState};
use std::sync::{mpsc, Arc, Mutex, RwLock};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Gpio_toggle {
    pub focusable: bool,
}

impl Default for Gpio_toggle {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Gpio_toggle {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl TuiWidget for Gpio_toggle {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Gpio_toggle
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
            TuiText::styled("GPIO 0: \nGPIO 1: \nGPIO 2: \nGPIO 3: \n", Style::default().fg(Color::Gray)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);


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
                _ => {
                    match gpioin[gpio_pins_1[i]] {
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
            TuiText::styled("GPIO 4: \nGPIO 5: \nGPIO 6: \nGPIO 7: \n", Style::default().fg(Color::Gray)),
            ];

        let mut para = Paragraph::new(text.iter())
        .style(Style::default().fg(Color::White).bg(Color::Reset))
        .alignment(Alignment::Left)
        .wrap(true);
        let area = increment(40, Axis::X, area);
        para.draw(area, buf);

        let mut s2 = String::from("");



            for i in 0..4 {
                match gpio_states[gpio_pins_2[i]]{
                    GpioState::Disabled => {
                        let disabled_string = "Disabled";
                        s2.push_str(&format!("{}\n",
                        disabled_string, ));
                    }
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

            let text = [TuiText::styled(s2, Style::default().fg(Color::LightGreen))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(10, Axis::X, area);
            para.draw(area, buf);


    }




    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        let gpio_states = _data.sim.get_gpio_states();


        match &_data.shims{
            

            Some(shim) => { 


             match event {
                 WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
                 WidgetEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                    match(c) {
                        '0' => {
                            let lock = RwLock::write(&shim.gpio);
                             lock.unwrap().set_pin(GpioPin::G0, true); 
                        },
                        '1' => {
                            let lock = RwLock::write(&shim.gpio);
                            lock.unwrap().set_pin(GpioPin::G1, true); 
                        },
                        '2' => {
                            let lock = RwLock::write(&shim.gpio);
                            lock.unwrap().set_pin(GpioPin::G2, true); 
                          
                            
                       
                        },
                        '3' => {
                            let lock = RwLock::write(&shim.gpio);

                            lock.unwrap().set_pin(GpioPin::G3, true); 
                           
                            
                       
                        },
                        '4' => {
                            let lock = RwLock::write(&shim.gpio);
                            lock.unwrap().set_pin(GpioPin::G4, true); 
                          
                            
                       
                        },
                        '5' => {
                            let lock = RwLock::write(&shim.gpio);

                            lock.unwrap().set_pin(GpioPin::G5, true); 
                            
                            
                       
                        },
                        '6' => {
                            let lock = RwLock::write(&shim.gpio);
                            lock.unwrap().set_pin(GpioPin::G6, true); 
                          
                            
                       
                        },
                        '7' => {
                            let lock = RwLock::write(&shim.gpio);
                            lock.unwrap().set_pin(GpioPin::G7, true); 
                            
                        },
                        _ => {
                      
                        }


                    }
                
                    true
                },
            _ => false,
            }
         
        }
        _ => { false}
        
    }
}
}
