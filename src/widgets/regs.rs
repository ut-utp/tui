//! A widget that does nothing but occupy space.
//!
//! Useful for testing and for blank spaces.

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use lc3_isa::{Addr, Instruction, Reg, Word};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Regs 
{
}

impl Default for Regs {
    fn default() -> Self {
        Self {
        }
    }
}

impl TuiWidget for Regs
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Regs
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let regs_psr_pc = data.sim.get_registers_psr_and_pc();
        let (regs, psr, pc) = regs_psr_pc;

        let text = [
            TuiText::styled("R0:\nR1:\nR2:\nR3:\n", Style::default().fg(Color::Gray)),
            TuiText::styled("PSR:\n", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);

        let mut s = String::from("");
        for i in 0..4 {
            s.push_str(&format!(
                "{:#018b} {:#06x} {:#05}\n",
                regs[i], regs[i], regs[i]
            ));
        }
        s.push_str(&format!("{:#018b} {:#06x} {:#05}\n", psr, psr, psr));

        let text = [TuiText::styled(s, Style::default().fg(Color::LightGreen))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        para.draw(area, buf);

        let text = [
            TuiText::styled("R4:\nR5:\nR6:\nR7:\n", Style::default().fg(Color::Gray)),
            TuiText::styled("PC:\n", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
        ];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(40, Axis::X, area);
        para.draw(area, buf);

        s = String::from("");
        for i in 4..8 {
            s.push_str(&format!(
                "{:#018b} {:#06x} {:#05}\n",
                regs[i], regs[i], regs[i]
            ));
        }
        s.push_str(&format!("{:#018b} {:#06x} {:#05}\n", pc, pc, pc));

        let text = [TuiText::styled(s, Style::default().fg(Color::LightGreen))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        para.draw(area, buf);
        
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,
            _ => false,
        }
    }
}
