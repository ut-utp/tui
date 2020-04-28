//! TODO!

use super::widget_impl_support::*;

use lc3_isa::{Addr, Instruction, Reg, Word};
use std::convert::TryInto;

use lc3_traits::control::control::{Event,ProcessorMode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackWindow
{
}

impl Default for StackWindow {
    fn default() -> Self {
        Self {
        }
    }
}

impl TuiWidget for StackWindow
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for StackWindow
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let mut addr_v = Vec::new();
        let mut frame_v = Vec::new();
        let mut mode_v = Vec::new();

        let call_stack = data.sim.get_call_stack();
        let mut i = 0;

        frame_v.push(TuiText::styled("#\n",Style::default().fg(c!(NumT))));
        addr_v.push(TuiText::styled("Address\n",Style::default().fg(c!(AddrT))));
        mode_v.push(TuiText::styled("Mode\n",Style::default().fg(c!(DataT))));

        while let Some(frame) = call_stack[i] {
            let x = format!("{}\n", i);
            frame_v.push(TuiText::styled(x,Style::default().fg(c!(Num))));
            let x = format!("{:#06x}\n", frame.0);
            addr_v.push(TuiText::styled(x,Style::default().fg(c!(Addr))));
            let x = match frame.1 {
                ProcessorMode::Supervisor => "S\n",
                ProcessorMode::User => "U\n",
            };
            mode_v.push(TuiText::styled(x,Style::default().fg(c!(Privilege))));
            i = i + 1;
        }

        let mut para = Paragraph::new(frame_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let area = increment(5, Axis::X, area);
        para = Paragraph::new(addr_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);

        let area = increment(10, Axis::X, area);
        para = Paragraph::new(mode_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.draw(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        match event {
             _ => false,
        }
    }
}
