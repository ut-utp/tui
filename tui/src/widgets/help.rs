//! TODO!

use super::widget_impl_support::*;

use super::super::Tui;

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

impl TuiWidget for Help
{
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Do not call this!");
    }
}


impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Help
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
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
                Ctrl: + t twice to reset.\n", Style::default().fg(c!(Help))),
        ];

        // TODO: allow parameterization of this in the usual way.
        let mut para = Paragraph::new(text.iter())
            .style(Style::default().fg(Colour::White).bg(Colour::Reset))
            .alignment(Alignment::Left)
            .wrap(true);

        para.draw(area, buf)
    }

    fn update(&mut self, event: WidgetEvent, _data: &mut TuiData<'a, 'int, C, I, O>, _terminal: &mut Terminal<B>) -> bool {
        match event {
            WidgetEvent::Mouse(_) | WidgetEvent::Focus(FocusEvent::GotFocus) => self.focusable,
            _ => false,
        }
    }
}
