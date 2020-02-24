//! TODO

use lc3_application_support::init::{BlackBox, Init};
use lc3_application_support::shim_support::Shims;
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{EventFuture, SyncEventFutureSharedState};
use lc3_traits::control::control::Control;

use std::sync::Mutex;

pub struct Tui<'a, 'int, C, I = SourceShim, O = Mutex<Vec<u8>>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub(in crate::tui) sim: &'a mut C,
    pub(in crate::tui) input: Option<&'a I>,
    pub(in crate::tui) output: Option<&'a O>,
    pub(in crate::tui) shims: Option<Shims<'int>>,
}

impl<'a, 'int, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, 'int, C, I, O> {
    pub fn new(sim: &'a mut C) -> Self {
        Self {
            sim,
            input: None,
            output: None,
            shims: None,
        }
    }

    // The concern with offering this interface instead of the below is that
    // it's more likely users could attach a set of shim peripherals that aren't
    // actually being used with the simulator.
    //
    // But, in reality, I think this is equally likely either way and this is a
    // nicer API.
    pub fn attach_shims(mut self, shims: Shims<'int>) -> Self {
        // self.shims = Some(Shims::from_peripheral_set(shims));
        self.shims = Some(shims);
        self
    }

    pub fn attach_input_output(mut self, input: &'a I, output: &'a O) -> Self {
        self.input = Some(input);
        self.output = Some(output);
        self
    }

    // pub fn run(self) -> Result<(), failure::Error> {
    //     loop {

    //     }
    // }
}

type DynControl<'a> = (dyn Control<EventFuture = EventFuture<'a, SyncEventFutureSharedState>> + 'a);
type DynInputSink<'a> = (dyn InputSink + 'a);
type DynOutputSource<'a> = (dyn OutputSource + 'a);

pub type DynTui<'a, 'int> = Tui<'a, 'int, DynControl<'a>, DynInputSink<'a>, DynOutputSource<'a>>;

impl<'a, 'int> DynTui<'a, 'int> {
    pub fn new_boxed<C>(sim: &'a mut C) -> Self
    where
        C: Control<EventFuture = EventFuture<'a, SyncEventFutureSharedState>> + 'a,
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
        <I as Init<'a>>::ControlImpl: Control<EventFuture = EventFuture<'a, SyncEventFutureSharedState>> + 'a,
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
