//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Instruction, Reg, Word};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemConsole
{
    mem_addr: Addr,
    mode: u8,
    input: String,
}

impl Default for MemConsole {
    fn default() -> Self {
        Self {
            mem_addr: 0,
            mode: 0,
            input: String::from(""),
        }
    }
}

impl TuiWidget for MemConsole
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for MemConsole
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let mut t = TuiText::styled("\n", Style::default());
        if self.mode == 0 {
            t = TuiText::styled(
                "INPUT ADDRESS\n",
                Style::default().fg(Colour::Red).modifier(Modifier::BOLD),
            );
        } else {
            t = TuiText::styled(
                format!("Current Addr: {:#06x}\n", self.mem_addr),
                Style::default().fg(Colour::Gray),
            );
        }

        let text = [
            t,
            TuiText::raw(self.input.clone()),
        ];

        let mut para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

        para.draw(area, buf);

        let area = increment(25, Axis::X, area);

        if self.mode == 0 {
            let text = [
              TuiText::raw("Enter an address to get started.\n You can use default decimal format,\n or add 0x for Hex, and 0b for binary.\n e.g. 16 = 0x10 = 0b10000"),
            ];

            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            para.draw(area, buf);

        } else {
            let text = [
                TuiText::styled("Memory Manipulation Help\n", Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40))),
                TuiText::styled("b to set breakpoint, w to set watchpoint\n", Style::default().fg(Colour::LightCyan)),
                TuiText::styled("rb to remove breakpoint, rw to remove watchpoint\n", Style::default().fg(Colour::LightRed)),
                TuiText::styled("j to jump to address\n", Style::default().fg(Colour::Magenta)),
                TuiText::styled("e to enter a new address\n", Style::default().fg(Colour::LightGreen)),
            ];

            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            para.draw(area, buf);       
        }
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                let x = format!("{}", c);
                self.input.push_str(&x);
                true
            }

            Key(KeyEvent { code: KeyCode::Backspace, modifiers: EMPTY }) => {
                self.input.pop();
                true
            }

            Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                if self.mode == 0 {
                    match self.input.parse::<Addr>() {
                        Ok(a) => {
                            self.mode = 1;
                            self.mem_addr = a;
                            data.log(format!("[Addr] {}\n", a), Colour::Green);
                        }
                        Err(e) => {data.log(format!("[Addr] {}\n", e), Colour::Red)}
                    }
                    if self.input.len() > 2 {
                        let val = self.input.split_off(2);
                        if self.input == "0x" {
                            match Addr::from_str_radix(&val, 16) {
                                Ok(a) => {
                                    self.mode = 1;
                                    self.mem_addr = a;
                                }
                                Err(_e) => {}
                            }
                        } else if self.input == "0b" {
                            match Addr::from_str_radix(&val, 2) {
                                Ok(a) => {
                                    self.mode = 1;
                                    self.mem_addr = a;
                                }
                                Err(_e) => {}
                            }
                        }
                    }       
                    data.log(format!("[Addr] {}\n", self.input), Colour::Green);
                } else {
                    if self.input == "b" {
                        match data.sim.set_breakpoint(self.mem_addr) {
                            Ok(val) => {data.bp.insert(self.mem_addr, val); /*self.mode = 0;*/},
                            Err(_e) => {},
                        }
                    } else if self.input == "w" {
                        match data.sim.set_memory_watchpoint(self.mem_addr) {
                            Ok(val) => {data.wp.insert(self.mem_addr, val); /*self.mode = 0;*/},
                            Err(_e) => {},
                        }

                    } else if self.input == "rb" {
                        match data.bp.remove(&self.mem_addr) {
                            Some(val) => {data.sim.unset_breakpoint(val); /*self.mode = 0;*/},
                            None => {},
                        };
                    } else if self.input == "rw" {
                        match data.wp.remove(&self.mem_addr) {
                            Some(val) =>  {data.sim.unset_memory_watchpoint(val);/*self.mode = 0;*/},
                            None => {},
                        };
                    } else if self.input == "j" {
                        //offset = data.sim.get_pc().wrapping_sub(self.mem_addr - 2);
                        //self.mode = 0;
                    } else if self.input == "e" {
                        self.mode = 0;
                    } else {
                        match self.input.parse::<Word>() {
                            Ok(w) => {
                                data.sim.write_word(self.mem_addr, w);
                                //self.mode = 0;
                            }
                            Err(_e) => {}
                        }
                        if self.input.len() > 2 {
                            let val = self.input.split_off(2);
                            if self.input == "0x" {
                                match Word::from_str_radix(&val, 16) {
                                    Ok(w) => {
                                        data.sim.write_word(self.mem_addr, w);
                                        //self.mode = 0;
                                    }
                                    Err(_e) => {}
                                }
                            } else if self.input == "0b" {
                                match Word::from_str_radix(&val, 2) {
                                    Ok(w) => {
                                        data.sim.write_word(self.mem_addr, w);
                                        //self.mode = 0;
                                    }
                                    Err(_e) => {}
                                }
                            }
                        }
                    }
                }
                self.input = String::from("");
                true
            }
             _ => false,
        }
    }
}
