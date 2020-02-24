use lc3_tui::DynTui;
use lc3_application_support::init::{BlackBox, SimDevice, SimWithRpcDevice};

use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;
use flexi_logger::{Logger, opt_format};

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
    // fn setup<'a>(&self, b: &'a mut BlackBox) -> DynTui<'a, 'static> {
    fn setup<'a, 'b: 'a>(&'a self, b: &'b mut BlackBox) -> DynTui<'b, 'static> {
        use DeviceType::*;

        match self {
            Board => unimplemented!(),
            DeviceType::Sim => DynTui::new_boxed_from_init::<SimDevice>(b),
            DeviceType::SimWithRpc => DynTui::<'b, 'static>::new_boxed_from_init::<SimWithRpcDevice>(b),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "lc3-tui", about = "The UTP LC-3 TUI.")]
struct Args {
    /// Type of device to use with the TUI.
    #[structopt(short, long, default_value = "sim")]
    device: DeviceType,

    /// Enable logging
    // (TODO!)
    #[structopt(short, long)]
    debug: bool,

    /// Program file (optional)
    #[structopt(parse(from_os_str))]
    program_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<failure::Error>> {
    let options = Args::from_args();

    // TODO!
    if options.debug {
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

    // tui.run();

    Ok(())
}
