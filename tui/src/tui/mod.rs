//! TODO

use lc3_application_support::init::{BlackBox, Init};
use lc3_application_support::shim_support::Shims;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{EventFuture, SyncEventFutureSharedState};
use lc3_traits::control::control::{Control, Event};

use lc3_isa::Addr;

use std::sync::Mutex;
use std::time::Duration;
use std::collections::HashMap;
use std::string::ToString;
use std::cell::RefCell;


use tui::widgets::Text as TuiText;
use tui::style::{Style, Color};

pub mod ansi;

pub mod run;
pub mod events;
pub mod widget;

pub mod program_source;


pub use program_source::ProgramSource;
pub type Res<T> = Result<T, failure::Error>;

#[allow(explicit_outlives_requirements)]
pub struct TuiData<'a, 'int, C, I = SourceShim, O = Mutex<Vec<u8>>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub sim: &'a mut C,
    pub input: Option<&'a I>,
    pub input_string: RefCell<String>,
    pub output: Option<&'a O>,
    pub shims: Option<Shims<'int>>,

    pub(in crate) program_source: Option<ProgramSource>,
    /// Determines whether we will tell the assembler to build using the OS
    /// *and* whether we skip past the OS on loads and resets.
    pub(in crate) use_os: bool,

    pub(in crate) reset_flag: u8,
    pub(in crate) load_flag: u8,
    pub(in crate) jump: (u8, Addr),
    pub(in crate) mem_reg_inter: (u8, Addr),
    pub(in crate) debug_log: Option<Vec<TuiText<'a>>>,
    pub(in crate) console_hist: RefCell<Vec<String>>,
    // pub(in crate) console_hist: AnsiTextContainer<'a>, // TODO!
    pub(in crate) log: Vec<TuiText<'a>>,
    pub(in crate) bp: HashMap<Addr, usize>,
    pub(in crate) wp: HashMap<Addr, usize>,

    pub(in crate) flush_all_events: Option<Flush>,
    /// Is `Some(_)` when an `Event` has _just_ occurred.
    pub(in crate) current_event: Option<Event>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(in super) enum Flush {
    Requested(u8),
    Acknowledged(u8),
}

#[allow(explicit_outlives_requirements)]
impl<'a, 'int, C, I, O> TuiData<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub fn log<L: ToString>(&mut self, line: L, colour: Color) {
        self.log.push(TuiText::styled(line.to_string(), Style::default().fg(colour)))
    }

    pub fn log_raw<L: ToString>(&mut self, line: L) {
        self.log.push(TuiText::raw(line.to_string()))
    }

    pub(in crate) fn flush_events(&mut self) {
        self.flush_all_events = Some(match self.flush_all_events {
            Some(Flush::Requested(i)) => Flush::Requested(i + 1),
            Some(Flush::Acknowledged(i)) => Flush::Acknowledged(i + 1),
            None => Flush::Requested(0),
        })
    }

    pub fn get_current_event(&self) -> Option<Event> {
        self.current_event
    }
}


#[allow(explicit_outlives_requirements)]
pub struct Tui<'a, 'int, C, I = SourceShim, O = Mutex<Vec<u8>>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub data: TuiData<'a, 'int, C, I, O>,

    pub(in crate::tui) update_period: Duration,
    // pub(in crate::tui)

}

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
    pub fn new(sim: &'a mut C) -> Self {
        Self {
            data: TuiData {
                sim,
                input: None,
                input_string: RefCell::new(String::from("")),
                output: None,
                shims: None,

                program_source: None,
                use_os: true,

                reset_flag: 0,
                load_flag: 0,
                jump: (0,0x200),
                mem_reg_inter: (0, 0),

                debug_log: if crate::debug::in_debug_mode() {
                    Some(Vec::with_capacity(32 * 1024 * 1024))
                } else {
                    None
                },

                // TODO(rrbutani)!
                // console_hist: RefCell::new(Vec::<TuiText<'a>>::with_capacity(500)),

                console_hist: RefCell::new(Vec::<String>::with_capacity(500)),

                log: Vec::with_capacity(16 * 1024 * 1024),

                bp: HashMap::new(),
                wp: HashMap::new(),

                flush_all_events: None,
                current_event: None,
            },

            update_period: Duration::from_millis(250),
        }
    }

    // The concern with offering this interface instead of the below is that
    // it's more likely users could attach a set of shim peripherals that aren't
    // actually being used with the simulator.
    //
    // But, in reality, I think this is equally likely either way and this is a
    // nicer API.
    pub fn attach_shims(mut self, shims: Shims<'int>) -> Self {
        self.data.shims = Some(shims);
        self
    }

    pub fn attach_input_output(mut self, input: &'a I, output: &'a O) -> Self {
        self.data.input = Some(input);
        self.data.output = Some(output);
        self
    }
}

type DynControl<'a> = (dyn Control<EventFuture = EventFuture<'static, SyncEventFutureSharedState>> + 'a);
type DynInputSink<'a> = (dyn InputSink + 'a);
type DynOutputSource<'a> = (dyn OutputSource + 'a);

pub type DynTui<'a, 'int> = Tui<'a, 'int, DynControl<'a>, DynInputSink<'a>, DynOutputSource<'a>>;

impl<'a, 'int> DynTui<'a, 'int> {
    pub fn new_boxed<C>(sim: &'a mut C) -> Self
    where
        C: Control<EventFuture = EventFuture<'static, SyncEventFutureSharedState>>,
    {
        Self::new(sim)
    }

    pub fn attach_input_output_boxed<I, O>(self, input: &'a I, output: &'a O) -> Self
    where
        I: InputSink + 'a,
        O: OutputSource + 'a,
    {
        self.attach_input_output(input, output)
    }
}

impl<'a> DynTui<'a, 'static> {
    pub fn new_boxed_from_init<I: Init<'a>>(b: &'a mut BlackBox) -> Self
    where
        <I as Init<'a>>::ControlImpl: Control<EventFuture = EventFuture<'static, SyncEventFutureSharedState>> + 'a,
        <I as Init<'a>>::ControlImpl: Sized,
        <I as Init<'a>>::Input: Sized,
        <I as Init<'a>>::Output: Sized,
    {
        Self::new_boxed_from_init_with_config_inner::<I>(b, None)
    }

    pub fn new_boxed_from_init_with_config<I: Init<'a>>(b: &'a mut BlackBox, config: I::Config) -> Self
    where
        <I as Init<'a>>::ControlImpl: Control<EventFuture = EventFuture<'static, SyncEventFutureSharedState>> + 'a,
        <I as Init<'a>>::ControlImpl: Sized,
        <I as Init<'a>>::Input: Sized,
        <I as Init<'a>>::Output: Sized,
    {
        Self::new_boxed_from_init_with_config_inner::<I>(b, Some(config))
    }

    fn new_boxed_from_init_with_config_inner<I: Init<'a>>(b: &'a mut BlackBox, config: Option<I::Config>) -> Self
    where
        <I as Init<'a>>::ControlImpl: Control<EventFuture = EventFuture<'static, SyncEventFutureSharedState>> + 'a,
        <I as Init<'a>>::ControlImpl: Sized,
        <I as Init<'a>>::Input: Sized,
        <I as Init<'a>>::Output: Sized,
    {
        let (sim, shims, input, output) = if let Some(config) = config {
            I::init_with_config(b, config)
        } else {
            I::init(b)
        };

        let mut tui = Self::new_boxed::<<I as Init<'a>>::ControlImpl>(sim);

        if let (Some(inp), Some(out)) = (input, output) {
            tui = tui.attach_input_output_boxed(inp, out)
        }

        if let Some(shims) = shims {
            tui = tui.attach_shims(shims)
        }

        tui
    }
}

impl<'a, 'int, C, I, O> Tui<'a, 'int, C, I, O>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a
{
    pub fn set_program_source(&mut self, src: ProgramSource) -> &mut Self {
        self.data.program_source = Some(src);
        self
    }

    pub fn set_use_os(&mut self, use_os: bool) -> &mut Self {
        self.data.use_os = use_os;
        self
    }

    pub fn set_update_period(&mut self, period: Duration) -> &mut Self {
        self.update_period = period;
        self
    }
}
