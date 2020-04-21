//! Colours used in the tui.

use tui::style::Color as TuiColour;

macro_rules! palette {
    ($($field:tt $(= $default:expr)?),* $(,)?) => {
        pub trait ColourPalette { $(
            const $field: TuiColour $(= $default)?;
        )*}

        #[allow(non_snake_case)]
        pub struct CurrentPalette {
            $($field: TuiColour,)*
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
    Data = TuiColour::Green,
    Insts = TuiColour::LightCyan,
    MemoryView = TuiColour::Gray,

    Breakpoint = TuiColour::Red,
    Watchpoint = TuiColour::Rgb(0xFF, 0x97, 0x40),

    Pause = TuiColour::Yellow,
    Error = TuiColour::LightRed,
    Success = TuiColour::Green,

    Highlight = TuiColour::Cyan,

    Focus = TuiColour::Red,
    Reset = TuiColour::Red,

    Title = TuiColour::Rgb(0xFF, 0x97, 0x40),

}

declare_palette! { DefaultPalette = { }}

// declare_palette! { Dark = {
//     MemoryView: TuiColour::Blue,

//     Modeline: TuiColour::Reset,
// }}

lazy_static::lazy_static! {
    static ref PALETTE: CurrentPalette = {
        // TODO: select based on env vars!
        DefaultPalette.into()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! c { ($nom:tt) => { *CurrentPalette.$nom }; }

pub use crate::c;
