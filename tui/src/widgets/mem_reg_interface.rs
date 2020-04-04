//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Instruction, Reg, Word};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemRegInterface
{
    mem_addr: Addr,
    reg_num: Reg,
    mode: u8,
    input: String,
}

impl Default for MemRegInterface {
    fn default() -> Self {
        Self {
            mem_addr: 0,
            reg_num: Reg::R0,
            mode: 0,
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
        let mut t = TuiText::styled("\n", Style::default());
        if self.mode == 0 {
            t = TuiText::styled(
                "INPUT ADDRESS\n",
                Style::default().fg(Colour::Red).modifier(Modifier::BOLD),
            );
        } else if self.mode == 1 {
            t = TuiText::styled(
                format!("Current Addr: {:#06x}\n", self.mem_addr),
                Style::default().fg(Colour::Gray),
            );
        } else if self.mode == 2 {
            t = TuiText::styled(
                format!("Current Reg: {}\n", self.reg_num),
                Style::default().fg(Colour::Gray),
            );
        } else if self.mode == 3 {
            t = TuiText::styled(
                format!("Current Reg: PC\n"),
                Style::default().fg(Colour::Gray),
            );
        } else if self.mode == 4 {
            t = TuiText::styled(
                format!("Current Reg: PSR\n Currently Defunct, may remove\n"),
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
              TuiText::raw("Enter an address or register to get started.\n You can use default decimal format,\n or add 0x for Hex, and 0b for binary.\n e.g. 16 = 0x10 = 0b10000\n For regs, can do R0 to R7 or PC"),
            ];

            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            para.draw(area, buf);

        } else if self.mode == 1 {
            let text = [
                TuiText::styled("Memory Manipulation Help\n", Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40))),
                TuiText::styled("b to set breakpoint, w to set watchpoint\n", Style::default().fg(Colour::LightCyan)),
                TuiText::styled("rb to remove breakpoint, rw to remove watchpoint\n", Style::default().fg(Colour::LightRed)),
                TuiText::styled("j to jump to address\n", Style::default().fg(Colour::Magenta)),
                TuiText::styled("e to enter a new adress, or type a register to change directly\n", Style::default().fg(Colour::LightGreen)),
                TuiText::styled("Type a value to change data at the addresss\n", Style::default().fg(Colour::Gray)),
            ];

            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            para.draw(area, buf);       
        } else if self.mode >= 2 {
            let text = [
                TuiText::styled("Register Manipulation Help\n", Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40))),
                TuiText::styled("b to set breakpoint at reg address, w to set watchpoint\n", Style::default().fg(Colour::LightCyan)),
                TuiText::styled("rb to remove breakpoint at reg address, rw to remove watchpoint\n", Style::default().fg(Colour::LightRed)),
                TuiText::styled("j to jump to reg address\n", Style::default().fg(Colour::Magenta)),
                TuiText::styled("e to enter a new address, or type a register to change directly\n", Style::default().fg(Colour::LightGreen)),
                TuiText::styled("Type a value to change data in the reg\n", Style::default().fg(Colour::Gray)),
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
                self.input = self.input.to_lowercase();
                if self.input.len() == 2 {
                    if self.input.starts_with("r") {
                        self.input.remove(0);
                        match self.input.parse() {
                            Ok(r) => {
                                self.mode = 2;
                                self.reg_num = match r {
                                    0 => Reg::R0,
                                    1 => Reg::R1,
                                    2 => Reg::R2,
                                    3 => Reg::R3,
                                    4 => Reg::R4,
                                    5 => Reg::R5,
                                    6 => Reg::R6,
                                    7 => Reg::R7,
                                    _ => {self.mode = 0; Reg::R0},
                                };
                                data.log(format!("[Reg] {}\n", r), Colour::Green);
                            }
                            Err(e) => {data.log(format!("[Reg] {}\n", e), Colour::Red)}
                        }
                    } else if self.input == "pc" {
                        self.mode = 3;
                    } else if self.input == "psr" {
                        self.mode = 4;
                    }
                } else if self.mode == 0 {
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
                } else if self.mode == 1 {
                    if self.input == "b" {
                        match data.sim.set_breakpoint(self.mem_addr) {
                            Ok(val) => {
                                data.bp.insert(self.mem_addr, val);
                            }
                            Err(_e) => {},
                        }
                    } else if self.input == "w" {
                        match data.sim.set_memory_watchpoint(self.mem_addr) {
                            Ok(val) => {
                                data.wp.insert(self.mem_addr, val);
                            }
                            Err(_e) => {},
                        }

                    } else if self.input == "rb" {
                        match data.bp.remove(&self.mem_addr) {
                            Some(val) => {
                                data.sim.unset_breakpoint(val);
                            }
                            None => {},
                        }
                    } else if self.input == "rw" {
                        match data.wp.remove(&self.mem_addr) {
                            Some(val) => {
                                data.sim.unset_memory_watchpoint(val);
                            }
                            None => {},
                        }
                    } else if self.input == "j" {
                        //offset = data.sim.get_pc().wrapping_sub(self.mem_addr - 2);
                    } else if self.input == "e" {
                        self.mode = 0;
                    } else {
                        match self.input.parse::<Word>() {
                            Ok(w) => {
                                data.sim.write_word(self.mem_addr, w);
                            }
                            Err(_e) => {}
                        }
                        if self.input.len() > 2 {
                            let val = self.input.split_off(2);
                            if self.input == "0x" {
                                match Word::from_str_radix(&val, 16) {
                                    Ok(w) => {
                                        data.sim.write_word(self.mem_addr, w);
                                    }
                                    Err(_e) => {}
                                }   
                            } else if self.input == "0b" {
                                match Word::from_str_radix(&val, 2) {
                                    Ok(w) => {
                                        data.sim.write_word(self.mem_addr, w);
                                    }
                                    Err(_e) => {}
                                }
                            }
                        }
                    }
                } else if self.mode >= 2 {
                    self.input = self.input.to_lowercase();
                    let mut addr = 0;

                    if self.mode == 2 {
                        addr = data.sim.get_register(self.reg_num);
                    } else {
                        addr = data.sim.get_pc();
                    }

                    if self.input == "b" {
                        match data.sim.set_breakpoint(addr) {
                            Ok(val) => {
                                data.bp.insert(addr, val);
                            }
                            Err(_e) => {},
                        }
                    } else if self.input == "w" {
                        match data.sim.set_memory_watchpoint(addr) {
                            Ok(val) => {
                                data.wp.insert(addr, val);
                            }
                            Err(_e) => {},
                        }

                    } else if self.input == "rb" {
                        match data.bp.remove(&addr) {
                            Some(val) => {
                                data.sim.unset_breakpoint(val);
                            }
                            None => {},
                        }
                    } else if self.input == "rw" {
                        match data.wp.remove(&addr) {
                            Some(val) =>  {
                                data.sim.unset_memory_watchpoint(val);
                            }
                            None => {},
                        }
                    } else if self.input == "j" {
                        //offset = data.sim.get_pc().wrapping_sub(self.mem_addr - 2);
                    } else if self.input == "e" {
                        self.mode = 0;
                    } else {
                        if self.mode == 2 {
                            match self.input.parse::<Word>() {
                                Ok(w) => {
                                    data.sim.set_register(self.reg_num, w);
                                }
                                Err(_e) => {}
                            }
                            if self.input.len() > 2 {
                                let val = self.input.split_off(2);
                                if self.input == "0x" {
                                    match Word::from_str_radix(&val, 16) {
                                        Ok(w) => {
                                            data.sim.set_register(self.reg_num, w);
                                        }
                                        Err(_e) => {}
                                    }   
                                } else if self.input == "0b" {
                                    match Word::from_str_radix(&val, 2) {
                                        Ok(w) => {
                                            data.sim.set_register(self.reg_num, w);
                                        }
                                        Err(_e) => {}
                                    }
                                }
                            }
                        } else if self.mode == 3 {
                            match self.input.parse::<Word>() {
                                Ok(w) => {
                                    data.sim.set_pc(w);
                                }
                                Err(_e) => {}
                            }
                            if self.input.len() > 2 {
                                let val = self.input.split_off(2);
                                if self.input == "0x" {
                                    match Word::from_str_radix(&val, 16) {
                                        Ok(w) => {
                                            data.sim.set_pc(w);
                                        }
                                        Err(_e) => {}
                                    }   
                                } else if self.input == "0b" {
                                    match Word::from_str_radix(&val, 2) {
                                        Ok(w) => {
                                            data.sim.set_pc(w);
                                        }
                                        Err(_e) => {}
                                    }
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
