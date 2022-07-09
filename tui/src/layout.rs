//! Module defining the layout of the widgets used by the TUI.

use crate::tui::widget::{Widgets, Widget};
use crate::tui::widget::util::ConditionalSendBound;
use crate::widgets::*;
use crate::colours::c;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::layout::{Layout, Direction, Constraint};
use tui::widgets::{Block, Borders};
use tui::terminal::Terminal;
use tui::style::{Style, Color as Colour};

// Returns the root widget for our layout.
//
// This is currently 'static' (i.e. doesn't change based on the inputs given)
// but that could change in the future.
// TODO: potentially parameterize this from with user configurable options!
pub fn layout<'a, 'int: 'a, C, I, O, B: 'a>(
    name: Option<&'a str>,
    extra_tabs: Vec<(Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>, String)>,
) -> impl Widget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    Terminal<B>: ConditionalSendBound,
{
    let mut root = RootWidget::new(layout_tabs(name, extra_tabs))
        .add(Modeline::new(LoadButton::new()));

    root
}


pub fn layout_tabs<'a, 'int: 'a, C, I, O, B: 'a>(
    name: Option<&'a str>,
    extra_tabs: Vec<(Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>, String)>,
) -> Tabs<'a, 'int, C, I, O, B, impl Fn() -> TabsBar<'a, String>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    Terminal<B>: ConditionalSendBound,
{
    let horz = Layout::default().direction(Direction::Horizontal);
    let vert = Layout::default().direction(Direction::Vertical);
    let b = Block::default()
        .title_style(Style::default().fg(c!(Title)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Colour::White))
        .style(Style::default().bg(Colour::Reset));
    let empty = Empty::default();

    let mut root = Widgets::new(horz.clone());

    let mem = Mem::default();
    let regs = Regs::default();
    let console = Console::default();
    let gpio = Gpio::default();
    let pwm =  Pwm::default();
    let timers = Timers::default();
    let clock = Clock::default();
    let adc = Adc::default();
    let console_peripherals = ConsolePeripherals::default();

    let gpio2 = Gpio::default();
    let pwm2 =  Pwm::default();
    let timers2 = Timers::default();
    let adc2 = Adc::default();


    let mut peripherals = Widgets::new(vert.clone());
    let mut io = Widgets::new(vert.clone());

    let mut left = Widgets::new(vert.clone());
    let _ = left.add_widget(Constraint::Percentage(65), mem, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Memory")))
        .add_widget(Constraint::Percentage(35), regs, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Registers + PC+ PSR").title_style(Style::default().fg(c!(Title)))));

    let mut right = Widgets::new(vert.clone());

    let _ = io.add_widget(Constraint::Percentage(35), gpio.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("GPIO").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(20), adc.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("ADC").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(13), timers.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("Timers").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(14), pwm.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("PWM").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(13), clock.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("Clock").title_style(Style::default().fg(c!(Title)))));


    let _ = peripherals.add_widget(Constraint::Percentage(35), gpio2.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("GPIO").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(20), adc2.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("ADC").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(10), timers2.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("Timers").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(10), pwm2.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("PWM").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(10), clock.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(c!(Border))).title("Clock").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(15), console_peripherals, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Peripheral Console")));

    let _ = right.add_widget(Constraint::Percentage(60), console, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Console")))
        .add_widget(Constraint::Percentage(40), io, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("IO")));

    let _ = root.add_widget(Constraint::Percentage(48), left, None)
        .add_widget(Constraint::Percentage(52), right, None);

    let mut help = Widgets::new(horz.clone());
    let mut middle = Widgets::new(vert.clone());
    let help_text = Help::default();

    let _ = middle.add_widget(Constraint::Percentage(20), empty.focusable(false), None)
        .add_widget(Constraint::Percentage(60), help_text, Some(b.clone().border_style(Style::default().fg(Colour::Yellow)).title("Help")))
        .add_widget(Constraint::Percentage(20), empty.focusable(false), None);

    let _ = help.add_widget(Constraint::Percentage(20), empty.focusable(false), None)
        .add_widget(Constraint::Percentage(60), middle, None)
        .add_widget(Constraint::Percentage(20), empty.focusable(false), None);

    let mut memory = Widgets::new(vert.clone());

    let mem = Mem::default();
    let regs = Regs::default();
    let _ = memory.add_widget(Constraint::Percentage(80), mem, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Memory")))
        .add_widget(Constraint::Percentage(20), regs, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Registers + PC+ PSR").title_style(Style::default().fg(c!(Title)))));


    let mut big_console_tab  = Widgets::new(vert.clone());
    let _ = big_console_tab.add_widget(Constraint::Percentage(100), Console::default(), Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Console")));

    let mut log = Widgets::new(horz.clone());

    let log_window = Text::new(|t| t.log.as_ref());
    let _ = log.add_widget(Constraint::Percentage(100), log_window, Some(b.clone().border_style(Style::default().fg(Colour::Green)).title("Global Program Log")));

    let mut debug = Widgets::new(horz.clone());
    let mut top_right = Widgets::new(horz.clone());
    let _ = top_right.add_widget(Constraint::Percentage(15), BreakWindow::default(), Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Breakpoints")))
        .add_widget(Constraint::Percentage(45), WatchWindow::default(), Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Watchpoints").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(40), StackWindow::default(), Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Call Stack").title_style(Style::default().fg(c!(Title)))));

    let mem = Mem::new_with_debug(true);
    let regs = Regs::new_with_debug(true);
    let mut left = Widgets::new(vert.clone());
    let _ = left.add_widget(Constraint::Percentage(80), mem, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Memory")))
        .add_widget(Constraint::Percentage(20), regs, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Registers + PC+ PSR").title_style(Style::default().fg(c!(Title)))));

    let console = Console::default();
    let mem_console = MemRegInterface::default();

    let mut right = Widgets::new(vert.clone());
    let _ = right.add_widget(Constraint::Percentage(35), top_right, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Debug Tools")))
        .add_widget(Constraint::Percentage(40), console, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Console").title_style(Style::default().fg(c!(Title)))))
        .add_widget(Constraint::Percentage(25), mem_console, Some(b.clone().border_style(Style::default().fg(c!(Border))).title("Memory Interface").title_style(Style::default().fg(c!(Title)))));


    let _ = debug.add_widget(Constraint::Percentage(50), left, None)
        .add_widget(Constraint::Percentage(50), right, None);

    use crate::strings::*;

    let mut tabs = Tabs::new(root, s!(RootTab))
        .add(peripherals, s!(PeripheralsTab))
        .add(memory, s!(MemTab))
        .add(big_console_tab, s!(ConsoleTab))
        .add(debug, s!(DebugTab))
        .add(help, s!(HelpTab))
        .add(log, s!(LogTab))
        .with_tabs_bar(move || {
            TabsBar::default()
                .block(Block::default().title(name.unwrap_or(s!(TabBarName))).borders(Borders::ALL).border_style(Style::default().fg(c!(Border))))
                .style(Style::default().fg(Colour::White))
                .highlight_style(Style::default().fg(Colour::LightCyan))
                // .divider(tui::symbols::DOT)
        });

    for (w, t) in extra_tabs {
        tabs = tabs.add_dyn(w, t);
    }

    if crate::debug::in_debug_mode() {
        let events = Text::new(|t| t.debug_log.as_ref().unwrap());

        let mut event_log = Widgets::new(vert.clone());
        let _ = event_log
            .add_widget(Constraint::Percentage(100), events, Some(b.clone().border_style(Style::default().fg(Colour::Green)).title("Event Log")));

        tabs = tabs
            .add(event_log, s!(EventLogTab));
    }


    tabs
}
