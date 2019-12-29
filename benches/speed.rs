//! A quick benchmark that measures how long it takes to execute a simple
//! program in a few configurations.

// TODO: move this to the top of the workspace! (doesn't belong in TUI)
// TODO: have CI run this and give us reports

use lc3_isa::{Word, program, util::AssembledProgram};
use lc3_os::OS_IMAGE;

use lazy_static::lazy_static;

// TODO: new macro that basically does the below + sets the orig hook
// TODO: have obj conv set the orig hook

const fn fib_program_executed_insn_count(num_iters: Word) -> u64 {
    (159 * (num_iters as u64) + 347)
}

fn build_fib_memory_image(num_iters: Word) -> MemoryDump {
    const F: Word = 24;

    let prog: AssembledProgram = program! {
        .ORIG #0x3000;
        BRnzp @START;

        @NUM_ITERS .FILL #num_iters;
        @FIB_NUM .FILL #F;

        @START
        LD R1, @NUM_ITERS;

        @LOOP
            BRz @END;

            @FIB_START
                AND R3, R3, #0; // R3 = 0
                ADD R4, R3, #1; // R4 = 1

                LD R2, @FIB_NUM;

            @FIB
                // ADD R2, R2, #0;
                BRz @END_FIB;

                ADD R5, R3, #0;
                ADD R3, R4, #0;
                ADD R4, R4, R5;

                ADD R2, R2, #-1;
                BRnzp @FIB;

            @END_FIB
                ADD R0, R3, #0;
                OUT;

            ADD R1, R1, #-1;
            BRnzp @LOOP;

        @END
            HALT;
    }.into();

    let mut image = OS_IMAGE.clone();
    image.layer_loadable(&prog);

    image
}

fn fib_closed_form(n: Word) -> u64 {
    let g: f64 = (1. + 5f64.sqrt()) / 2.0;
    let r: f64 = (g.powi(n as i32) - (-g).powi(-(n as i32))) / 5f64.sqrt();

    r as u64
}

use lc3_baseline_sim::interp::{Interpreter, InterpreterBuilder, PeripheralInterruptFlags, InstructionInterpreter};
use lc3_traits::control::{Control, rpc::{Encoding, Transport, TransparentEncoding, Device, Controller}};
use lc3_shims::{memory::MemoryShim, peripherals::PeripheralsShim};
use lc3_traits::peripherals::stubs::PeripheralsStub;
use lc3_isa::util::MemoryDump;

pub fn bare_interpreter<'a, 'b>(program: MemoryDump, flags: &'b PeripheralInterruptFlags) -> Interpreter<'b, MemoryShim, PeripheralsStub<'b>> {
    let memory = MemoryShim::new(*program);

    let mut interp: Interpreter<'b, MemoryShim, PeripheralsStub<'b>> = InterpreterBuilder::new()
        .with_defaults()
        .with_memory(memory)
        .build();

    interp.reset();
    interp.init(flags);

    interp
}

use lc3_baseline_sim::sim::Simulator;
type Sim<'a> = Simulator<'a, Interpreter<'a, MemoryShim, PeripheralsStub<'a>>>;

pub fn simulator<'a>(program: MemoryDump, flags: &'a PeripheralInterruptFlags) -> Sim<'a> {
    let mut sim = Simulator::new(bare_interpreter(program, flags));
    sim.reset();

    sim
}

use std::thread::Builder as ThreadBuilder;

static FLAGS: PeripheralInterruptFlags = PeripheralInterruptFlags::new();
fn device_thread<Enc: 'static, Transp: 'static>(rx: Receiver<()>, mut device: Device<Enc, Transp, Sim<'static>>, program: MemoryDump)
where
    Enc: Encoding + Send,
    Transp: Transport<Enc::Encoded> + Send,
{
    ThreadBuilder::new()
        .name("Device Thread".to_string())
        .stack_size(1024 * 1024 * 4)
        .spawn(move || {
            let mut sim = simulator(program, &FLAGS);

            loop {
                device.step(&mut sim);
                if let State::Halted = sim.get_state() {
                    if let Ok(()) = rx.try_recv() {
                        break
                    }
                }
            }
        });
}

use lc3_traits::control::rpc::SyncEventFutureSharedState;

lazy_static! {
    static ref STATE: SyncEventFutureSharedState = SyncEventFutureSharedState::new();
}

use lc3_traits::control::rpc::mpsc_sync_pair;
use std::sync::mpsc::{channel, Receiver, Sender};

// TODO: test spin vs. sleep
pub fn remote_simulator/*<'a, 'b: 'a>*/(program: MemoryDump) -> (Sender<()>, impl Control) {
    let (controller, device) = mpsc_sync_pair(&STATE);
    let (tx, rx) = channel();

    device_thread::<TransparentEncoding, _>(rx, device, program);

    (tx, controller)
}

//// Benches ////


use criterion::{BenchmarkId, BatchSize, Criterion, Throughput};
use lc3_baseline_sim::interp::MachineState;

// const ITERS: [Word; 10] = [1, 10, 100, 500, 1000, 5000, 10000, 25000, 50000, 65535];
const ITERS: [Word; 5] = [1, 10, 100, 500, 1000];
// 506, 1937, 16247, 79847, 159347, 795347,
// // 159 * x + 347

use lc3_traits::control::State;

fn bench_fib(c: &mut Criterion) {
    let flags = PeripheralInterruptFlags::new();
    let mut group = c.benchmark_group("fib(24)");

    for num_iter in ITERS.iter() {
        group.throughput(Throughput::Elements(fib_program_executed_insn_count(*num_iter)));

        group.bench_with_input(BenchmarkId::new("Bare Interpreter - step", *num_iter), num_iter, |b, num| {
            let mut int = bare_interpreter(build_fib_memory_image(*num), &flags);
            b.iter(|| {
                int.reset();
                while let MachineState::Running = int.step() { }
            })
        });

        group.bench_with_input(BenchmarkId::new("Simulator - step", *num_iter), num_iter, |b, num| {
            let mut sim = simulator(build_fib_memory_image(*num), &flags);
            b.iter(|| {
                sim.reset();
                while let State::Paused = sim.step() { }
            })
        });

        group.bench_with_input(BenchmarkId::new("Remote Simulator - step: mpsc, transparent", *num_iter), num_iter, |b, num| {
            let (halt, mut sim) = remote_simulator(build_fib_memory_image(*num));
            b.iter(|| {
                sim.reset();
                while let State::Paused = sim.step() { }
            });

            halt.send(());
        });
    }
}

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, bench_fib);
criterion_main!(benches);
