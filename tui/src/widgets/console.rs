//! TODO!

use super::widget_impl_support::*;

use std::convert::TryInto;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Console
{
}

impl Default for Console {
    fn default() -> Self {
        Self {
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
        if matches!((data.output, data.input), (None, None)) {
            return Block::default()
                .style(Style::default().bg(Colour::Gray).fg(Colour::Gray))
                .draw(area, buf);
        }

        // TODO(rrbutani): redo this to use the text container.

        // Append any new output we have:
        if let Some(out) = data.output {
            if let Some(s) = out.get_chars() {
                data.console_hist.borrow_mut().push(s);
            }
        }

        let console_output = match data.output{ // collect from the output source
            Some(output) => {
                match output.get_chars() {
                    Some(s) => {
                        s
                    },
                    None => {
                       "".to_string()
                    },

                }
            },
           None => {
               "".to_string()
           }

        };
        if console_output != "" {
            //let vector = RefCell::new(data.history_vec);
           data.console_hist.borrow_mut().push(console_output); // collect from output source
        }

        let mut bottom_area = area;
        if area.height <= 1 {
        } else if area.height <= 4 {
            let area = Rect::new(area.x, area.y+area.height/2, area.width, 3);
            bottom_area = increment(1, Axis::Y, area);
        } else {
            let area = Rect::new(area.x, area.y+area.height-3, area.width, 3);
            bottom_area = increment(1, Axis::Y, area);
        }

        let mut temp = data.console_hist.borrow().clone();
        let mut temp = temp.concat();
        let mut hist = data.console_hist.borrow().clone();
        let mut hist = hist.concat();
        let mut temp_clone = temp.clone();
        let mut lines = 0;
        while temp_clone != "" {
            if temp_clone.pop() == Some('\n') {
                lines += 1;
            }
        }
        while lines > bottom_area.y-area.y {
            if lines > 100 {
                hist.remove(0);
            }

            if temp.remove(0) == '\n' {
                lines -=1;
            }

        }

        let mut hist_2 = Vec::new();
        hist_2.push(hist);

        data.console_hist.replace(hist_2);

        let text_history = [TuiText::styled(temp, Style::default().fg(c!(ConsoleOut)))];
        let mut para = Paragraph::new(text_history.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf); // the idea of this is to write the output before the ">", but I'm not sure this accomplishes that...

        let text = [TuiText::styled(">", Style::default().fg(c!(Title)))];

        para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        if area.height <= 1 {
            para.draw(area, buf);
        } else if area.height <= 4 {
            let area = Rect::new(area.x, area.y+area.height/2, area.width, 3);
            para.draw(area, buf);
        } else {
            let area = Rect::new(area.x, area.y+area.height-3, area.width, 3);
            para.draw(area, buf);
        }

        if bottom_area.height >= 2 {
            let text = [TuiText::styled(data.input_string.borrow_mut().clone(), Style::default().fg(c!(ConsoleOut)))];
            para = Paragraph::new(text.iter())
                .style(Style::default().fg(Colour::White).bg(Colour::Reset))
                .alignment(Alignment::Left)
                .wrap(true);
            para.draw(bottom_area,buf);
        }

    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();


        match event {
            Focus(FocusEvent::GotFocus) => true,
            Focus(FocusEvent::LostFocus) => true,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,
            Mouse(MouseEvent::Down(_, _, _, _)) => true,


            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {

                match data.input {
                    Some(input) => {
                        let fallible = input.put_char(c);  // put characters into input sink

                        match fallible {
                            Some(some) => {
                                let x = format!("{}", c);
                                data.input_string.borrow_mut().push_str(&x);
                                true
                            },

                            None => {
                                false
                            }

                        }
                    },
                    None => {
                        false

                    }
                }

            },
            Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                data.input_string.replace(String::from(""));

                true
            },
             _ => false,
        }
    }
}
