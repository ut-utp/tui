//! TODO!

use super::widget_impl_support::*;

use std::convert::TryInto;

use lc3_isa::{Addr, Instruction, Reg, Word};
use lc3_traits::control::control::{Event};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mem
{
    offset: u16,
    focus: u16,
    position: Rect,
    reset_flag: u8,
    addr: Addr,
    follow: bool,
    debug: (bool, u8)
}

impl Mem {
    pub fn default() -> Self {
        Self::new_with_debug(false)
    }

    pub fn new_with_debug(toggle: bool) -> Self {
        Self {
            offset: 2,
            focus: 0,
            position: Rect::new(0,0,0,0),
            reset_flag:0,
            addr: 0x200,
            follow: true,
            debug: (toggle, 0),
        }
    }

    pub fn scroll_up(&mut self, increment: u16) {
        self.offset = self.offset.saturating_sub(increment);
        self.focus = self.focus.wrapping_add(increment);
        self.addr = self.addr.wrapping_sub(increment);
        self.follow = false;
    }

    pub fn scroll_down(&mut self, increment: u16) {
        self.offset = self.offset.wrapping_add(increment);
        self.focus = self.focus.wrapping_sub(increment);
        self.addr = self.addr.wrapping_add(increment);
        self.follow = false;
    }

    pub fn home(&mut self) {
        self.offset = 2;
        self.focus = 0;
        self.follow = true;
    }
}

impl TuiWidget for Mem
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Mem
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        if self.reset_flag != data.reset_flag{
            self.home();
            self.reset_flag = data.reset_flag;
        }

        self.position = area;

        if self.offset > area.height.saturating_sub(1) {
            self.offset = area.height.saturating_sub(1);
        }

        if self.debug.0 && data.jump.0 != self.debug.1 {
            self.addr = data.jump.1;
            self.follow = false;
            self.debug.1 = data.jump.0;
        }

        let pc = data.sim.get_pc();

        if self.follow {
            self.addr = pc;
        } else {
            let position = pc.wrapping_sub(self.focus);
            let diff = self.addr.wrapping_sub(position);
            self.focus = self.focus.wrapping_sub(diff);
        }

        let mut mem: [Word; 50] = [0; 50];
        let mut x: u16 = 0;
        while x != 50 {
            mem[x as usize] = data.sim.read_word(pc.wrapping_sub(self.offset).wrapping_add(x).wrapping_sub(self.focus));
            x = x + 1;
        }

        let mut arrow_v = Vec::new();
        let mut bp_v = Vec::new();
        let mut wp_v = Vec::new();
        let mut addresses_v = Vec::new();
        let mut bin_v = Vec::new();
        let mut hex_v = Vec::new();
        let mut dec_v = Vec::new();
        let mut insts_v = Vec::new();
        x = 0;

        while x != 50 {
            let mut inst_f = true;
            let inst: Instruction = match mem[x as usize].try_into(){
                Ok(x) => x,
                Err(e) => {
                    inst_f = false;
                    Instruction::AddReg {
                        dr: Reg::R0,
                        sr1: Reg::R0,
                        sr2: Reg::R0,
                    }}
            };
            //let inst = "TODO";

            let cur_addr = pc.wrapping_sub(self.offset).wrapping_add(x).wrapping_sub(self.focus);
            if x == self.offset.wrapping_add(self.focus) {
                let x = String::from("-->\n");
                arrow_v.push(TuiText::styled(x,Style::default().fg(c!(PC))));
            } else if x == self.offset {
                let x = String::from("-->\n");
                arrow_v.push(TuiText::styled(x,Style::default().fg(c!(Highlight))));
            } else {
                arrow_v.push(TuiText::raw("\n"));
            }

            let mut bp_colour = c!(Breakpoint);
            let mut wp_colour = c!(Watchpoint);
            let mut addr_colour = c!(Name);
            let mut data_colour = c!(Data);
            let mut inst_colour = c!(Inst);

            if x == self.offset {
                if cur_addr == pc {
                    bp_colour = c!(PC);
                    wp_colour = c!(PC);
                    addr_colour = c!(PC);
                    data_colour = c!(PC);
                    inst_colour = c!(PC);
                } else {
                    bp_colour = c!(Highlight);
                    wp_colour = c!(Highlight);
                    addr_colour = c!(Highlight);
                    data_colour = c!(Highlight);
                    inst_colour = c!(Highlight);
                }
            }

            match data.get_current_event() {
                Some(event) => {
                    match event {
                        Event::Breakpoint {addr} => {
                            if cur_addr == pc {
                                bp_colour = c!(Breakpoint);
                                wp_colour = c!(Breakpoint);
                                addr_colour = c!(Breakpoint);
                                data_colour = c!(Breakpoint);
                                inst_colour = c!(Breakpoint);
                            }
                        }
                        Event::MemoryWatch {addr, data} => {
                            if addr == cur_addr {
                                bp_colour = c!(Highlight);
                                wp_colour = c!(Highlight);
                                addr_colour = c!(Highlight);
                                data_colour = c!(Highlight);
                                inst_colour = c!(Highlight);
                            }
                        }
                        Event::DepthReached {current_depth } => {     // TODO: maybe pick some other color for this?
                            if cur_addr == pc {
                                bp_colour = Colour::Red;
                                wp_colour = Colour::Red;
                                addr_colour = Colour::Red;
                                data_colour = Colour::Red;
                                inst_colour = Colour::Red;
                            }
                        }
                        Event::Error {err} => {
                            if cur_addr == pc {
                                bp_colour = c!(Error);
                                wp_colour = c!(Error);
                                addr_colour = c!(Error);
                                data_colour = c!(Error);
                                inst_colour = c!(Error);
                            }
                        },
                        Event::Interrupted => {
                            if cur_addr == pc {
                                bp_colour = c!(Pause);
                                wp_colour = c!(Pause);
                                addr_colour = c!(Pause);
                                data_colour = c!(Pause);
                                inst_colour = c!(Pause);
                            }
                        }
                        Event::Halted => {
                            if cur_addr == pc {
                                bp_colour = c!(Halted);
                                wp_colour = c!(Halted);
                                addr_colour = c!(Halted);
                                data_colour = c!(Halted);
                                inst_colour = c!(Halted);
                            }
                        }
                        _ => {}
                    }
                },
                None => {}
            };

            if self.debug.0 && data.mem_reg_inter.0 == 1 {
                if cur_addr == data.mem_reg_inter.1 {
                    bp_colour = c!(MemRegHighlight);
                    wp_colour = c!(MemRegHighlight);
                    addr_colour = c!(MemRegHighlight);
                    data_colour = c!(MemRegHighlight);
                    inst_colour = c!(MemRegHighlight);
                }
            }

            if data.bp.contains_key(&cur_addr) {
                bp_v.push(TuiText::styled("<b>\n", Style::default().fg(bp_colour)));
            } else {
                bp_v.push(TuiText::raw("\n"));
            }

            if data.wp.contains_key(&cur_addr) {
                wp_v.push(TuiText::styled("<w>\n", Style::default().fg(wp_colour)));
            } else {
                wp_v.push(TuiText::raw("\n"));
            }

            let s = format!("{:#06x}\n", cur_addr);
            addresses_v.push(TuiText::styled(s, Style::default().fg(addr_colour)));

            let s = format!("{:#018b}\n", mem[x as usize]);
            bin_v.push(TuiText::styled(s, Style::default().fg(data_colour)));

            let s = format!("{:#06x}\n", mem[x as usize]);
            hex_v.push(TuiText::styled(s, Style::default().fg(data_colour)));

            let s = format!("{:#05}\n", mem[x as usize]);
            dec_v.push(TuiText::styled(s, Style::default().fg(data_colour)));

            if inst_f {
                let s = format!("{}\n", inst);
                insts_v.push(TuiText::styled(s, Style::default().fg(inst_colour)));
            } else {
                insts_v.push(TuiText::raw("\n"))
            }
            x = x + 1;
        }

        let mut para = Paragraph::new(arrow_v.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

        para.draw(area, buf);

        para = Paragraph::new(bp_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        if area.width < 4 {
            return
        }
        para.draw(area, buf);

        para = Paragraph::new(wp_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        if area.width < 4 {
            return
        }
        para.draw(area, buf);

        para = Paragraph::new(addresses_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        if area.width < 8 {
            return
        }
        para.draw(area, buf);

        para = Paragraph::new(bin_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(false);

        let area = increment(10, Axis::X, area);
        if area.width < 19 {
            return
        }
        para.draw(area, buf);

        para = Paragraph::new(hex_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(false);

        let area = increment(19, Axis::X, area);
        if area.width < 7 {
            return
        }
        para.draw(area, buf);

        para = Paragraph::new(dec_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(false);

        let area = increment(7, Axis::X, area);
        if area.width < 5 {
            return
        }
        para.draw(area, buf);

        para = Paragraph::new(insts_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(8, Axis::X, area);
        if area.width < 20 {
            return
        }
        para.draw(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;

        fn set_bp<'a, 'int, C, I, O>(offset: u16, data: &mut TuiData<'a, 'int, C, I, O>)
        where
            C: Control + ?Sized + 'a,
            I: InputSink + ?Sized + 'a,
            O: OutputSource + ?Sized + 'a,
        {
            let cur_addr = data.sim.get_pc().wrapping_sub(offset);
            match data.bp.remove(&cur_addr) {
                Some(val) => {data.sim.unset_breakpoint(val as u8);},
                None => {match data.sim.set_breakpoint(cur_addr) {
                    Ok(val) => {data.bp.insert(cur_addr, val as usize);},
                    Err(_e) => {},
                }},
            };
        }

        fn set_wp<'a, 'int, C, I, O>(offset: u16, data: &mut TuiData<'a, 'int, C, I, O>)
        where
            C: Control + ?Sized + 'a,
            I: InputSink + ?Sized + 'a,
            O: OutputSource + ?Sized + 'a,
        {
            let cur_addr = data.sim.get_pc().wrapping_sub(offset);
            match data.wp.remove(&cur_addr) {
                Some(val) => {data.sim.unset_memory_watchpoint(val as u8);},
                None => {match data.sim.set_memory_watchpoint(cur_addr) {
                    Ok(val) => {data.wp.insert(cur_addr, val as usize);},
                    Err(_e) => {},
                }},
            };
        }
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(button, x, y, _)) => {
                true
            }

            Mouse(MouseEvent::Down(button, x, y, _)) => {
                let x = x.wrapping_sub(self.position.x);
                let y = y.wrapping_sub(self.position.y);
                if (4 <= x) && (x <= 8) {
                    set_bp(self.focus.wrapping_sub(y).wrapping_add(self.offset), data);
                } else if (9 <= x) && (x <= 13) {
                    set_wp(self.focus.wrapping_sub(y).wrapping_add(self.offset), data)
                } else if (15 <= x) && (x <= 55) {
                    self.focus = self.focus.wrapping_add(self.offset).wrapping_sub(y);
                    if y > 60{
                        self.offset = 0;
                    } else {
                        self.offset = y;
                    }
                    self.follow = false;
                    self.addr = data.sim.get_pc().wrapping_sub(self.focus);
                }
                true
            }

            Mouse(MouseEvent::ScrollUp(_, _, modif)) => {
                match modif {
                    KeyModifiers::SHIFT => self.scroll_up(5),
                    KeyModifiers::CONTROL =>self.scroll_up(10),
                    KeyModifiers::ALT => self.scroll_up(20),
                    _ => self.scroll_up(1),
                }

                true
            }
            Mouse(MouseEvent::ScrollDown(_, _, modif)) => {
                match modif {
                    KeyModifiers::SHIFT => self.scroll_down(5),
                    KeyModifiers::CONTROL =>self.scroll_down(10),
                    KeyModifiers::ALT => self.scroll_down(20),
                    _ => self.scroll_down(1),
                }
                true
            }

            Key(KeyEvent { code: KeyCode::Up, modifiers: EMPTY }) => {
                self.scroll_up(1);
                true
            }
            Key(KeyEvent { code: KeyCode::Down, modifiers: EMPTY }) => {
                self.scroll_down(1);
                true
            }

            Key(KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::SHIFT }) => {
                self.scroll_up(10);
                true
            }
            Key(KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::SHIFT }) => {
                self.scroll_down(10);
                true
            }

            Key(KeyEvent { code: KeyCode::PageUp, modifiers: EMPTY }) => {
                self.focus = self.focus.wrapping_add(self.position.height).wrapping_sub(1);
                self.follow = false;
                self.addr = self.addr.wrapping_add(self.position.height).wrapping_sub(1);
                true
            }
            Key(KeyEvent { code: KeyCode::PageDown, modifiers: EMPTY }) => {
                self.focus = self.focus.wrapping_sub(self.position.height).wrapping_add(1);
                self.follow = false;
                self.addr = self.addr.wrapping_sub(self.position.height).wrapping_add(1);
                true
            }

            Key(KeyEvent { code: KeyCode::Home, modifiers: EMPTY }) => {
                self.home();
                true
            }

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                match c {
                    'w' => {
                        set_wp(self.focus, data);
                        true
                    }

                    'b' => {
                        set_bp(self.focus, data);
                        true
                    }

                    'h' => {
                        self.home();
                        true
                    }

                    _ => false
                }

            }

            _ => false,
        }
    }
}
