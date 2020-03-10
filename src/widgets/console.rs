//! TODO!

use super::widget_impl_support::*;

use tui::widgets::{Text as TuiText, Paragraph};
use tui::style::{Color, Style};
use tui::layout::Alignment;

use lc3_isa::{Addr, Instruction, Reg, Word};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Console
{
    history: String,
    input: String,
}

impl Default for Console {
    fn default() -> Self {
        Self {
            history: String::from(""),
            input: String::from(""),
        }
    }
}

impl TuiWidget for Console
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Console
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let Console_psr_pc = data.sim.get_registers_psr_and_pc();
        let (Console, psr, pc) = Console_psr_pc;

        /*let text = [
            TuiText::styled("R0:\nR1:\nR2:\nR3:\n", Style::default().fg(Color::Gray)),
            TuiText::styled("PSR:\n", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40))),
        ];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf);*/

        let text = [TuiText::styled(">", Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];

        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(true);


        if area.height <= 1 {
            para.draw(area, buf);
        } else if area.height <= 4 {
            let area = Rect::new(area.x, area.y+area.height/2, area.width, 3);
            para.draw(area, buf);

            let text = [TuiText::styled(self.input.clone(), Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(1, Axis::Y, area);
            if area.height < 2 {
                return;
            }
            para.draw(area,buf);
        } else {
            let area = Rect::new(area.x, area.y+area.height-3, area.width, 3);
            para.draw(area, buf);

            let text = [TuiText::styled(self.input.clone(), Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true);

            let area = increment(1, Axis::Y, area);
            if area.height < 2 {
                return;
            }
            para.draw(area,buf);
        }

    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                let x = format!("{}", c);
                self.input.push_str(&x);
                true
            }

            Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                self.input = String::from("");
                true
            }
             _ => false,
        }
    }
}
