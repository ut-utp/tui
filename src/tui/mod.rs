//! TODO

use lc3_application_support::init::{BlackBox, Init};
use lc3_application_support::shim_support::Shims;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{EventFuture, SyncEventFutureSharedState};
use lc3_traits::control::control::Control;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

pub mod run;
pub mod events;
pub mod widget;

pub type Res<T> = Result<T, failure::Error>;

#[allow(explicit_outlives_requirements)]
pub struct TuiData<'a, 'int, C, I = SourceShim, O = Mutex<Vec<u8>>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub(in crate) sim: &'a mut C,
    pub(in crate) input: Option<&'a I>,
    pub(in crate) output: Option<&'a O>,
    pub(in crate) shims: Option<Shims<'int>>,

    pub(in crate) program_path: Option<PathBuf>,
}

#[allow(explicit_outlives_requirements)]
pub struct Tui<'a, 'int, C, I = SourceShim, O = Mutex<Vec<u8>>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub(in crate::tui) data: TuiData<'a, 'int, C, I, O>,

    pub(in crate::tui) update_period: Duration,
    // pub(in crate::tui)

}

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
    pub fn new(sim: &'a mut C) -> Self {
        Self {
            data: TuiData {
                sim,
                input: None,
                output: None,
                shims: None,

                program_path: None,
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
        let (sim, shims, input, output) = I::init(b);

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
    pub fn set_program_path(&mut self, path: PathBuf) -> &mut Self {
        self.data.program_path = Some(path);
        self
    }

    pub fn set_update_period(&mut self, period: Duration) -> &mut Self {
        self.update_period = period;
        self
    }
}
