//! TODO!

use super::widget_impl_support::*;

use lc3_isa::Addr;
use lc3_traits::control::control::Event;

use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakWindow {
    highlight: u16,
    highlight_addr: Addr,
    bp_len: u16,
    position: Rect,
}

impl Default for BreakWindow {
    fn default() -> Self {
        Self {
            highlight: 200,
            highlight_addr: 0,
            bp_len: 0,
            position: Rect::new(0,0,0,0),
        }
    }
}

impl<Wt: WidgetTypes> Widget<Wt> for BreakWindow {
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        self.position = area;
        let mut flag = false;
        let mut event_flag = false;

        let num_bps: u16 = data.bp.len().try_into().unwrap();
        if self.bp_len != num_bps {
            if self.highlight != 200 {
                flag = true;
                self.highlight = 200;
            }
            self.bp_len = num_bps;
        }

        let mut event_addr: Addr = 0;
        if let Some(Event::Breakpoint{addr}) = data.get_current_event() {
            event_flag = true;
            event_addr = addr;
        }


        let mut t_i = Vec::new();
        let mut t_a = Vec::new();

        t_i.push(TuiText::styled("#\n",Style::default().fg(c!(NumT))));
        t_a.push(TuiText::styled("Address\n",Style::default().fg(c!(AddrT))));

        let mut event_highlight = 200;
        let mut i = 0;

        let mut vec = Vec::new();
        for (bp_addr, _) in data.bp.iter() {
            vec.push(*bp_addr);
        }
        vec.sort();


        for bp_addr in vec.iter() {
            if flag && *bp_addr == self.highlight_addr {
                self.highlight = i;
                flag = false;
            }

            if event_flag && *bp_addr == event_addr {
                event_highlight = i;
                event_flag = false;
            }

            if i == event_highlight && i == self.highlight {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(BWHighlight))));
                let x = format!("{:#06x}\n",bp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(BWHighlight))));
                self.highlight_addr = *bp_addr;
            } else if i == event_highlight {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(Breakpoint))));
                let x = format!("{:#06x}\n",bp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(Breakpoint))));
            } else if i == self.highlight {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(Highlight))));
                let x = format!("{:#06x}\n",bp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(Highlight))));
                self.highlight_addr = *bp_addr;
            } else {
                let x = format!("{}\n", i);
                t_i.push(TuiText::styled(x,Style::default().fg(c!(Num))));
                let x = format!("{:#06x}\n",bp_addr);
                t_a.push(TuiText::styled(x, Style::default().fg(c!(Addr))));
            }

            i = i + 1;
        }

        let mut para = Paragraph::new(t_i.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.render(area, buf);

        let area = increment(5, Axis::X, area);
        para = Paragraph::new(t_a.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.render(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
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
                if c.is_digit(10) {
                    self.highlight = c.to_digit(10).unwrap().try_into().unwrap();
                }
                true
            }

            Key(KeyEvent { code: KeyCode::Backspace, modifiers: EMPTY }) => {
                if self.highlight < self.bp_len {
                    match data.bp.remove(&self.highlight_addr) {
                        Some(val) =>  {data.sim.unset_breakpoint(val as u8);/*self.mode = 0;*/},
                        None => {},
                    };
                }
                true
            }
             _ => false,
        }
    }
}
