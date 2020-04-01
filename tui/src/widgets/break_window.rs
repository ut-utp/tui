//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Instruction, Reg, Word};
use std::convert::TryInto;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakWindow
{
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

impl TuiWidget for BreakWindow
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for BreakWindow
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        self.position = area;
        let mut flag = false;
        if self.bp_len != data.bp.len().try_into().unwrap() {
            if self.highlight != 200 {
                flag = true;
                self.highlight = 200;
            }
            self.bp_len = data.bp.len().try_into().unwrap();
        }

        let mut bp_indices = String::from("");
        let mut addresses = String::from("");
        let mut index_highlight = String::from("");
        let mut addr_highlight = String::from("");
        let mut i = 0;

        for (bp_addr, index) in data.bp.iter() {
            if flag && *bp_addr == self.highlight_addr {
                self.highlight = i;
                flag = false;
            }

            if i != self.highlight {
                bp_indices.push_str(&format!(
                    "{}\n",
                    i
                ));

                addresses.push_str(&format!(
                    "{:#06x}\n",
                    bp_addr
                ));

                index_highlight.push_str("\n");
                addr_highlight.push_str("\n");
            } else {
                index_highlight.push_str(&format!(
                    "{}\n",
                    i
                ));

                addr_highlight.push_str(&format!(
                    "{:#06x}\n",
                    bp_addr
                ));

                self.highlight_addr = *bp_addr;

                bp_indices.push_str("\n");
                addresses.push_str("\n");
            }

            i = i + 1;
        }

        let text = [TuiText::styled(bp_indices,Style::default().fg(Colour::Rgb(0xFF, 0x97, 0x40)))];
        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let text = [TuiText::styled(index_highlight,Style::default().fg(Colour::Cyan))];
        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let area = increment(5, Axis::X, area);
        let text = [TuiText::styled(addresses, Style::default().fg(Colour::Gray))];
        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let text = [TuiText::styled(addr_highlight,Style::default().fg(Colour::Cyan))];
        let mut para = Paragraph::new(text.iter())
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
                let y = y.wrapping_sub(self.position.y);
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
                if self.highlight < self.bp_len {
                    match data.bp.remove(&self.highlight_addr) {
                        Some(val) =>  {data.sim.unset_breakpoint(val);/*self.mode = 0;*/},
                        None => {},
                    };
                }
                true
            }
             _ => false,
        }
    }
}
