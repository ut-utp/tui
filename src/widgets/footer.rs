//! TODO!

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Footer
{
}

impl Default for Footer {
    fn default() -> Self {
        Self {
        }
    }
}

impl TuiWidget for Footer
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Footer
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let text = [
            TuiText::styled("To control the TUI, you can use ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("S to Step ", Style::default().fg(Color::LightCyan)),
            TuiText::styled("through instructions, ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("P to Pause, ", Style::default().fg(Color::LightRed)),
            TuiText::styled("R to Run, ", Style::default().fg(Color::LightYellow)),
            TuiText::styled("and ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("Q to Quit\n", Style::default().fg(Color::Gray)),
            TuiText::styled("To set the peripherals use ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("Ctrl + ", Style::default().fg(Color::White)),
            TuiText::styled("g for GPIO, ", Style::default().fg(Color::Rgb(0xee, 0xee, 0xee))),
            TuiText::styled("a for ADC, ", Style::default().fg(Color::Rgb(0xdd, 0xdd, 0xdd))),
            TuiText::styled("p for PWM, ", Style::default().fg(Color::Rgb(0xcc, 0xcc, 0xcc))),
            TuiText::styled("t for Timers, ", Style::default().fg(Color::Rgb(0xbb, 0xbb, 0xbb))),
            TuiText::styled("and ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("c for CLK\n", Style::default().fg(Color::Rgb(0xaa, 0xaa, 0xaa))),
            TuiText::styled("To affect the simulator, use ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("Alt + ", Style::default().fg(Color::White)),
            TuiText::styled("p for PC, ", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
            TuiText::styled("m for Memory, ", Style::default().fg(Color::LightCyan)),
            TuiText::styled("and ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("r to reset\n", Style::default().fg(Color::Gray)),
            TuiText::styled("To control memory, use ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("UP and DOWN ", Style::default().fg(Color::Gray)),
            TuiText::styled("arrow keys. ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("Shift + arrow ", Style::default().fg(Color::Gray)),
            TuiText::styled("jumps 10, ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("Control + arrow ", Style::default().fg(Color::Gray)),
            TuiText::styled("jumps 100. ", Style::default().fg(Color::LightGreen)),
            TuiText::styled("Ctrl + h returns to PC\n", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))
        ];

        // TODO: allow parameterization of this in the usual way.
        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,
            _ => false,
        }
    }
}
