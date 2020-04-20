//! TODO!

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use lc3_isa::{Addr, Instruction, Reg, Word};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegDiff {
    old: ([Word; Reg::NUM_REGS], Word, Word),
    new: ([Word; Reg::NUM_REGS], Word, Word),
}

impl RegDiff {
    fn default() -> Self {
        Self {
            old: ([0; Reg::NUM_REGS], 0, 0),
            new: ([0; Reg::NUM_REGS], 0, 0),
        }
    }

    fn push(&mut self, stuff: ([Word; Reg::NUM_REGS], Word, Word)) {
        let (_, _, pc) = stuff;
        if pc != self.new.2 {
            self.old = self.new;
            self.new = stuff;
        }
    }

    fn diff(&self) -> ([Colour; Reg::NUM_REGS], Colour, Colour) {
        let mut colours = ([Colour::LightGreen; Reg::NUM_REGS], Colour::LightGreen, Colour::LightGreen);
        for i in 0..Reg::NUM_REGS {
            if self.old.0[i] != self.new.0[i] {
                colours.0[i] = Colour::Magenta;
            }
        }

        if self.old.1 != self.new.1 {
            colours.1 = Colour::Magenta;
        }
        colours
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Regs
{
    state: RegDiff,
    debug: bool,
}

impl Regs {
    pub fn default() -> Self {
        Self::new_with_debug(false)
    }

    pub fn new_with_debug(debug: bool) -> Self {
        Self {
            state: RegDiff::default(),
            debug,
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
        self.state.push(regs_psr_pc);
        let mut colours = self.state.diff();

        if self.debug && data.mem_reg_inter.0 == 2 {
            let reg = data.mem_reg_inter.1;
            if reg == 10 {
                colours.2 = Colour::Gray;
            } else {
                colours.0[reg as usize] = Colour::Gray;
            }
        }

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

        let mut reg_v = Vec::new();
        for i in 0..4 {
            let s = format!(
                "{:#018b} {:#06x} {:#05}\n",
                regs[i], regs[i], regs[i]
            );
            reg_v.push(TuiText::styled(s, Style::default().fg(colours.0[i])));
        }
        let s = format!("{:#018b} {:#06x} {:#05}\n", psr, psr, psr);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1)));

        para = Paragraph::new(reg_v.iter())
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

        reg_v.clear();
        for i in 4..8 {
            let s = format!(
                "{:#018b} {:#06x} {:#05}\n",
                regs[i], regs[i], regs[i]
            );
            reg_v.push(TuiText::styled(s, Style::default().fg(colours.0[i])));
        }
        let s = format!("{:#018b} {:#06x} {:#05}\n", pc, pc, pc);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.2)));

        para = Paragraph::new(reg_v.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(5, Axis::X, area);
        para.draw(area, buf);

    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => false,
            Focus(FocusEvent::LostFocus) => false,
            Mouse(MouseEvent::Up(_, _, _, _)) => false,
            Mouse(MouseEvent::Down(_, _, _, _)) => false,
            _ => false,
        }
    }
}
