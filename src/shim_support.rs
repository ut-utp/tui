//! TODO!

use crate::io_peripherals::{InputSink, OutputSource};

use lc3_traits::peripherals::PeripheralSet;
use lc3_shims::peripherals::{Source, Sink};
use lc3_shims::peripherals::{GpioShim, AdcShim, PwmShim, TimersShim, ClockShim, InputShim, OutputShim};

use std::sync::{Arc, Mutex, RwLock};

pub(crate) struct Shims<'a> {
    pub(crate) gpio: Arc<RwLock<GpioShim<'a>>>,
    pub(crate) adc: Arc<RwLock<AdcShim>>,
    pub(crate) pwm: Arc<RwLock<PwmShim>>,
    pub(crate) timers: Arc<RwLock<TimersShim<'a>>>,
    pub(crate) clock: Arc<RwLock<ClockShim>>,
}

pub type ShimPeripheralSet<'a> = PeripheralSet<
    'a,
    Arc<RwLock<GpioShim<'a>>>,
    Arc<RwLock<AdcShim>>,
    Arc<RwLock<PwmShim>>,
    Arc<RwLock<TimersShim<'a>>>,
    Arc<RwLock<ClockShim>>,
    Arc<Mutex<InputShim<'a, 'a>>>,
    Arc<Mutex<OutputShim<'a, 'a>>>,
>;

pub fn new_shim_peripherals_set<'a, I, O>(input: &'a I, output: &'a O)
        -> (ShimPeripheralSet<'a>, &'a impl InputSink, &'a impl OutputSource)
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
