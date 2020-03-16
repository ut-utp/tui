//! A widget that does nothing but occupy space.
//!
//! Useful for testing and for blank spaces.

use super::widget_impl_support::*;
use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;
use lc3_traits::peripherals::timers::{TimerId, TimerState};
use std::sync::{mpsc, Arc, Mutex, RwLock};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timers_toggle {
    pub focusable: bool,
}

impl Default for Timers_toggle {
    fn default() -> Self {
        Self {
            focusable: false,
        }
    }
}

impl Timers_toggle {
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl TuiWidget for Timers_toggle {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Timers_toggle
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{

    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let timer_state = data.sim.get_timer_states();
        let timer_configs = data.sim.get_timer_config(); 


        let text = [
            TuiText::styled("Timer 0: \n", Style::default().fg(Color::Gray)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);
        

        let mut s0 = String::from("");

        let t0 = match timer_state[TimerId::T0] {
            TimerState::Disabled => {
                let disabled_string = "Disabled";
                s0.push_str(&format!("{}\n", disabled_string))
            }
            TimerState::Repeated => s0.push_str(&format!(
                    "Repeat:  {:#018b} {:#06x} {:#05}\n",
                    timer_configs[TimerId::T0],
                    timer_configs[TimerId::T0],
                    timer_configs[TimerId::T0]
                )),
            TimerState::SingleShot => s0.push_str(&format!(
                    "Single:  {:#018b} {:#06x} {:#05}\n",
                    timer_configs[TimerId::T0],
                    timer_configs[TimerId::T0],
                    timer_configs[TimerId::T0]
                )),
        };
        
        let text = [TuiText::styled(s0, Style::default().fg(Color::LightGreen))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.draw(area, buf);
        




        let text = [
            TuiText::styled("Timer 1: \n", Style::default().fg(Color::Gray)),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(40, Axis::X, area);
        para.draw(area, buf);

        let mut s1 = String::from("");

        let t1 = match timer_state[TimerId::T1] {
            TimerState::Disabled => {
                let disabled_string = "Disabled";
                s1.push_str(&format!("{}\n", disabled_string))
            }
            TimerState::Repeated => s1.push_str(&format!("Repeat:  {:#018b} {:#06x} {:#05}\n",
                    timer_configs[TimerId::T1],
                    timer_configs[TimerId::T1],
                    timer_configs[TimerId::T1]
                )),
            TimerState::SingleShot => s1.push_str(&format!(
                    "Single:  {:#018b} {:#06x} {:#05}\n",
                    timer_configs[TimerId::T1],
                    timer_configs[TimerId::T1],
                    timer_configs[TimerId::T1]
                )),
        };
     

        

        let text = [TuiText::styled(s1, Style::default().fg(Color::LightGreen))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        let area = increment(10, Axis::X, area);
        para.draw(area, buf);

        

    }



    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {

        match &_data.shims {

            Some(shim) => {
                match event {
                    WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
                    WidgetEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                        match c {
                            '\n' => {
                                let lock = RwLock::write(&shim.timers);

                                let mut x = format!("{}", c).replace("\n", "");
                                
                                // lock.unwrap().set_period(TimerId::T0, x);
                                
                                true
                            },

                            _ => {false}



                        }
                    },
                    _ => false,
                }

            },
           
    _ => {false}
}
    }
}
