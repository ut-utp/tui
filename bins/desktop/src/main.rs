//! TODO!

use anyhow::Context;
use flexi_logger::{Logger, opt_format};
use structopt::StructOpt;

use lc3_tui::{DynTui, ProgramSource};
use lc3_tui::layout;
use lc3_application_support::init::{
    BlackBox, BoardDevice, BoardConfig, SimDevice, SimWithRpcDevice
};

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::fmt::{self, Display};

#[derive(Debug)]
enum DeviceType {
    // Board { path: PathBuf, baud_rate: u32 }, // TODO: options?
    Board(BoardConfig<PathBuf>), // TODO: options?
    Sim,
    SimWithRpc,
}

impl FromStr for DeviceType {
    type Err = &'static str;

    fn from_str(sim: &str) -> Result<Self, Self::Err> {
        use DeviceType::*;

        match sim {
            "board" => Ok(Board(Default::default())),
            "sim" => Ok(Sim),
            "sim-rpc" => Ok(SimWithRpc),
            "websocket" => unimplemented!(), // TODO!
            s if s.starts_with("board=") =>{
                let mut iter = s.split("=").skip(1);

                let config = iter.next().expect(format!("Expected a board path, got nothing. ({})", s).as_str());
                let mut iter = config.split(":");

                let board = iter.next().expect(format!("Expected a board path, got nothing. ({})", s).as_str());
                let baud_rate = if let Some(baud_rate) = iter.next() {
                    baud_rate.parse().expect("A baud rate when `:` is after the board path.")
                } else {
                    1_500_000
                };

                let path = PathBuf::from(board);

                Ok(Board(BoardConfig::new(path, baud_rate)))

            },
            _ => Err("Could not parse device type!")
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DeviceType::*;

        let s = if fmt.alternate() {
            match self {
                Board(c) => {
                    return write!(fmt, "on a board: {}", c.path.display());
                },
                Sim => "locally",
                SimWithRpc => "locally via rpc",
            }
        } else {
            match self {
                Board(_) => "Board",
                Sim => "Simulator",
                SimWithRpc => "Simulator With RPC",
            }
        };

        write!(fmt, "{}", s)
    }
}

impl DeviceType {
    fn setup<'a, 'b: 'a>(&'a self, b: &'b mut BlackBox) -> DynTui<'b, 'static> {
        use DeviceType::*;

        match self {
            Board(config) => DynTui::new_boxed_from_init_with_config::<BoardDevice<'_, _, PathBuf>>(b, config.clone()), // TODO: config!
            DeviceType::Sim => DynTui::new_boxed_from_init::<SimDevice>(b),
            DeviceType::SimWithRpc => DynTui::<'b, 'static>::new_boxed_from_init::<SimWithRpcDevice>(b),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TimeInMs(pub Duration);

impl From<TimeInMs> for Duration {
    fn from(time: TimeInMs) -> Self {
        time.0
    }
}

impl FromStr for TimeInMs {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TimeInMs(Duration::from_millis(s.parse()?)))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "lc3-tui", about = "The UTP LC-3 TUI.")]
struct Args {
    /// Type of device to use with the TUI.
    #[structopt(short, long, default_value = "sim", help = "TODO")]
    device: DeviceType,

    /// Enable logging
    // (TODO!)
    #[structopt(short = "v", long, help = "TODO")]
    logging: bool,

    /// Program source (optional)
    #[structopt(help = "TODO")]
    program_source: Option<ProgramSource>,

    /// Update period
    #[structopt(short, long, default_value = "50", help = "Update period in milliseconds")]
    update_period: TimeInMs,

    /// Build/run with without the OS
    #[structopt(long, help = "Builds .asm files without the UTP LC-3 OS and does *not* skip past the OS on loads and resets when this is set")]
    without_os: bool,
}

pub fn with_stack_size<R: Send + 'static, F: FnOnce() -> R + Send + 'static>(ss: usize, f: F) -> R {
    let child = std::thread::Builder::new()
        .stack_size(ss)
        .spawn(f)
        .unwrap();

    child.join().unwrap()
}


fn main() -> anyhow::Result<()> {
    let options = Args::from_args();

    // TODO!
    if options.logging {
        Logger::with_env_or_str("loggy=trace")
            .log_to_file()
            .directory("log_files")
            .format(opt_format)
            .start()
            .unwrap();
    }

    with_stack_size(1024 * 1024 * 16, move || {
        let mut b = BlackBox::new();
        let mut tui = options.device.setup(&mut b);

        if let Some(s) = options.program_source {
            tui.set_program_source(s);
        }

        tui.set_use_os(!options.without_os);

        let name = format!("UTP LC-3 Simulator (running {:#})", options.device);

        let no_extra_tabs = Vec::new();
        let layout = layout::layout(
            Some(name.as_ref()),
            no_extra_tabs
        );

        tui.set_update_period(options.update_period.into());
        tui.run_with_crossterm(Some(layout))?;

        println!("Goodbye! 👋");
        Ok(())
    })
}
