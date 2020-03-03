//! A tabs widget owns its tabs that allows users to switch between the tabs.
//!
//! TODO!

// Note: the key bindings thing is not nesting friendly; that is only the top
// most Tabs widget will actually receive and handle them. This is fine.

// For now, this requires that the tabs be _static_ (i.e. can't add or remove
// tabs after creating the item) but this restriction can be lifted later if
// there's a use case for it.

use super::widget_impl_support::*;

pub use tui::widgets::Tabs as TabsBar;

pub struct Tabs<'a, 'int, C, I, O, B, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    F: Fn() -> TabsBar<'a, String>,
{
    /// Function that produces a template `TabsBar` instance that gives us
    /// styling and dividers and such.
    tabs_bar_func: Option<F>,
    /// The titles of the tabs.
    titles: Vec<String>,
    /// The actual tabs.
    tabs: Vec<Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>>,
    /// Current tab.
    current_tab: usize,
}

impl<'a, 'int, C, I, O, B, F> Tabs<'a, 'int, C, I, O, B, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    F: Fn() -> TabsBar<'a, String>,
{
    // This style of constructor is there to ensure that there's at least one
    // tab.
    pub fn new<W: Widget<'a, 'int, C, I, O, B> + 'a, S: ToString>(first_tab: W, title: S) -> Self {
        Self {
            tabs_bar_func: None,
            titles: vec![title.to_string()],
            tabs: vec![Box::new(first_tab)],
            current_tab: 0,
        }
    }

    pub fn add<W: Widget<'a, 'int, C, I, O, B> + 'a, S: ToString>(mut self, tab: W, title: S) -> Self {
        self.tabs.push(Box::new(tab));
        self.titles.push(title.to_string());

        self
    }

    pub fn with_tabs_bar(mut self, func: F) -> Self {
        self.tabs_bar_func = Some(func);
        self
    }

    // TODO: possibly make this configurable
    fn area_split(&self, area: Rect) -> (Rect, Rect) {
        if let [bar, rest] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(area)
            [..] {
            return (bar, rest)
        } else {
            unreachable!()
        }
    }

    fn tabs_bar(&self) -> TabsBar<'_, String> {
        if let Some(ref f) = self.tabs_bar_func {
            f()
        } else {
            TabsBar::default()
        }
        .titles(self.titles.as_ref())
        .select(self.current_tab)
    }

    fn propagate(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        self.tabs[self.current_tab].update(event, data)
    }

    fn propagate_to_all(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        self.tabs.iter_mut().fold(false, |b, w| b | w.update(event, data))
    }

    fn switch_to_tab(&mut self, idx: usize) -> bool {
        if idx < self.tabs.len() {
            self.current_tab = idx;
            true
        } else {
            false
        }
    }
}

impl<'a, 'int, C, I, O, B, F> TuiWidget for Tabs<'a, 'int, C, I, O, B, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    F: Fn() -> TabsBar<'a, String>,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        // Shouldn't actually be called, but just in case:
        // (note: if there are Widgets within our tabs this will be A Problem)
        let (bar, rest) = self.area_split(area);

        self.tabs_bar().draw(bar, buf);
        TuiWidget::draw(&mut *self.tabs[self.current_tab], rest, buf);
    }
}

impl<'a, 'int, C, I, O, B, F> Widget<'a, 'int, C, I, O, B> for Tabs<'a, 'int, C, I, O, B, F>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    F: Fn() -> TabsBar<'a, String>,
{
    fn draw(&mut self, sim: &C, area: Rect, buf: &mut Buffer) {
        let (bar, rest) = self.area_split(area);

        self.tabs_bar().draw(bar, buf);
        Widget::draw(&mut *self.tabs[self.current_tab], sim, rest, buf)
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;
        const EMPTY: KeyModifiers = KeyModifiers::empty();

        match event {
            Key(e) => match e {
                KeyEvent { code: KeyCode::Char(n @ '1'..='9'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char(n @ '1'..='9'), modifiers: KeyModifiers::ALT } => {
                    // Switch to 0 indexing:
                    self.switch_to_tab(n as usize - '1' as usize)
                },
                KeyEvent { code: KeyCode::F(n), modifiers: EMPTY } => {
                    // Switch to 0 indexing:
                    self.switch_to_tab(n as usize - 1)
                }

                KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::CONTROL /*| KeyModifiers::SHIFT*/ } => {
                    self.switch_to_tab(self.current_tab.checked_sub(1).unwrap_or(self.tabs.len()))
                }
                KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::CONTROL } => {
                    // let idx = if let Some(idx) = self.current_tab.checked_add(1) {
                    //     if idx >= self.tabs.len() {
                    //         0
                    //     } else {
                    //         idx
                    //     }
                    // } else {
                    //     0
                    // };

                    // self.switch_to_tab(idx)

                    self.switch_to_tab(self.current_tab.checked_add(1).filter(|i| *i < self.tabs.len()).unwrap_or(0))
                }

                _ => self.propagate(event, data),
            }

            // TODO: handle mouse events!

            // Resize all the tabs!
            Resize(_, _) => self.propagate_to_all(event, data),

            _ => self.propagate(event, data)
        }
    }
}
