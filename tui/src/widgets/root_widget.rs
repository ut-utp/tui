//! A tabs widget that owns its tabs and allows users to switch between the tabs.
//!
//! TODO!

// Note: the key bindings thing is not nesting friendly; that is only the top
// most Tabs widget will actually receive and handle them. This is fine.

// For now, this requires that the tabs be _static_ (i.e. can't add or remove
// tabs after creating the item) but this restriction can be lifted later if
// there's a use case for it.

use super::widget_impl_support::*;

pub use tui::widgets::Tabs as TabsBar;

#[allow(explicit_outlives_requirements)]
pub struct RootWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    /// The actual tabs.
    components: Vec<Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>>,
    /// Footer or Main
    cur_focus: usize,
    // Cutoff Threshold
    footer_cutoff: u16,
}

impl<'a, 'int, C, I, O, B> RootWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    // This style of constructor is there to ensure that there's at least one
    // tab.
    pub fn new<W: Widget<'a, 'int, C, I, O, B> + 'a>(main: W) -> Self {
        Self {
            components: vec![Box::new(main)],
            cur_focus: 0,
            footer_cutoff: 0,
        }
    }

    pub fn add<W: Widget<'a, 'int, C, I, O, B> + 'a>(mut self, footer: W) -> Self {
        self.components.push(Box::new(footer));
        self
    }

    // TODO: possibly make this configurable
    fn area_split(&self, area: Rect) -> (Rect, Rect) {
        if let [bar, rest] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(4)].as_ref())
            .split(area)
            [..] {
            return (bar, rest)
        } else {
            unreachable!()
        }
    }

    fn propagate(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        self.components[self.cur_focus].update(event, data, terminal)
    }

    fn drop_extra_focus(&mut self, section: usize, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) {
        let event = WidgetEvent::Focus(FocusEvent::LostFocus);
        self.components[section].update(event, data, terminal);
    }

    fn give_focus(&mut self, section: usize, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) {
        let event = WidgetEvent::Focus(FocusEvent::GotFocus);
        self.components[section].update(event, data, terminal);
    }

    fn propagate_to_main(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        let out = self.components[0].update(event, data, terminal);
        if self.cur_focus == 1 {
            self.drop_extra_focus(0, data, terminal);
        }
        out
    }

    fn propagate_to_footer(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        let out = self.components[1].update(event, data, terminal);
        if self.cur_focus == 0 {
            self.drop_extra_focus(1, data, terminal);
        }
        out
    }

    fn propagate_to_all(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        self.components.iter_mut().fold(false, |b, w| b | w.update(event, data, terminal))
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for RootWidget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        let (main, footer) = self.area_split(area);
        self.footer_cutoff = footer.y;

        Widget::draw(&mut *self.components[0], data, main, buf);
        Widget::draw(&mut *self.components[1], data, footer, buf);
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Key(e) => match e {
                KeyEvent { code: KeyCode::Char(n @ '1'..='9'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char(n @ '1'..='9'), modifiers: KeyModifiers::ALT } => {
                    // Switch to 0 indexing:
                    self.drop_extra_focus(0, data, terminal);
                    self.propagate_to_main(event, data, terminal)
                },
                KeyEvent { code: KeyCode::F(n), modifiers: EMPTY } => {
                    // Switch to 0 indexing:
                    self.drop_extra_focus(0, data, terminal);
                    self.propagate_to_main(event, data, terminal)
                }

                // Crossterm seems to drop `ctrl` so we'll compromise with this for now (TODO):
                KeyEvent { code: KeyCode::BackTab, modifiers: EMPTY } |
                KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::CONTROL } => {
                    self.drop_extra_focus(0, data, terminal);
                    self.propagate_to_main(event, data, terminal)
                }

                KeyEvent { code: KeyCode::Char('i'), modifiers: KeyModifiers::ALT } |
                KeyEvent { code: KeyCode::Char('i'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char('o'), modifiers: KeyModifiers::ALT } |
                KeyEvent { code: KeyCode::Char('o'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::ALT } |
                KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char('p'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char('r'), modifiers: KeyModifiers::CONTROL } => {
                    self.propagate_to_footer(event, data, terminal)
                }
                KeyEvent { code: KeyCode::Char('t'), modifiers: KeyModifiers::CONTROL } => {
                    self.drop_extra_focus(0, data, terminal);
                    self.cur_focus = 1;
                    self.give_focus(1, data, terminal);
                    self.propagate_to_footer(event, data, terminal)
                }
                KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL } => {
                    self.propagate_to_footer(event, data, terminal)
                }

                KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::CONTROL } => {
                    if self.cur_focus == 0 {
                        let edge = self.propagate(event, data, terminal);
                        if edge == false {
                            self.drop_extra_focus(0, data, terminal);
                            self.cur_focus = 1;
                            self.give_focus(1, data, terminal);
                        }
                        true
                    } else {
                        false
                    }
                }

                KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::CONTROL } => {
                    if self.cur_focus == 1 {
                        self.drop_extra_focus(1, data, terminal);
                        self.cur_focus = 0;
                        self.give_focus(0, data, terminal);
                        true
                    } else {
                        self.propagate(event, data, terminal)
                    }
                }

                // Crossterm seems to drop `ctrl` so we'll compromise with this for now (TODO):
                KeyEvent { code: KeyCode::Tab, modifiers: EMPTY } |
                KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::CONTROL } => {
                    self.drop_extra_focus(0, data, terminal);
                    self.propagate_to_main(event, data, terminal)
                }

                _ => self.propagate(event, data, terminal),
            }

            Mouse(e) => {
                use MouseEvent::*;
                match e {
                    Down(_, col, row, _) | Up(_, col, row, _) => {
                        if row >= self.footer_cutoff {
                            self.drop_extra_focus(0, data, terminal);
                            self.cur_focus = 1;
                            self.propagate(event, data, terminal)
                        } else {
                            if self.propagate_to_main(event, data, terminal) {
                                self.cur_focus = 0;
                                self.drop_extra_focus(1, data, terminal);
                                true
                            } else {
                                false
                            }
                        }
                    },
                    _ => self.propagate(event, data, terminal),
                }
            }

            // TODO: pick a style for the tabs!
            // TODO: handle mouse events! (blocked on the above)

            // Resize all the tabs!
            Resize(_, _) => self.propagate_to_all(event, data, terminal),

            Update => self.propagate_to_all(event, data, terminal),

            _ => self.propagate(event, data, terminal)
        }
    }
}
