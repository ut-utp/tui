//! Colours used in the tui.

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
    PC = TuiColour::Rgb(0xFF, 0x97, 0x40),

    Breakpoint = TuiColour::Rgb(0xCC, 0x02, 0x02),
    Watchpoint = TuiColour::Rgb(0x30, 0x49, 0xDE),
    BWHighlight = TuiColour::Magenta,

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

    Highlight = TuiColour::Cyan,

    Focus = TuiColour::Red,
    Reset = TuiColour::Red,
    Load = TuiColour::Cyan,

    Title = TuiColour::Rgb(0xFF, 0x97, 0x40),
    Border = TuiColour::Blue,
    Help = TuiColour::Green,

    Modeline = TuiColour::White,
    StepB = TuiColour::Cyan,
    LoadB = TuiColour::White,
    mDefault = TuiColour::DarkGray,

}

declare_palette! { DefaultPalette = { }}

// declare_palette! { Dark = {
//     MemoryView: TuiColour::Blue,

//     Modeline: TuiColour::Reset,
// }}

lazy_static::lazy_static! {
    pub static ref PALETTE: CurrentPalette = {
        // TODO: select based on env vars!
        DefaultPalette.into()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! c { ($nom:tt) => { (*$crate::colours::PALETTE).$nom }; }

pub use crate::c;
