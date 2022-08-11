//! TODO

use lc3_application_support::init::{BlackBox, Init};
use lc3_application_support::shim_support::Shims;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{EventFuture, SyncEventFutureSharedState};
use lc3_traits::control::control::{Control, Event};

use lc3_isa::Addr;

use std::marker::PhantomData;
use std::sync::Mutex;
use std::time::Duration;
use std::collections::HashMap;
use std::string::ToString;
use std::cell::RefCell;


use tui::widgets::Text as TuiText;
use tui::style::{Style, Color};

pub mod ansi;
use ansi::AnsiTextContainer;

pub mod run;
pub mod events;
pub mod widget;

pub mod program_source;
pub use program_source::ProgramSource;

pub use anyhow::Result as Res;

// just so we don't have to lug around three separate types onto
// `Widget` and all of its implementors
pub trait TuiTypes {
    type Control: Control + ?Sized;
    type Input: InputSink + ?Sized;
    type Output: OutputSource + ?Sized;
}

#[derive(Debug)]
pub struct TuiTypeSet<C, I = SourceShim, O = Mutex<Vec<u8>>>(PhantomData<C>, PhantomData<I>, PhantomData<O>)
where
    C: ?Sized + Control,
    I: ?Sized + InputSink,
    O: ?Sized + OutputSource,
;

impl<C, I, O> TuiTypes for TuiTypeSet<C, I, O>
where
    C: ?Sized + Control,
    I: ?Sized + InputSink,
    O: ?Sized + OutputSource,
{
    type Control = C;
    type Input = I;
    type Output = O;
}


pub struct TuiData<'a, T: TuiTypes> {
    pub sim: &'a mut <T as TuiTypes>::Control,
    pub input: Option<&'a <T as TuiTypes>::Input>,
    pub output: Option<&'a <T as TuiTypes>::Output>,
    pub shims: Option<Shims>,

    pub(crate) program_source: Option<ProgramSource>,
    /// Determines whether we will tell the assembler to build using the OS
    /// *and* whether we skip past the OS on loads and resets.
    pub(crate) use_os: bool,

    pub(crate) reset_flag: u8,
    pub(crate) load_flag: u8,
    pub(crate) jump: (u8, Addr),
    pub(crate) mem_reg_inter: (u8, Addr),

    pub(crate) console_input_string: RefCell<String>, // TODO: Give this a better name, maybe.
    pub(crate) console_hist: RefCell<AnsiTextContainer<'a>>,

    pub(crate) debug_log: Option<Vec<TuiText<'a>>>,
    pub(crate) log: Vec<TuiText<'a>>,

    pub(crate) bp: HashMap<Addr, usize>,
    pub(crate) wp: HashMap<Addr, usize>,

    pub(crate) flush_all_events: Option<Flush>,
    /// Is `Some(_)` when an `Event` has _just_ occurred.
    pub(crate) current_event: Option<Event>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(in super) enum Flush {
    Requested(u8),
    Acknowledged(u8),
}

impl<'a, T: TuiTypes> TuiData<'a, T> {
    pub fn log<L: ToString>(&mut self, line: L, colour: Color) {
        self.log.push(TuiText::styled(line.to_string(), Style::default().fg(colour)))
    }

    pub fn debug_log<L: ToString>(&mut self, func: impl FnOnce(&Self) -> (L, Color)) {
        if self.debug_log.is_some() {
            let (line, colour) = func(self);
            self.debug_log.as_mut().unwrap().push(TuiText::styled(line.to_string(), Style::default().fg(colour)));
        }
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


pub struct Tui<'a, T: TuiTypes> {
    pub data: TuiData<'a, T>,

    pub(in crate::tui) update_period: Duration,
    // pub(in crate::tui)

}

impl<'a, T: TuiTypes> Tui<'a, T> {
    pub fn new(sim: &'a mut <T as TuiTypes>::Control) -> Self {
        Self {
            data: TuiData {
                sim,
                input: None,
                output: None,
                shims: None,

                program_source: None,
                use_os: true,

                reset_flag: 0,
                load_flag: 0,
                jump: (0,0x200),
                mem_reg_inter: (0, 0),

                console_input_string: RefCell::new(String::from("")),
                console_hist: RefCell::new(AnsiTextContainer::with_capacity(1024)),

                debug_log: if crate::debug::in_debug_mode() {
                    Some(Vec::with_capacity(32 * 1024 * 1024))
                } else {
                    None
                },

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
    pub fn attach_shims(mut self, shims: Shims) -> Self {
        self.data.shims = Some(shims);
        self
    }

    pub fn attach_input_output(
        mut self,
        input: &'a <T as TuiTypes>::Input,
        output: &'a <T as TuiTypes>::Output,
    ) -> Self {
        self.data.input = Some(input);
        self.data.output = Some(output);
        self
    }
}

type DynControl<'a> = (dyn Control<EventFuture = EventFuture<'static, SyncEventFutureSharedState>> + 'a);
type DynInputSink<'a> = (dyn InputSink + 'a);
type DynOutputSource<'a> = (dyn OutputSource + 'a);

pub type DynTui<'a> = Tui<'a, TuiTypeSet<DynControl<'a>, DynInputSink<'a>, DynOutputSource<'a>>>;

impl<'a> DynTui<'a> {
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

impl<'a> DynTui<'a> {
    pub fn new_from_init<I: Init<'a>>(
        b: &'a mut BlackBox,
        config: Option<I::Config>,
    ) -> Tui<'a, TuiTypeSet<I::ControlImpl, I::Input, I::Output>> {
        let (sim, shims, input, output) = if let Some(config) = config {
            I::init_with_config(b, config)
        } else {
            I::init(b)
        };

        let mut tui = Tui::new(sim);

        if let (Some(inp), Some(out)) = (input, output) {
            tui = tui.attach_input_output(inp, out);
        }

        if let Some(shims) = shims {
            tui = tui.attach_shims(shims)
        }

        tui
    }
}

impl<'a> DynTui<'a> {
    pub fn new_boxed_from_init<I: Init<'a>>(b: &'a mut BlackBox) -> Self
    where
        <I as Init<'a>>::ControlImpl: Control<
            EventFuture = EventFuture<'static, SyncEventFutureSharedState>
        > + 'a + Sized,
        <I as Init<'a>>::Input: Sized,
        <I as Init<'a>>::Output: Sized,
    {
        Self::new_boxed_from_init_with_config_inner::<I>(b, None)
    }

    pub fn new_boxed_from_init_with_config<I: Init<'a>>(
        b: &'a mut BlackBox,
        config: I::Config,
    ) -> Self
    where
        <I as Init<'a>>::ControlImpl: Control<
            EventFuture = EventFuture<'static, SyncEventFutureSharedState>
        > + 'a + Sized,
        <I as Init<'a>>::Input: Sized,
        <I as Init<'a>>::Output: Sized,
    {
        Self::new_boxed_from_init_with_config_inner::<I>(b, Some(config))
    }

    fn new_boxed_from_init_with_config_inner<I: Init<'a>>(
        b: &'a mut BlackBox,
        config: Option<I::Config>,
    ) -> Self
    where
        <I as Init<'a>>::ControlImpl: Control<
            EventFuture = EventFuture<'static, SyncEventFutureSharedState>
        > + 'a + Sized,
        <I as Init<'a>>::Input: Sized,
        <I as Init<'a>>::Output: Sized,
    {
        // TODO: allow to be fallible?
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

impl<'a, T: TuiTypes> Tui<'a, T> {
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
