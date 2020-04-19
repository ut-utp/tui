//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Instruction, Reg, Word};
use MemRegMode::*;
use std::convert::TryFrom;

// Arbitrary maximum, fits all valid commands
const MAX_INPUT_LEN: u16 = 24;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemRegMode {
    INPUT_SOURCE,
    MEMORY_MOD,
    REGISTER_MOD,
    PC_MOD,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemRegInterface
{
    mem_addr: Addr,
    reg_num: Reg,
    mode: MemRegMode,
    input: String,
}

impl Default for MemRegInterface {
    fn default() -> Self {
        Self {
            mem_addr: 0,
            reg_num: Reg::R0,
            mode: INPUT_SOURCE,
            input: String::from(""),
        }
    }
}

impl TuiWidget for MemRegInterface
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for MemRegInterface
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let prompt = match self.mode {
            INPUT_SOURCE => {
                TuiText::styled(
                    "INPUT ADDRESS\n",
                    Style::default().fg(Colour::Red).modifier(Modifier::BOLD),
                )
            },
            MEMORY_MOD => {
                TuiText::styled(
                    format!("Current Address: {:#06x}\n", self.mem_addr),
                    Style::default().fg(Colour::Gray),
                )
            },
            REGISTER_MOD => {
                TuiText::styled(
                    format!("Current Register: {}\n", self.reg_num),
                    Style::default().fg(Colour::Gray),
                )
            },
            PC_MOD => {
                TuiText::styled(
                    format!("Current Register: PC\n"),
                    Style::default().fg(Colour::Gray),
                )
            },
        };

        let prompt_input_text = [
            prompt,
            TuiText::raw(self.input.clone()),
        ];

        let mut para = Paragraph::new(prompt_input_text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

        para.draw(area, buf);

        let area = increment(MAX_INPUT_LEN+1, Axis::X, area);

        let instructions = match self.mode {
            INPUT_SOURCE => {
                [TuiText::raw("Enter an address or register to get started.\n You can use default decimal format,\n or add 0x for hexadecimal, and 0b for binary.\n e.g. 16 = 0x10 = 0b10000\n For registers, enter R0 to R7 or PC"), ]
            },
            MEMORY_MOD => {
                [TuiText::styled("Memory Manipulation Help\nb to toggle breakpoint\nw to toggle watchpoint\nj to jump to address\ne to enter a new address\nType a value to change data at the address\n", Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40))), ]
            },
            REGISTER_MOD | PC_MOD => {
                [TuiText::styled("Register Manipulation Help\nb to toggle breakpoint at reg address\nw to toggle watchpoint\nj to jump to reg address\ne to enter a new address\nType a value to change data in the register\n", Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40))), ]
            },
        };

        para = Paragraph::new(instructions.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        fn set_bp<'a, 'int, C, I, O>(cur_addr: u16, data: &mut TuiData<'a, 'int, C, I, O>)
        where
            C: Control + ?Sized + 'a,
            I: InputSink + ?Sized + 'a,
            O: OutputSource + ?Sized + 'a,
        {
            match data.bp.remove(&cur_addr) {
                Some(val) => {data.sim.unset_breakpoint(val);},
                None => {match data.sim.set_breakpoint(cur_addr) {
                    Ok(val) => {data.bp.insert(cur_addr, val);},
                    Err(_e) => {},
                }},
            };
        }

        fn set_wp<'a, 'int, C, I, O>(cur_addr: u16, data: &mut TuiData<'a, 'int, C, I, O>)
        where
            C: Control + ?Sized + 'a,
            I: InputSink + ?Sized + 'a,
            O: OutputSource + ?Sized + 'a,
        {
            match data.wp.remove(&cur_addr) {
                Some(val) => {data.sim.unset_memory_watchpoint(val);},
                None => {match data.sim.set_memory_watchpoint(cur_addr) {
                    Ok(val) => {data.wp.insert(cur_addr, val);},
                    Err(_e) => {},
                }},
            };
        }

        macro_rules! parse_addr {
            ($on_success:block, $value:ident) => {
                if self.input.starts_with("0x") {
                    match Addr::from_str_radix(&self.input[2..], 16) {
                        Ok(word) => {
                            $value = word;
                            $on_success;
                        }
                        Err(_e) => {
                            data.log(format!("[Addr] Invalid hex value: {}\n", self.input), Colour::Red);
                        }
                    }
                } else if self.input.starts_with("0b") {
                    match Addr::from_str_radix(&self.input[2..], 2) {
                        Ok(word) => {
                            $value = word;
                            $on_success;
                            self.mode = INPUT_SOURCE;
                        }
                        Err(_e) => {
                            data.log(format!("[Addr] Invalid binary value: {}\n", self.input), Colour::Red);
                        }
                    }
                } else {
                    match self.input.parse::<Addr>() {
                        Ok(word) => {
                            $value = word;
                            $on_success;
                        }
                        Err(e) => {data.log(format!("[Addr] Invalid value: {}\n", self.input), Colour::Red)}
                    }
                }
            }
        }

        macro_rules! modify_addr {
            ($addr:ident, $word:ident, $on_write:block) => {
                if self.input == String::from("b") {
                    set_bp($addr, data);
                    self.mode = INPUT_SOURCE;
                } else if self.input == String::from("w") {
                    set_wp($addr, data);
                    self.mode = INPUT_SOURCE;
                } else if self.input == String::from("j") {
                    //offset = data.sim.get_pc().wrapping_sub(self.mem_addr - 2);
                } else if self.input == String::from("e") {
                    self.mode = INPUT_SOURCE;
                } else {
                    parse_addr!(
                        $on_write,
                        $word
                    );
                }
            }
        }

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                let x = format!("{}", c);
                if self.input.len() < MAX_INPUT_LEN as usize {
                    self.input.push_str(&x);
                }
                true
            }

            Key(KeyEvent { code: KeyCode::Backspace, modifiers: EMPTY }) => {
                self.input.pop();
                true
            }

            Key(KeyEvent {code: KeyCode::Esc, modifiers: EMPTY}) => {
                // If input is non-empty, clear input
                if self.input.len() > 0 {
                    self.input = String::from("");
                } else {
                    // Else, change to INPUT_SOURCE mode
                    self.mode = INPUT_SOURCE;
                }
                true
            }

            Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                self.input = self.input.trim().to_lowercase();
                match self.mode {
                    INPUT_SOURCE => {
                        if self.input.starts_with("r") {
                            match self.input[1..].parse::<u8>() {
                                Ok(value) => {
                                    match Reg::try_from(value) {
                                        Ok(reg) => {
                                            self.mode = REGISTER_MOD;
                                            self.reg_num = reg;
                                        },
                                        Err(e) => {
                                            data.log(format!("[Reg] Invalid register: {}\n", self.input), Colour::Red);
                                        }
                                    };
                                },
                                Err(e) => {
                                    data.log(format!("[Reg] Invalid register: {}\n", self.input), Colour::Red);
                                }
                            }
                        } else if self.input == String::from("pc") {
                            self.mode = PC_MOD;
                        } else {
                            let mut addr: Word;
                            parse_addr!(
                                {
                                    self.mode = MEMORY_MOD;
                                    self.mem_addr = addr;
                                },
                                addr
                            )
                        }
                    },
                    MEMORY_MOD => {
                        let addr = self.mem_addr;
                        let mut word: Word;
                        modify_addr!(
                            addr,
                            word,
                            {data.sim.write_word(addr, word);}
                        );
                    },
                    REGISTER_MOD => {
                        let addr_from_reg = data.sim.get_register(self.reg_num);
                        let mut word: Word;
                        modify_addr!(
                            addr_from_reg,
                            word,
                            {data.sim.set_register(self.reg_num, word);}
                        );
                    },
                    PC_MOD => {
                        let addr_from_pc = data.sim.get_pc();
                        let mut word: Word;
                        modify_addr!(
                            addr_from_pc,
                            word,
                            {data.sim.set_pc(word);}
                        );
                    },
                };
                self.input = String::from("");
                true
            }
            _ => false,
        }
    }
}