//! TODO!

use super::widget_impl_support::*;

use std::convert::TryInto;

use lc3_isa::{Addr, Instruction, Reg, Word};


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
        let mut bp_locs = String::from("");
        let mut wp_locs = String::from("");
        let mut addresses = String::from("");
        let mut insts = String::from("");
        let mut bin = String::from("");
        let mut hex = String::from("");
        let mut dec = String::from("");
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

            let addr = pc.wrapping_sub(self.offset).wrapping_add(x).wrapping_sub(self.focus);
            if x == self.offset.wrapping_add(self.focus) {
                let x = String::from("-->\n");
                arrow_v.push(TuiText::styled(x,Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40))));
            } else if x == self.offset {
                let x = String::from("-->\n");
                arrow_v.push(TuiText::styled(x,Style::default().fg(Colour::Cyan)));
            } else {
                arrow_v.push(TuiText::raw("\n"));
            }

            if data.bp.contains_key(&addr) {
                bp_locs.push_str("<b>\n");
            } else {
                bp_locs.push_str("\n");
            }

            if data.wp.contains_key(&addr) {
                wp_locs.push_str("<w>\n");
            } else {
                wp_locs.push_str("\n");
            }

            addresses.push_str(&format!(
                "{:#06x}\n",
                addr
            ));

            bin.push_str(&format!(
                "{:#018b}\n",
                mem[x as usize]
            ));

            hex.push_str(&format!(
                "{:#06x}\n",
                mem[x as usize]
            ));

            dec.push_str(&format!(
                "{:#05}\n",
                mem[x as usize]
            ));

            if inst_f {
                insts.push_str(&format!("{}\n", inst));
            } else {
                insts.push_str(&format!("\n"))
            }
            x = x + 1;
        }

        let mut para = Paragraph::new(arrow_v.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

        para.draw(area, buf);

        let text = [TuiText::styled(
            bp_locs,
            Style::default().fg(Colour::Rgb(0xCC, 0x02, 0x02)),
        )];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        if area.width < 4 {
            return
        }
        para.draw(area, buf);

        let text = [TuiText::styled(
            wp_locs,
            Style::default().fg(Colour::Rgb(0x30, 0x49, 0xDE)),
        )];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        if area.width < 4 {
            return
        }
        para.draw(area, buf);

        let text = [TuiText::styled(addresses, Style::default().fg(Colour::Gray))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        if area.width < 8 {
            return
        }
        para.draw(area, buf);

        let text = [TuiText::styled(bin, Style::default().fg(Colour::LightGreen))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(false);

        let area = increment(10, Axis::X, area);
        if area.width < 19 {
            return
        }
        para.draw(area, buf);

        let text = [TuiText::styled(hex, Style::default().fg(Colour::LightGreen))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(false);

        let area = increment(19, Axis::X, area);
        if area.width < 7 {
            return
        }
        para.draw(area, buf);

        let text = [TuiText::styled(dec, Style::default().fg(Colour::LightGreen))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(false);

        let area = increment(7, Axis::X, area);
        if area.width < 5 {
            return
        }
        para.draw(area, buf);

        let text = [TuiText::styled(insts, Style::default().fg(Colour::LightCyan))];

        para = Paragraph::new(text.iter())
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
                Some(val) => {data.sim.unset_breakpoint(val);},
                None => {match data.sim.set_breakpoint(cur_addr) {
                    Ok(val) => {data.bp.insert(cur_addr, val);},
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
                Some(val) => {data.sim.unset_memory_watchpoint(val);},
                None => {match data.sim.set_memory_watchpoint(cur_addr) {
                    Ok(val) => {data.wp.insert(cur_addr, val);},
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

            Mouse(MouseEvent::ScrollUp(_, _, _)) => {
                self.scroll_up(1);
                true
            }
            Mouse(MouseEvent::ScrollDown(_, _, _)) => {
                self.scroll_down(1);
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
                true
            }
            Key(KeyEvent { code: KeyCode::PageDown, modifiers: EMPTY }) => {
                self.focus = self.focus.wrapping_sub(self.position.height).wrapping_add(1);
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
