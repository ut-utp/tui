//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Reg, Word};
use MemRegMode::*;
use std::convert::TryFrom;

// Arbitrary maximum, fits all valid commands
const MAX_INPUT_LEN: u16 = 24;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemRegMode {
    InputSource,
    MemoryMod,
    RegisterMod,
    PcMod,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemRegInterface
{
    mem_addr: Addr,
    reg_num: Reg,
    mode: MemRegMode,
    input: String,
    reset_flag: u8,
}

impl Default for MemRegInterface {
    fn default() -> Self {
        Self {
            mem_addr: 0,
            reg_num: Reg::R0,
            mode: InputSource,
            input: String::from(""),
            reset_flag: 0,
        }
    }
}

impl<Wt: WidgetTypes> Widget<Wt> for MemRegInterface {
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        if self.reset_flag != data.reset_flag {
            self.mode = InputSource;
            self.reset_flag = data.reset_flag;
        }

        let prompt = match self.mode {
            InputSource => {
                TuiText::styled(
                    "INPUT ADDRESS\n",
                    Style::default().fg(c!(ConsoleRequest)).modifier(Modifier::BOLD),
                )
            },
            MemoryMod => {
                TuiText::styled(
                    format!("Current Address: {:#06x}\n", self.mem_addr),
                    Style::default().fg(c!(Name)),
                )
            },
            RegisterMod => {
                TuiText::styled(
                    format!("Current Register: {}\n", self.reg_num),
                    Style::default().fg(c!(Name)),
                )
            },
            PcMod => {
                TuiText::styled(
                    format!("Current Register: PC\n"),
                    Style::default().fg(c!(Pc)),
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

        para.render(area, buf);

        let area = increment(MAX_INPUT_LEN+1, Axis::X, area);

        let instructions = match self.mode {
            InputSource => {
                [TuiText::styled("Enter an address or register to get started.\n You can use default decimal format,\n or add 0x for hexadecimal, and 0b for binary.\n e.g. 16 = 0x10 = 0b10000\n For registers, enter R0 to R7 or PC", Style::default().fg(c!(ConsoleHelp))), ]
            },
            MemoryMod => {
                [TuiText::styled("Memory Manipulation Help\nb to toggle breakpoint\nw to toggle watchpoint\nj to jump to address\ne to enter a new address\nType a value to change data at the address\n", Style::default().fg(c!(Title))), ]
            },
            RegisterMod | PcMod => {
                [TuiText::styled("Register Manipulation Help\nb to toggle breakpoint at reg address\nw to toggle watchpoint\nj to jump to reg address\ne to enter a new address\nType a value to change data in the register\n", Style::default().fg(c!(Title))), ]
            },
        };

        para = Paragraph::new(instructions.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        fn set_bp<Wt: WidgetTypes>(cur_addr: u16, data: &mut Data<Wt>) {
            match data.bp.remove(&cur_addr) {
                Some(val) => {data.sim.unset_breakpoint(val as u8);},
                None => {match data.sim.set_breakpoint(cur_addr) {
                    Ok(val) => {data.bp.insert(cur_addr, val as usize);},
                    Err(_e) => {},
                }},
            };
        }

        fn set_wp<Wt: WidgetTypes>(cur_addr: u16, data: &mut Data<Wt>) {
            match data.wp.remove(&cur_addr) {
                Some(val) => {data.sim.unset_memory_watchpoint(val as u8);},
                None => {match data.sim.set_memory_watchpoint(cur_addr) {
                    Ok(val) => {data.wp.insert(cur_addr, val as usize);},
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
                            data.log(format!("[Addr] Invalid hex value: {}\n", self.input), c!(InvalidInput));
                        }
                    }
                } else if self.input.starts_with("x") {
                    match Addr::from_str_radix(&self.input[1..], 16) {
                        Ok(word) => {
                            $value = word;
                            $on_success;
                        }
                        Err(_e) => {
                            data.log(format!("[Addr] Invalid hex value: {}\n", self.input), c!(InvalidInput));
                        }
                    }
                } else if self.input.starts_with("0b") {
                    match Addr::from_str_radix(&self.input[2..], 2) {
                        Ok(word) => {
                            $value = word;
                            $on_success;
                        }
                        Err(_e) => {
                            data.log(format!("[Addr] Invalid binary value: {}\n", self.input), c!(InvalidInput));
                        }
                    }
                } else {
                    match self.input.parse::<Addr>() {
                        Ok(word) => {
                            $value = word;
                            $on_success;
                        }
                        Err(e) => {data.log(format!("[Addr] Invalid value: {}\n", self.input), c!(InvalidInput))}
                    }
                }
            }
        }

        macro_rules! modify_addr {
            ($addr:ident, $word:ident, $on_write:block) => {
                if self.input == String::from("b") {
                    set_bp::<Wt>($addr, data);
                    self.mode = InputSource;
                } else if self.input == String::from("w") {
                    set_wp::<Wt>($addr, data);
                    self.mode = InputSource;
                } else if self.input == String::from("j") {
                    data.jump = (data.jump.0+1,$addr);
                    self.mode = InputSource;
                } else if self.input == String::from("e") {
                    self.mode = InputSource;
                    data.mem_reg_inter = (0,0);
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

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT }) |
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
                    // Else, change to InputSource mode
                    self.mode = InputSource;
                }
                true
            }

            Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                self.input = self.input.trim().to_lowercase();
                match self.mode {
                    InputSource => {
                        if self.input.starts_with("r") {
                            match self.input[1..].parse::<u8>() {
                                Ok(value) => {
                                    match Reg::try_from(value) {
                                        Ok(reg) => {
                                            self.mode = RegisterMod;
                                            self.reg_num = reg;
                                            data.mem_reg_inter = (2, value.into());
                                        },
                                        Err(e) => {
                                            data.log(format!("[Reg] Invalid register: {}\n", self.input), c!(InvalidInput));
                                        }
                                    };
                                },
                                Err(e) => {
                                    data.log(format!("[Reg] Invalid register: {}\n", self.input), c!(InvalidInput));
                                }
                            }
                        } else if self.input == String::from("pc") {
                            self.mode = PcMod;
                            data.mem_reg_inter = (2, 10);
                        } else {
                            let mut addr: Word;
                            parse_addr!(
                                {
                                    self.mode = MemoryMod;
                                    self.mem_addr = addr;
                                    data.mem_reg_inter = (1, addr);
                                },
                                addr
                            )
                        }
                    },
                    MemoryMod => {
                        let addr = self.mem_addr;
                        let mut word: Word;
                        modify_addr!(
                            addr,
                            word,
                            {data.sim.write_word(addr, word);}
                        );
                    },
                    RegisterMod => {
                        let addr_from_reg = data.sim.get_register(self.reg_num);
                        let mut word: Word;
                        modify_addr!(
                            addr_from_reg,
                            word,
                            {data.sim.set_register(self.reg_num, word);}
                        );
                    },
                    PcMod => {
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
