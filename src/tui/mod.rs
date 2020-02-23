//! TODO

use lc3_application_support::shim_support::{ShimPeripheralSet, Shims};
use lc3_application_support::io_peripherals::{InputSink, OutputSource};

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{EventFuture, SyncEventFutureSharedState};
use lc3_traits::control::control::Control;

use std::sync::Mutex;

pub struct Tui<'a, C, I = SourceShim, O = Mutex<Vec<u8>>>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
{
    pub(in crate::tui) sim: &'a mut C,
    pub(in crate::tui) input: Option<&'a I>,
    pub(in crate::tui) output: Option<&'a O>,
    pub(in crate::tui) shims: Option<Shims<'a>>,
}

impl<'a, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, C, I, O> {
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
    pub fn attach_shims(mut self, shims: &'a ShimPeripheralSet<'a>) -> Self {
        self.shims = Some(Shims::from_peripheral_set(shims));
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

pub type DynTui<'a> = Tui<'a, DynControl<'a>, DynInputSink<'a>, DynOutputSource<'a>>;

impl<'a> DynTui<'a> {
    pub fn new_boxed<C, I, O>(sim: &'a mut C, input: &'a I, output: &'a O) -> Self
    where
        C: Control<EventFuture = EventFuture<'a, SyncEventFutureSharedState>> + 'a,
        I: InputSink + 'a,
        O: OutputSource + 'a,
    {
        Self::new(sim, input, output)
    }
}
