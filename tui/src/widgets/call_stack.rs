//! TODO!

use super::widget_impl_support::*;

use lc3_traits::control::control::ProcessorMode;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct StackWindow;

impl<Wt: WidgetTypes> Widget<Wt> for StackWindow {
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
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
            let (x, style) = match frame.1 {
                ProcessorMode::Supervisor => ("S\n", c!(CallStackSupervisorMode)),
                ProcessorMode::User => ("U\n", c!(CallStackUserMode)),
            };
            mode_v.push(TuiText::styled(x,Style::default().fg(style)));
            i = i + 1;
        }

        let mut para = Paragraph::new(frame_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.render(area, buf);

        let area = increment(5, Axis::X, area);
        para = Paragraph::new(addr_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.render(area, buf);

        let area = increment(10, Axis::X, area);
        para = Paragraph::new(mode_v.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);
        para.render(area, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
        match event {
            _ => false,
        }
    }
}
