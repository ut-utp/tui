//! TODO

use lc3_shims::peripherals::{Source, SourceShim};
use lc3_shims::peripherals::Sink;
use lc3_traits::peripherals::PeripheralSet;
use lc3_shims::peripherals::{GpioShim, AdcShim, PwmShim, TimersShim, ClockShim, InputShim, OutputShim};
use lc3_traits::control::rpc::{EventFuture, SyncEventFutureSharedState};
use lc3_traits::control::control::Control;

use std::ops::DerefMut;
use std::sync::{Arc, Mutex, RwLock};

pub trait InputSink {
    // Note: probably only ASCII for now.
    //
    // Should return `None` on errors/invalid chars.
    fn put_char(&self, c: char) -> Option<()>;
}

pub trait OutputSource {
    // Note: probably only ASCII for now.
    //
    // Should return `None` when no characters are available.
    fn get_chars(&self) -> Option<String>;
}

pub(crate) struct Shims<'a> {
    gpio: Arc<RwLock<GpioShim<'a>>>,
    adc: Arc<RwLock<AdcShim>>,
    pwm: Arc<RwLock<PwmShim>>,
    timers: Arc<RwLock<TimersShim<'a>>>,
    clock: Arc<RwLock<ClockShim>>,
}

type ShimPeripheralSet<'a> = PeripheralSet<
    'a,
    Arc<RwLock<GpioShim<'a>>>,
    Arc<RwLock<AdcShim>>,
    Arc<RwLock<PwmShim>>,
    Arc<RwLock<TimersShim<'a>>>,
    Arc<RwLock<ClockShim>>,
    Arc<Mutex<InputShim<'a, 'a>>>,
    Arc<Mutex<OutputShim<'a, 'a>>>,
>;

// This is fine!
impl InputSink for &SourceShim {
    fn put_char(&self, c: char) -> Option<()> {
        self.push(c);
        Some(())
    }
}

// This is less fine.. (should maybe be as generic as the Sink trait, but that
// is not trivial) (TODO)
impl OutputSource for Box<Mutex<Vec<u8>>> {
    fn get_chars(&self) -> Option<String> {
        // This is bad, maybe:
        let mut vec = self.lock().unwrap();
        if !vec.is_empty() {
            let v = std::mem::replace(vec.deref_mut(), Vec::new());

            // TODO: maybe handle non-utf8 char better than this.
            String::from_utf8(v).ok()
        } else {
            None
        }
    }
}


// TODO: move this to the right place...
// fn new_shim_peripherals_set<'a>() -> (ShimPeripheralSet<'a>, &'static SourceShim, &'static Mutex<Vec<u8>>) { // TODO: char instead of u8...
fn new_shim_peripherals_set<'a, I, O>(input: &'a I, output: &'a O) -> (ShimPeripheralSet<'a>, &'a impl InputSink, &'a impl OutputSource)
where
    I: InputSink + Source + Send + Sync + 'a,
    O: OutputSource + Sink + Send + Sync + 'a,
{
    let gpio_shim = Arc::new(RwLock::new(GpioShim::default()));
    let adc_shim = Arc::new(RwLock::new(AdcShim::default()));
    let pwm_shim = Arc::new(RwLock::new(PwmShim::default()));
    let timer_shim = Arc::new(RwLock::new(TimersShim::default()));
    let clock_shim = Arc::new(RwLock::new(ClockShim::default()));

    let input_shim = Arc::new(Mutex::new(InputShim::with_ref(input)));
    let output_shim = Arc::new(Mutex::new(OutputShim::with_ref(output)));

    (PeripheralSet::new(gpio_shim, adc_shim, pwm_shim, timer_shim, clock_shim, input_shim, output_shim),
        input,
        output,
    )
}

impl<'a> Shims<'a> {
    pub(crate) fn from_peripheral_set(p: &ShimPeripheralSet<'a>) -> Self {
        Self {
            gpio: p.gpio.clone(),
            adc: p.adc.clone(),
            pwm: p.pwm.clone(),
            timers: p.timers.clone(),
            clock: p.clock.clone(),
        }
    }
}

pub struct Tui<'a, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> {
    sim: &'a mut C,
    input: &'a I,
    output: &'a O,
    shims: Option<Shims<'a>>,
}

impl<'a, C: Control + ?Sized + 'a, I: InputSink + ?Sized + 'a, O: OutputSource + ?Sized + 'a> Tui<'a, C, I, O> {
    pub fn new(sim: &'a mut C, input: &'a I, output: &'a O) -> Self {
        Self {
            sim,
            input,
            output,
            shims: None,
        }
    }

    // The concern with offerring this interface instead of the below is that
    // it's more likely users could attach a set of shim peripherals that aren't
    // actually being used with the simulator.
    //
    // But, in reality, I think this is equally likely either way and this is a
    // nicer API.
    pub fn attach_shims(mut self, shims: &'a ShimPeripheralSet<'a>) -> Self {
        self.shims = Some(Shims::from_peripheral_set(shims));
        self
    }

    // pub fn new_with_shims(sim: C, input: &'a I, output: &'a O, shims: &'a ShimPeripheralSet<'a>) -> Self {
    //     Self {
    //         sim,
    //         input,
    //         output,
    //         shims: Some(Shims::from_peripheral_set(shims))
    //     }
    // }

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
