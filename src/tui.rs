//! TODO

use lc3_shims::peripherals::SourceShim;
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
    fn put_char(&mut self, c: char) -> Option<()>;
}

pub trait OutputSource {
    // Note: probably only ASCII for now.
    //
    // Should return `None` when no characters are available.
    fn get_chars(&mut self) -> Option<String>;
}

// TODO: impl the above for the Input and Output Shim.

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
    fn put_char(&mut self, c: char) -> Option<()> {
        self.push(c);
        Some(())
    }
}

// This is less fine.. (should maybe be as generic as the Sink trait, but that
// is not trivial) (TODO)
impl OutputSource for Box<Mutex<Vec<u8>>> {
    fn get_chars(&mut self) -> Option<String> {
        // This is bad, maybe:
        let vec = self.lock().unwrap();
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
fn new_shim_peripherals_set<'a>() -> (ShimPeripheralSet<'a>, &'static SourceShim, &'static Mutex<Vec<u8>>) { // TODO: char instead of u8...
    let gpio_shim = Arc::new(RwLock::new(GpioShim::default()));
    let adc_shim = Arc::new(RwLock::new(AdcShim::default()));
    let pwm_shim = Arc::new(RwLock::new(PwmShim::default()));
    let timer_shim = Arc::new(RwLock::new(TimersShim::default()));
    let clock_shim = Arc::new(RwLock::new(ClockShim::default()));

    let source_shim = Box::new(SourceShim::new());
    let source_shim = Box::leak(source_shim); // TODO: don't do this!!!!!
    let input_shim = Arc::new(Mutex::new(InputShim::with_ref(source_shim)));

    // TODO: use a ring buffer or something instead of a Vec for the Sink impl.
    let console_output = Box::new(Mutex::new(Vec::new()));
    let console_output = Box::leak(console_output); // TODO: don't do this!!!
    let output_shim = Arc::new(Mutex::new(OutputShim::with_ref(console_output)));

    (PeripheralSet::new(gpio_shim, adc_shim, pwm_shim, timer_shim, clock_shim, input_shim, output_shim),
        source_shim,
        console_output,
    )
}

impl<'a> Shims<'a> {
    fn from_peripheral_set(p: &ShimPeripheralSet<'a>) -> Self {
        Self {
            gpio: p.gpio.clone(),
            adc: p.adc.clone(),
            pwm: p.pwm.clone(),
            timers: p.timers.clone(),
            clock: p.clock.clone(),
        }
    }
}

pub struct Tui<'a, C: Control + 'a, I: InputSink + 'a, O: OutputSource + 'a> {
    sim: C,
    input: I,
    output: O,
    shims: Option<Shims<'a>>,
}

impl<'a, C: Control + 'a, I: InputSink + 'a, O: OutputSource + 'a> Tui<'a, C, I, O> {
    pub fn new(sim: C, input: I, output: O) -> Self {
        Self {
            sim,
            input,
            output,
            shims: None,
        }
    }

    pub fn new_with_shims(sim: C, shims: &ShimPeripheralSet<'a>) -> Self {
        Self {
            sim,

            shims: Some()
        }
    }

    pub fn run(self) -> Result<(), failure::Error> {
        loop {

        }
    }
}

type BoxedControl<'a> = Box<(dyn Control<EventFuture = EventFuture<'a, SyncEventFutureSharedState>> + 'a)>;
// type BoxedInputSink<'a> = Box<(dyn Control<EventFuture = EventFuture<'a, SyncEventFutureSharedState>> + 'a)>;
// type BoxedOutputSink<'a> =

// impl Tui<Box<dyn Control<EventFuture = EventFuture<SyncEventFutureSharedState>>>, Box<dyn InputSink>, Box<dyn OutputSource>> {

// }
