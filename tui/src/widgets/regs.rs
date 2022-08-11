//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Reg, Word, Bits};

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

    fn diff(&self) -> ([Colour; Reg::NUM_REGS], [Colour; 5], Colour) {
        let mut colours = ([c!(Data); Reg::NUM_REGS], [c!(Privilege), c!(Priority), c!(n_bit), c!(z_bit), c!(p_bit)], c!(Data));
        for i in 0..Reg::NUM_REGS {
            if self.old.0[i] != self.new.0[i] {
                colours.0[i] = c!(RegHighlight);
            }
        }

        if self.old.1.bit(15) != self.new.1.bit(15){
            colours.1[0] = c!(RegHighlight);
        }

        if self.old.1.bits(8..10) != self.new.1.bits(8..10){
            colours.1[1] = c!(RegHighlight);
        }

        if self.old.1.bit(2) != self.new.1.bit(2){
            colours.1[2] = c!(RegHighlight);
        }

        if self.old.1.bit(1) != self.new.1.bit(1){
            colours.1[3] = c!(RegHighlight);
        }

        if self.old.1.bit(0) != self.new.1.bit(0){
            colours.1[4] = c!(RegHighlight);
        }

        colours
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Regs
{
    state: RegDiff,
    debug: bool,
    reset_flag: u8,
}

impl Regs {
    pub fn default() -> Self {
        Self::new_with_debug(false)
    }

    pub fn new_with_debug(debug: bool) -> Self {
        Self {
            state: RegDiff::default(),
            debug,
            reset_flag: 0,
        }
    }
}


impl<Wt: WidgetTypes> Widget<Wt> for Regs {
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        let regs_psr_pc = data.sim.get_registers_psr_and_pc();
        let (regs, psr, pc) = regs_psr_pc;
        let privilege = psr.bit(15) as u8;
        let priority = psr.bits(8..10);
        let pri1 = psr.bit(8) as u8;
        let pri2 = psr.bit(9) as u8;
        let pri3 = psr.bit(10) as u8;
        let n = psr.bit(2) as u8;
        let z = psr.bit(1) as u8;
        let p = psr.bit(0) as u8;

        if self.reset_flag != data.reset_flag{
            self.state.push((regs, psr, pc-1));
            self.reset_flag = data.reset_flag;
        }

        self.state.push(regs_psr_pc);

        let mut colours = self.state.diff();

        if self.debug && data.mem_reg_inter.0 == 2 {
            let reg = data.mem_reg_inter.1;
            if reg == 10 {
                colours.2 = c!(MemRegHighlight);
            } else {
                colours.0[reg as usize] = c!(MemRegHighlight);
            }
        }

        let text = [
            TuiText::styled("R0:\nR1:\nR2:\nR3:\n", Style::default().fg(c!(Name))),
            TuiText::styled("PSR:\nMode:\nPri:\n", Style::default().fg(c!(Pc))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf);

        let mut reg_v = Vec::new();
        for i in 0..4 {
            let s = format!(
                "{:#018b} {:#06x} {:#05}\n",
                regs[i], regs[i], regs[i]
            );
            reg_v.push(TuiText::styled(s, Style::default().fg(colours.0[i])));
        }

        reg_v.push(TuiText::styled("0b", Style::default().fg(c!(Data))));
        let s = format!("{}", privilege);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[0])));
        reg_v.push(TuiText::styled("0000", Style::default().fg(c!(Data))));
        let s = format!("{}{}{}", pri3, pri2, pri1);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[1])));
        reg_v.push(TuiText::styled("00000", Style::default().fg(c!(Data))));
        let s = format!("{}", n);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[2])));
        let s = format!("{}", z);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[3])));
        let s = format!("{}\n", p);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[4])));

        let s = match privilege {
            0 => "Supervisor\n",
            1 => "User\n",
            _ => unreachable!(),
        };
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[0])));

        let s = format!("{}\n", priority);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[1])));

        para = Paragraph::new(reg_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(6, Axis::X, area);
        para.render(area, buf);

        let text = [
            TuiText::styled("R4:\nR5:\nR6:\nR7:\n", Style::default().fg(c!(Name))),
            TuiText::styled("PC:\nnzp: ", Style::default().fg(c!(Pc))),
        ];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(40, Axis::X, area);
        para.render(area, buf);

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
        reg_v.push(TuiText::styled(format!("n: "), Style::default().fg(c!(Pc))));
        let s = format!("{}  ", n);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[2])));
        reg_v.push(TuiText::styled(format!("z: "), Style::default().fg(c!(Pc))));
        let s = format!("{}  ", z);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[3])));
        reg_v.push(TuiText::styled(format!("p: "), Style::default().fg(c!(Pc))));
        let s = format!("{}  ", p);
        reg_v.push(TuiText::styled(s, Style::default().fg(colours.1[4])));

        para = Paragraph::new(reg_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        let area = increment(6, Axis::X, area);
        para.render(area, buf);

    }

    fn update(&mut self, event: WidgetEvent, _data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
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
