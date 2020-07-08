//! TODO!

use super::widget_impl_support::*;

use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Console;

// TODO: scrolling!
// TODO: copy/paste!

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
                .render(area, buf);
        }

        // Append any new output we have:
        if let Some(s) = data.output.and_then(|source| source.get_chars()){
            data.console_hist.borrow_mut().push_string(s);
        }

        // Figure out the areas:
        // We want to reserve the bottom line for input and we want to print a dividing
        // line above it.
        let [output, input] = if let [o, i] = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(0)
            .vertical_margin(0)
            .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
            .split(area)
            [..] {
            [o, i]
        } else {
            unreachable!()
        };

        // TODO: this deals with lines that wrap in pretty much the worst way.
        // We always print the last `n` lines where `n` is the height of the
        // output area but the problem is that we count lines by newlines so
        // any lines that get wrapped cause another line to be "pushed out" of
        // the output area.
        //
        // Since we can't scroll, this ends up meaning you just can't see the
        // lines that have been pushed out.
        //
        // Because of this I'm going to disable output wrapping for now, but
        // this is not good!

        // Finally, do the drawing:
        // Paragraph::new(data.console_hist.borrow().get_lines(..))
        Paragraph::new(data.console_hist.borrow().get_last_n_lines(output.height as usize))
            .style(Style::default().fg(c!(ConsoleOut)).bg(Colour::Reset))
            .alignment(Alignment::Left)
            // .wrap(true)
            // .raw(true)
            .render(output, buf);

        Paragraph::new([
                TuiText::styled("─".repeat(input.width as usize), Style::default().fg(c!(Border))),
                TuiText::styled("\n> ", Style::default().fg(c!(ConsolePrompt))),
                TuiText::styled(data.console_input_string.borrow().deref(), Style::default().fg(c!(ConsoleIn))),
            ].iter())
            .style(Style::default().bg(Colour::Reset))
            // .wrap(true)
            .render(input, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();


        match event {
            Focus(FocusEvent::GotFocus)
            | Focus(FocusEvent::LostFocus)
            | Mouse(MouseEvent::Up(_, _, _, _))
            | Mouse(MouseEvent::Down(_, _, _, _)) => true,

            Key(KeyEvent { code: KeyCode::Char(c), modifiers: EMPTY }) => {
                data.input
                    .and_then(|sink| sink.put_char(c))
                    // If that succeeded, add the char to the input line:
                    .map(|()| data.console_input_string.get_mut().push(c))
                    .is_some()
            },

            Key(KeyEvent { code: KeyCode::Enter, modifiers: EMPTY }) => {
                data.console_input_string.get_mut().clear();
                true
            },

             _ => false,
        }
    }
}
