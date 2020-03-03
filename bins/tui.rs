use lc3_tui::DynTui;
use lc3_tui::layout;
use lc3_application_support::init::{BlackBox, SimDevice, SimWithRpcDevice};

use structopt::StructOpt;
use flexi_logger::{Logger, opt_format};

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
enum DeviceType {
    Board,
    Sim,
    SimWithRpc,
}

impl FromStr for DeviceType {
    type Err = &'static str;

    fn from_str(sim: &str) -> Result<Self, Self::Err> {
        use DeviceType::*;

        match sim {
            "board" => Ok(Board),
            "sim" => Ok(Sim),
            "sim-rpc" => Ok(SimWithRpc),
            "websocket" => unimplemented!(), // TODO!
            _ => Err("Could not parse device type!")
        }
    }
}

impl DeviceType {
    fn setup<'a, 'b: 'a>(&'a self, b: &'b mut BlackBox) -> DynTui<'b, 'static> {
        use DeviceType::*;

        match self {
            Board => unimplemented!(),
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

    /// Program file (optional)
    #[structopt(parse(from_os_str), help = "TODO")]
    program_file: Option<PathBuf>,

    /// Update period
    #[structopt(short, long, default_value = "80", help = "Update period in milliseconds")]
    update_period: TimeInMs,
}

fn main() -> Result<(), failure::Error> {
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

    let mut b = BlackBox::new();
    let mut tui = options.device.setup(&mut b);

    if let Some(p) = options.program_file {
        tui.set_program_path(p);
    }


    let layout = layout::layout_tabs();

    tui.set_update_period(options.update_period.into());
    tui.run_with_crossterm(Some(layout))?;

    println!("Good bye! 👋");
    Ok(())
}
