//! A widget that does nothing but occupy space.
//!
//! Useful for testing and for blank spaces.

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use lc3_isa::{Addr, Instruction, Reg, Word};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mem 
{
    pub focusable: bool,
    offset: u16,
}

impl Default for Mem {
    fn default() -> Self {
        Self {
            focusable: true,
            offset: 2,
        }
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
        let pc = data.sim.get_pc();
        let mut mem: [Word; 50] = [0; 50];
        let mut x: u16 = 0;
        while x != 50 {
            mem[x as usize] = data.sim.read_word(pc.wrapping_sub(self.offset).wrapping_add(x));
            x = x + 1;
        }

        let mut pc_arrow = String::from("");
        let mut bp_locs = String::from("");
        let mut wp_locs = String::from("");
        let mut addresses = String::from("");
        let mut insts = String::from("");
        let mut s = String::from("");
        x = 0;

        while x != 50 {
            //let inst: Instruction = mem[x as usize];
            let inst = "TODO";

            let addr = pc.wrapping_sub(self.offset).wrapping_add(x);
            if x == self.offset {
                pc_arrow.push_str("-->\n");
            } else {
                pc_arrow.push_str("\n");
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
            s.push_str(&format!(
                "{:#018b} {:#06x} {:#05}\n",
                mem[x as usize], mem[x as usize], mem[x as usize]
            ));
            insts.push_str(&format!("{}\n", inst));
            x = x + 1;
        }

        let text = [TuiText::styled(
            "\n\n-->",
            Style::default().fg(Color::Rgb(0x73, 0xB7, 0xE8)),
        )];

        let mut para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);
        
        para.draw(area, buf);  

        let text = [TuiText::styled(
            pc_arrow,
            Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)),
        )];

        let mut para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);
        
        para.draw(area, buf);

        let text = [TuiText::styled(
            bp_locs,
            Style::default().fg(Color::Rgb(0xCC, 0x02, 0x02)),
        )];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        para.draw(area, buf);

        let text = [TuiText::styled(
            wp_locs,
            Style::default().fg(Color::Rgb(0x30, 0x49, 0xDE)),
        )];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        para.draw(area, buf);

        let text = [TuiText::styled(addresses, Style::default().fg(Color::Gray))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = Rect::new(area.x+5, area.y, area.width, area.height);
        para.draw(area, buf);

        let text = [TuiText::styled(s, Style::default().fg(Color::LightGreen))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(10, Axis::X, area);
        para.draw(area, buf);

        let text = [TuiText::styled(insts, Style::default().fg(Color::LightCyan))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(40, Axis::X, area);
        para.draw(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,

            Mouse(MouseEvent::ScrollUp(_, _, _)) => {
                self.offset = self.offset.wrapping_add(1);
                true
            }
            Mouse(MouseEvent::ScrollDown(_, _, _)) => {
                self.offset = self.offset.wrapping_sub(1);
                true
            }

            Key(KeyEvent { code: KeyCode::Up, modifiers: EMPTY }) => {
                self.offset = self.offset.wrapping_sub(1);
                true
            }
            Key(KeyEvent { code: KeyCode::Down, modifiers: EMPTY }) => {
                self.offset = self.offset.wrapping_add(1);
                true
            }

            Key(KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::SHIFT }) => {
                self.offset = self.offset.wrapping_sub(10);
                true
            }
            Key(KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::SHIFT }) => {
                self.offset = self.offset.wrapping_add(10);
                true
            }

            Key(KeyEvent { code: KeyCode::PageUp, modifiers: EMPTY }) => {
                self.offset = self.offset.wrapping_sub(50);
                true
            }
            Key(KeyEvent { code: KeyCode::PageDown, modifiers: EMPTY }) => {
                self.offset = self.offset.wrapping_add(50);
                true
            }

            Key(KeyEvent { code: KeyCode::Home, modifiers: EMPTY }) => {
                self.offset = 2;
                true
            }

            Key(KeyEvent { code: KeyCode::Char(w), modifiers: EMPTY }) => {
                let cur_addr = _data.sim.get_pc().wrapping_sub(self.offset).wrapping_add(2);
                match _data.wp.remove(&cur_addr) {
                    Some(val) => {_data.sim.unset_memory_watchpoint(val);},
                    None => {match _data.sim.set_memory_watchpoint(cur_addr) {
                        Ok(val) => {_data.wp.insert(cur_addr, val);},
                        Err(_e) => {},
                    }},
                };
                true
            }

            Key(KeyEvent { code: KeyCode::Char(b), modifiers: EMPTY }) => {
                let cur_addr = _data.sim.get_pc().wrapping_sub(self.offset).wrapping_add(2);
                match _data.bp.remove(&cur_addr) {
                    Some(val) => {_data.sim.unset_breakpoint(val);},
                    None => {match _data.sim.set_breakpoint(cur_addr) {
                        Ok(val) => {_data.bp.insert(cur_addr, val);},
                        Err(_e) => {},
                    }},
                };
                self.offset = _data.sim.get_pc().wrapping_sub(cur_addr - 2);
                true
            }

            _ => false,
        }
    }
}
