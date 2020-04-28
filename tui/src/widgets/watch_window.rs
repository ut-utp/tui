//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Instruction, Reg, Word};
use std::convert::TryInto;

use lc3_traits::control::control::{Event};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchWindow
{
    highlight: u16,
    highlight_addr: Addr,
    wp_len: u16,
    position: Rect,
}

impl Default for WatchWindow {
    fn default() -> Self {
        Self {
            highlight: 200,
            highlight_addr: 0,
            wp_len: 0,
            position: Rect::new(0,0,0,0),
        }
    }
}

impl TuiWidget for WatchWindow
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for WatchWindow
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        self.position = area;
        let mut flag = false;

        let num_wps: u16 = data.wp.len().try_into().unwrap();
        if self.wp_len != num_wps {
            if self.highlight != 200 {
                flag = true;
                self.highlight = 200;
            }
            self.wp_len = num_wps;
        }

        let mut event_flag = false;
        let mut event_addr: Addr = 0;
        if let Some(Event::MemoryWatch{addr, data}) = data.get_current_event() {
            event_flag = true;
            event_addr = addr;
        }

        let mut t_i = Vec::new();
        let mut t_a = Vec::new();
        let mut t_v = Vec::new();
        let mut event_highlight = 200;
        let mut i = 0;
        let mut vec = Vec::new();

        t_i.push(TuiText::styled("#\n",Style::default().fg(c!(NumT))));
        t_a.push(TuiText::styled("Address\n",Style::default().fg(c!(AddrT))));
        t_v.push(TuiText::styled("Data\n",Style::default().fg(c!(DataT))));

        for (wp_addr, _) in data.wp.iter() {
            vec.push(*wp_addr);
        }
        vec.sort();

        for (wp_addr) in vec.iter() {
            if flag && *wp_addr == self.highlight_addr {
                self.highlight = i;
                flag = false;
            }

            if event_flag && *wp_addr == event_addr {
                event_highlight = i;
                event_flag = false;
            }

            if i == event_highlight && i == self.highlight {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(BWHighlight))));
                let x = format!("{:#06x}\n",wp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(BWHighlight))));
                let x = format!("{:#018b} {:#06x} {:#05}\n",data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr));
                t_v.push(TuiText::styled(x, Style::default().fg(c!(BWHighlight))));
                self.highlight_addr = *wp_addr;
            } else if i == event_highlight {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(Watchpoint))));
                let x = format!("{:#06x}\n",wp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(Watchpoint))));
                let x = format!("{:#018b} {:#06x} {:#05}\n",data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr));
                t_v.push(TuiText::styled(x, Style::default().fg(c!(Watchpoint))));
            } else if i == self.highlight {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(Highlight))));
                let x = format!("{:#06x}\n",wp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(Highlight))));
                let x = format!("{:#018b} {:#06x} {:#05}\n",data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr));
                t_v.push(TuiText::styled(x, Style::default().fg(c!(Highlight))));
                self.highlight_addr = *wp_addr;
            } else {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(Num))));
                let x = format!("{:#06x}\n",wp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(Addr))));
                let x = format!("{:#018b} {:#06x} {:#05}\n",data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr), data.sim.read_word(*wp_addr));
                t_v.push(TuiText::styled(x, Style::default().fg(c!(Data))));
            }

            i = i + 1;
        }

        let mut para = Paragraph::new(t_i.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let area = increment(5, Axis::X, area);
        para = Paragraph::new(t_a.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let area = increment(10, Axis::X, area);
        para = Paragraph::new(t_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, y, _)) => {
                let y = y.wrapping_sub(self.position.y).wrapping_sub(1);
                self.highlight = y;
                true
            }

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                if(c.is_digit(10)){
                    self.highlight = c.to_digit(10).unwrap().try_into().unwrap();
                }
                true
            }

            Key(KeyEvent { code: KeyCode::Backspace, modifiers: EMPTY }) => {
                if self.highlight < self.wp_len {
                    match data.wp.remove(&self.highlight_addr) {
                        Some(val) =>  {data.sim.unset_memory_watchpoint(val as u8);/*self.mode = 0;*/},
                        None => {},
                    };
                }
                true
            }
             _ => false,
        }
    }
}
