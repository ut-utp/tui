//! Colours used in the tui.

use crate::env::COLOUR_PALETTE_ENV_VAR;

use tui::style::Color as TuiColour;

macro_rules! palette {
    ($($field:tt $(= $default:expr)?),* $(,)?) => {
        pub trait ColourPalette { $(
            const $field: TuiColour $(= $default)?;
        )*}

        #[allow(non_snake_case)]
        pub struct CurrentPalette {
            $(pub $field: TuiColour,)*
        }

        impl CurrentPalette {
            fn from<P: ColourPalette>(_p: P) -> Self {
                Self {
                    $($field: P::$field,)*
                }
            }
        }

        impl<P: ColourPalette> From<P> for CurrentPalette {
            fn from(p: P) -> CurrentPalette {
                Self::from(p)
            }
        }
    };
}

macro_rules! declare_palette {
    ($nom:ident = { $($field:ident: $val:expr),* $(,)? }) => {
        pub struct $nom;

        impl ColourPalette for $nom {
            $(const $field: TuiColour = $val;)*
        }
    };
}

palette! {
    Data = TuiColour::LightGreen,
    Inst = TuiColour::LightCyan,
    Name = TuiColour::Gray,
    Addr = TuiColour::Gray,
    PC = TuiColour::Rgb(0xFF, 0x97, 0x40),
    Num = TuiColour::Rgb(0xFF, 0x97, 0x40),

    DataT = TuiColour::Green,
    AddrT = TuiColour::DarkGray,
    NumT = TuiColour::Rgb(0xFF, 0xC7, 0x70),

    Breakpoint = TuiColour::Rgb(0xCC, 0x02, 0x02),
    Watchpoint = TuiColour::Rgb(0x30, 0x49, 0xDE),
    BWHighlight = TuiColour::Magenta,

    Privilege = TuiColour::Cyan,
    Priority = TuiColour::Red,
    n_bit = TuiColour::Rgb(0xE9, 0xA3, 0xC9),
    z_bit = TuiColour::White,
    p_bit = TuiColour::Rgb(0xA1, 0xD7, 0x6A),

    MemRegHighlight = TuiColour::Gray,
    RegHighlight = TuiColour::Magenta,

    ConsoleIn = TuiColour::Rgb(0xFF, 0x97, 0x40),
    ConsoleOut = TuiColour::Rgb(0xFF, 0x97, 0x40),
    ConsoleHelp = TuiColour::Gray,
    ConsoleRequest = TuiColour::Red,
    InvalidInput = TuiColour:: Red,

    Run = TuiColour::Green,
    Pause = TuiColour::Yellow,
    Error = TuiColour::LightRed,
    Success = TuiColour::Green,
    Halted = TuiColour::White,
    Depth = TuiColour::LightYellow,

    Highlight = TuiColour::Cyan,

    Focus = TuiColour::Red,
    Reset = TuiColour::Red,
    LoadText = TuiColour::Cyan,
    LoadNoChanges = TuiColour::Reset,
    LoadPendingChanges = TuiColour::Red,

    Title = TuiColour::Rgb(0xFF, 0x97, 0x40),
    Border = TuiColour::Blue,
    Help = TuiColour::Green,

    Modeline = TuiColour::White,
    StepButtons = TuiColour::Cyan,
    LoadB = TuiColour::White,
    mDefault = TuiColour::DarkGray,

    CallStackSupervisorMode = TuiColour::Red,
    CallStackUserMode = TuiColour::Green,

    InProgress = TuiColour::Magenta,
    Disabled = TuiColour::DarkGray,
}

declare_palette! { DefaultPalette = { }}

// declare_palette! { Dark = {
//     MemoryView: TuiColour::Blue,

//     Modeline: TuiColour::Reset,
// }}

lazy_static::lazy_static! {
    pub static ref PALETTE: CurrentPalette = {
        if let Some(palette) = std::env::var_os(COLOUR_PALETTE_ENV_VAR) {
            match palette.to_str().unwrap() {
                "nord" => todo!(),
                _ => DefaultPalette.into(),
            }
        } else {
            DefaultPalette.into()
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! c { ($nom:tt) => { (*$crate::colours::PALETTE).$nom }; }

pub use crate::c;
