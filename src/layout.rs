//! Module defining the layout of the widgets used by the TUI.

use crate::tui::widget::{Widgets, Widget};
use crate::widgets::*;

use lc3_application_support::io_peripherals::InputSink;
use lc3_application_support::io_peripherals::OutputSource;
use lc3_traits::control::Control;

use tui::backend::Backend;
use tui::layout::{Layout, Direction, Constraint};
use tui::widgets::{Block, Borders};
use tui::terminal::Terminal;
use tui::style::{Style, Color};

// Returns the root widget for our layout.
//
// This is currently 'static' (i.e. doesn't change based on the inputs given)
// but that could change in the future.
// TODO: potentially parameterize this from with user configurable options!
pub fn layout<'a, 'int: 'a, C, I, O, B: 'a>() -> impl Widget<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    Terminal<B>: Send,
{
    layout_tabs()
}

/*fn make_footer(footer: &mut Widget) {
    let b = Block::default()
        .title_style(Style::default().fg(Color::Red))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Reset));
    let empty = Empty::default();

    let mut buttons = Widgets::new(horz.clone());
    //let run = Button::new(String::from("Run"), Color::Green, |t| &t.sim.run_until_event());
    //let pause = Button::new(String::from("Pause"), Color::Red, |t| &t.sim.pause());
    //let step = Button::new(String::from("Step"), Color::Yellow, |t| &t.sim.step());
    let run = empty.focusable(true);
    let pause = empty.focusable(true);
    let step = empty.focusable(true);

    let _ = buttons.add_widget(Constraint::Percentage(25), run, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Green))))
        .add_widget(Constraint::Percentage(25), pause, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Red))))
        .add_widget(Constraint::Percentage(25), step, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))))
        .add_widget(Constraint::Percentage(25), LoadButton::new(), Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::White))));

    let _ = footer.add_widget(Constraint::Percentage(50), Footer::default(), Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Footer")))
        .add_widget(Constraint::Percentage(50), buttons, None);
}*/

pub fn layout_tabs<'a, 'int: 'a, C, I, O, B: 'a>() -> Tabs<'a, 'int, C, I, O, B, impl Fn() -> TabsBar<'a, String>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    Terminal<B>: Send,
{
    let horz = Layout::default().direction(Direction::Horizontal);
    let vert = Layout::default().direction(Direction::Vertical);
    let b = Block::default()
        .title_style(Style::default().fg(Color::Red))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Reset));
    let empty = Empty::default();

    let mut root = Widgets::new(vert.clone());
    let mut root_main = Widgets::new(horz.clone());

    
    
    let mem = Mem::default();
    let regs = Regs::default();
    let console = Console::default();
    let gpio = Gpio::default();
    let pwm =  Pwm::default();
    let timers = Timers::default();
    let clock = Clock::default();
    let adc = Adc::default();
    let console_peripherals = Console_peripherals::default();


   


    let gpio_toggle = Gpio_toggle::default();
    let pwm_toggle =  Pwm_toggle::default();
    let timers_toggle = Timers_toggle::default();
    let adc_toggle = Adc_toggle::default();


    let mut peripherals = Widgets::new(vert.clone());
    let mut io = Widgets::new(vert.clone());
    // let mut top_left = Widgets::new(vert.clone());
    
    let mut left = Widgets::new(vert.clone());
    let _ = left.add_widget(Constraint::Percentage(80), mem, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Memory")))
        .add_widget(Constraint::Percentage(20), regs, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Registers + PC+ PSR").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))));

    let mut right = Widgets::new(vert.clone());

    let _ = io.add_widget(Constraint::Percentage(35), gpio.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("GPIO").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(20), adc.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("ADC").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(13), timers.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("Timers").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(14), pwm.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("PWM").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(13), clock.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("Clock").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))));


    let _ = peripherals.add_widget(Constraint::Percentage(35), gpio_toggle.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("GPIO").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(20), adc_toggle.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("ADC").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(10), timers_toggle.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("Timers").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(10), pwm_toggle.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("PWM").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(10), clock.focusable(false), Some(b.clone().borders(Borders::ALL & (!Borders::BOTTOM)).border_style(Style::default().fg(Color::Blue)).title("Clock").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))))
        .add_widget(Constraint::Percentage(15), console_peripherals, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Peripheral Console")));

    let _ = right.add_widget(Constraint::Percentage(60), console, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Console")))
        .add_widget(Constraint::Percentage(40), io, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("IO")));

    let _ = root_main.add_widget(Constraint::Percentage(50), left, None)
        .add_widget(Constraint::Percentage(50), right, None);

    
    let mut footer = Widgets::new(horz.clone());
    let mut buttons = Widgets::new(horz.clone());
    let run = empty.focusable(true);
    let pause = empty.focusable(true);
    let step = empty.focusable(true);

    let _ = buttons.add_widget(Constraint::Percentage(25), run, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Green))))
        .add_widget(Constraint::Percentage(25), pause, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Red))))
        .add_widget(Constraint::Percentage(25), step, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))))
        .add_widget(Constraint::Percentage(25), LoadButton::new(), Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::White))));

    let _ = footer.add_widget(Constraint::Percentage(50), Footer::default(), Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Footer")))
        .add_widget(Constraint::Percentage(50), buttons, None);

    let _ = root.add_widget(Constraint::Percentage(85), root_main, None)
        .add_widget(Constraint::Percentage(15), footer, None);

    let mut help = Widgets::new(horz.clone());
    let mut middle = Widgets::new(vert.clone());
    let help_text = Help::default();

    let _ = middle.add_widget(Constraint::Percentage(20), empty.focusable(false), None)
        .add_widget(Constraint::Percentage(60), help_text, Some(b.clone().border_style(Style::default().fg(Color::Yellow)).title("Help")))
        .add_widget(Constraint::Percentage(20), empty.focusable(false), None);

    let _ = help.add_widget(Constraint::Percentage(20), empty.focusable(false), None)
        .add_widget(Constraint::Percentage(60), middle, None)
        .add_widget(Constraint::Percentage(20), empty.focusable(false), None);

    let mut memory = Widgets::new(vert.clone());
    let mut memory_main = Widgets::new(horz.clone());

    let mem = Mem::default();
    let regs = Regs::default();
    let mut left = Widgets::new(vert.clone());
    let _ = left.add_widget(Constraint::Percentage(80), mem, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Memory")))
        .add_widget(Constraint::Percentage(20), regs, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Registers + PC+ PSR").title_style(Style::default().fg(Color::Rgb(0xFF, 0x97, 0x40)))));

    let _ = memory_main.add_widget(Constraint::Percentage(50), left, None)
        .add_widget(Constraint::Percentage(50), empty.focusable(false), None);

    let mut footer = Widgets::new(horz.clone());
    let mut buttons = Widgets::new(horz.clone());
    let run = empty.focusable(true);
    let pause = empty.focusable(true);
    let step = empty.focusable(true);

    let _ = buttons.add_widget(Constraint::Percentage(25), run, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Green))))
        .add_widget(Constraint::Percentage(25), pause, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Red))))
        .add_widget(Constraint::Percentage(25), step, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))))
        .add_widget(Constraint::Percentage(25), LoadButton::new(), Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::White))));

    let _ = footer.add_widget(Constraint::Percentage(50), Footer::default(), Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Footer")))
        .add_widget(Constraint::Percentage(50), buttons, None);

    let _ = memory.add_widget(Constraint::Percentage(85), memory_main, None)
       .add_widget(Constraint::Percentage(15), footer, None);

    let mut big_console_tab  = Widgets::new(vert.clone());

    let mut footer = Widgets::new(horz.clone());
    let mut buttons = Widgets::new(horz.clone());
    let run = empty.focusable(true);
    let pause = empty.focusable(true);
    let step = empty.focusable(true);

    let _ = buttons.add_widget(Constraint::Percentage(25), run, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Green))))
        .add_widget(Constraint::Percentage(25), pause, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Red))))
        .add_widget(Constraint::Percentage(25), step, Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))))
        .add_widget(Constraint::Percentage(25), LoadButton::new(), Some(b.clone().borders(Borders::ALL).border_style(Style::default().fg(Color::White))));

    let _ = footer.add_widget(Constraint::Percentage(50), Footer::default(), Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Footer")))
        .add_widget(Constraint::Percentage(50), buttons, None);
    //make_footer(footer);

    let console = Console::default();
    let _ = big_console_tab.add_widget(Constraint::Percentage(85), console, Some(b.clone().border_style(Style::default().fg(Color::Blue)).title("Console")))
        .add_widget(Constraint::Percentage(15), footer, None);

    let mut log = Widgets::new(horz.clone());

    let log_window = Text::new(|t| t.log.as_ref());
    let _ = log.add_widget(Constraint::Percentage(100), log_window, Some(b.clone().border_style(Style::default().fg(Color::Green)).title("Global Program Log")));

    let mut tabs = Tabs::new(root, "🌴 Root")
        .add(peripherals, "🎛️  Peripherals")
        .add(memory, "😀 Mem")
        .add(big_console_tab, "😲 Console")
        .add(empty, "😉 Baz")
        .add(help, "ℹ️  Help")
        .add(log, "📜 Log")
        .with_tabs_bar(|| {
            TabsBar::default()
                .block(Block::default().title("Tabs").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::LightCyan))
                // .divider(tui::symbols::DOT)
        });

    
    if crate::debug::in_debug_mode() {
        let events = Text::new(|t| t.debug_log.as_ref().unwrap());

        let mut debug = Widgets::new(vert.clone());
        let _ = debug
            .add_widget(Constraint::Percentage(100), events, Some(b.clone().border_style(Style::default().fg(Color::Green)).title("Event Log")));

        tabs = tabs
            .add(debug, "🐛 Debug Info");
    }


    tabs
}
