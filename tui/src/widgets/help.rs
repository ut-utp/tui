//! TODO!

use super::widget_impl_support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Help
{
    pub focusable: bool
}

impl Default for Help {
    fn default() -> Self {
        Self {
            focusable: true,
        }
    }
}

impl<Wt: WidgetTypes> Widget<Wt> for Help {
    fn draw(&mut self, data: &Data<Wt>, area: Rect, buf: &mut Buffer) {
        let text = [
            TuiText::styled("To control the TUI, you can click on any of the interactable interfaces.\n
                Most will respond to mouse events, some will respond to sroll.\n
                You can also use keyboard events to navigate.\n
                Tab and Ctrl tab swaps back and forth between tabs.\n
                FN + a number jumps to the tab of that number directly.\n
                Ctrl + a diretional arrow also moves between widgets.\n
                A lot of widgets have their own keybinds. See the full documentation for assistance.\n
                Finally, there are universal keybinds:\n
                Ctrl: + l to load, + r to run, + p to pause, + s to step\n
                Ctrl: + t twice to reset.\n
                Ctrl + u for Step Over, Ctrl + i for Step In (== Step), Ctrl + o for Step Out (these binds also work with Alt — i.e. Alt + u — for machines with issues with Ctrl)", Style::default().fg(c!(Help))),
        ];

        // TODO: allow parameterization of this in the usual way.
        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.render(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut Data<Wt>, _terminal: &mut Terminal<Wt::Backend>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
