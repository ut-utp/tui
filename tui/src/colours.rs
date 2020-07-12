//! Colours used in the tui.

use crate::env::COLOUR_PALETTE_ENV_VAR;

use tui::style::Color as TuiColour;

macro_rules! palette {
    ($($field:tt $(= $default:expr)?),* $(,)?) => {
        pub trait ColourPalette { $(
            #[allow(non_upper_case_globals)]
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
        #[derive(Debug)]
        pub struct $nom;

        impl $crate::colours::ColourPalette for $nom {
            $(
                #[allow(non_upper_case_globals)]
                const $field: TuiColour = $val;
            )*
        }
    };
}

palette! {
    Data = TuiColour::LightGreen,
    Inst = TuiColour::LightCyan,
    Name = TuiColour::Gray,
    Addr = TuiColour::Gray,
    Pc = TuiColour::Rgb(0xFF, 0x97, 0x40),
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
    ConsolePrompt = TuiColour::Cyan,
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
    LoadText = TuiColour::White,
    LoadNoChanges = TuiColour::Reset,
    LoadPendingChanges = TuiColour::LightRed,

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

// TODO: a dark palette?

/// Colours from the [nord colour palette](https://www.nordtheme.com/docs/colors-and-palettes).
pub mod nord {
    use super::TuiColour;
    macro_rules! rgb {
        ($val:literal) => {{

            let v: u32 = $val;

            // use lc3_isa::Bits;
            // let b = v.u8(0..7);
            // let g = v.u8(8..15);
            // let r = v.u8(16..23);

            let b = (v & 0xFF) as u8;
            let g = (v & 0xFF00 >> 8) as u8;
            let r = (v & 0xFF0000 >> 16) as u8;

            TuiColour::Rgb(r, g, b)
        }};
    }

    /// **Polar Night** is made up of four darker colors that are commonly used
    /// for base elements like backgrounds or text color in _bright ambiance
    /// designs_.
    pub mod polar_night {
        use super::*;

        /// **The origin color or the Polar Night palette.**
        ///
        /// For _dark ambiance designs_, it is used for background and area
        /// coloring while it's not used for syntax highlighting at all because
        /// otherwise it would collide with the same background color.
        ///
        /// For _bright ambiance designs_, it is used for base elements like
        /// plain text, the text editor caret and reserved syntax characters
        /// like curly- and square brackets.
        /// It is rarely used for passive UI elements like borders, but might be
        /// possible to achieve a higher contrast and better visual distinction
        /// (harder/not flat) between larger components.
        pub const NORD0: TuiColour = rgb!(0x2E3440);

        /// **A brighter shade color based on [`nord0`](NORD0).**
        ///
        /// For _dark ambiance_ designs it is used for elevated, more prominent or
        /// focused UI elements like:
        ///
        ///   - status bars and text editor gutters
        ///   - panels, modals and floating popups like notifications or auto
        ///   - completion
        ///   - user interaction/form components like buttons, text/select
        ///   - fields or checkboxes
        ///
        /// It also works fine for more inconspicuous and passive elements like
        /// borders or as dropshadow between different components.
        /// There's currently no official port project that makes use of it for
        /// syntax highlighting.
        ///
        /// For _bright ambiance designs_, it is used for more
        /// subtle/inconspicuous UI text elements that do not need so much
        /// visual attention.
        /// Other use cases are also state animations like a more brighter text
        /// color when a button is hovered, active or focused.
        pub const NORD1: TuiColour = rgb!(0x3B4252);

        /// **An even more brighter shade color of [`nord0`](NORD0).**
        ///
        /// For _dark ambiance designs_, it is used to colorize the currently
        /// active text editor line as well as selection- and text highlighting
        /// color.
        /// For both _bright & dark ambiance designs_ it can also be used as an
        /// brighter variant for the same target elements like [`nord1`](NORD1).
        pub const NORD2: TuiColour = rgb!(0x434C5E);

        /// **The brightest shade color based on [`nord0`](NORD0).**
        ///
        /// For _dark ambiance designs_, it is used for UI elements like indent-
        /// and wrap guide marker.
        /// In the context of code syntax highlighting it is used for comments
        /// and invisible/non-printable characters.
        ///
        /// For _bright ambiance designs_, it is, next to [`nord1`](NORD1) and
        /// [`nord2`](NORD2) as darker variants, also used for the most
        /// subtle/inconspicuous UI text elements that do not need so much
        /// visual attention.
        pub const NORD3: TuiColour = rgb!(0x4C566A);
    }

    use polar_night::*;

    // TODO!
    declare_palette! { PolarNight = {
        Data: TuiColour::White,
        Inst: TuiColour::White,
        Name: TuiColour::White,
        Addr: TuiColour::White,
        Pc: NORD1,
        Num: NORD1,

        DataT: NORD1,
        AddrT: NORD1,
        NumT: NORD1,

        Breakpoint: NORD3,
        Watchpoint: NORD3,
        BWHighlight: NORD3,

        Privilege: NORD2,
        Priority: NORD2,
        n_bit: NORD2,
        z_bit: NORD2,
        p_bit: NORD2,

        MemRegHighlight: NORD3,
        RegHighlight: NORD3,

        ConsoleIn: NORD1,
        ConsoleOut: NORD2,
        ConsolePrompt: NORD3,
        ConsoleHelp: NORD0,
        ConsoleRequest: NORD1,
        InvalidInput: NORD3,

        Run: NORD2,
        Pause: NORD1,
        Error: NORD3,
        Success: NORD2,
        Halted: NORD2,
        Depth: NORD2,

        Highlight: NORD3,

        Focus: NORD3,
        Reset: NORD1,
        LoadText: TuiColour::White,
        LoadNoChanges: NORD0,
        LoadPendingChanges: NORD3,

        Title: TuiColour::White,
        Border: NORD1,
        Help: TuiColour::Gray,

        Modeline: NORD1,
        StepButtons: NORD2,
        LoadB: NORD3,
        mDefault: NORD2,

        CallStackSupervisorMode: NORD3,
        CallStackUserMode: NORD0,

        InProgress: NORD2,
        Disabled: NORD0,
    }}

    // TODO: copy docs
    pub mod snow_storm {
        use super::*;
        pub const NORD4: TuiColour = rgb!(0xD8DEE9);
        pub const NORD5: TuiColour = rgb!(0xE5E9F0);
        pub const NORD6: TuiColour = rgb!(0xECEFF4);
    }

    // TODO!
    declare_palette! { SnowStorm = {

    }}

    /// **Frost** can be described as the heart palette of Nord, a group of four
    /// bluish colors that are commonly used for primary UI component and text
    /// highlighting and essential code syntax elements.
    ///
    /// All colors of this palette are used the same for both _dark & bright
    /// ambiance_ designs.
    pub mod frost {
        use super::*;
        /// **A calm and highly contrasted color reminiscent of frozen polar
        /// water.**
        ///
        /// Used for UI elements that should, next to the primary accent color
        /// [`nord8`](NORD8), stand out and get more visual attention.
        ///
        /// In the context of syntax highlighting it is used for classes, types
        /// and primitives.
        pub const NORD7: TuiColour = rgb!(0x8FBCBB);

        /// **The bright and shiny primary accent color reminiscent of pure and
        /// **clear ice.**
        ///
        /// Used for primary UI elements with main usage purposes that require
        /// the most visual attention.
        /// In the context of syntax highlighting it is used for declarations,
        /// calls and execution statements of functions, methods and routines.
        pub const NORD8: TuiColour = rgb!(0x88C0D0);

        /// **A more darkened and less saturated color reminiscent of arctic
        /// **waters.**
        ///
        /// Used for secondary UI elements that also require more visual
        /// attention than other elements.
        /// In the context of syntax highlighting it is used for language
        /// specific, syntactic and reserved keywords as well as:
        ///   - support characters
        ///   - operators
        ///   - tags
        ///   - units
        ///   - punctuations like (semi)colons, points and commas
        pub const NORD9: TuiColour = rgb!(0x81A1C1);

        /// **A dark and intensive color reminiscent of the deep arctic
        /// **ocean.**
        ///
        /// Used for tertiary UI elements that require more visual attention
        /// than default elements.
        /// In the context of syntax highlighting it is used for pragmas,
        /// comment keywords and pre-processor statements.
        pub const NORD10: TuiColour = rgb!(0x5E81AC);
    }

    // TODO!
    declare_palette! { Frost = {

    }}


    /// **Aurora** consists of five colorful components reminiscent of the
    /// "Aurora borealis“, sometimes referred to as polar lights or northern
    /// lights.
    ///
    /// All colors of this palette are used the same for both _dark & bright
    /// ambiance_ designs.
    pub mod aurora {
        use super::*;

        /// Used for UI elements that are rendering error states like linter
        /// markers and the highlighting of Git `diff` deletions.
        ///
        /// In the context of syntax highlighting it is used to override the
        /// highlighting of syntax elements that are detected as errors.
        pub const NORD11: TuiColour = rgb!(0xBF616A); // red

        /// Rarely used for UI elements, but it may indicate a more advanced or
        /// dangerous functionality.
        ///
        /// In the context of syntax highlighting it is used for special syntax
        /// elements like annotations and decorators.
        pub const NORD12: TuiColour = rgb!(0xD08770); // orange

        /// Used for UI elements that are rendering warning states like linter
        /// markers and the highlighting of Git `diff` modifications.
        ///
        /// In the context of syntax highlighting it is used to override the
        /// highlighting of syntax elements that are detected as warnings as
        /// well as escape characters and within regular expressions.
        pub const NORD13: TuiColour = rgb!(0xEBCB8B); // yellow

        /// Used for UI elements that are rendering success states and
        /// visualizations and the highlighting of Git `diff` additions.
        ///
        /// In the context of syntax highlighting it is used as main color for
        /// strings of any type like double/single quoted or interpolated.
        pub const NORD14: TuiColour = rgb!(0xA3BE8C); // green

        /// Rarely used for UI elements, but it may indicate a more uncommon
        /// functionality.
        ///
        /// In the context of syntax highlighting it is used as main color for
        /// numbers of any type like integers and floating point numbers.
        pub const NORD15: TuiColour = rgb!(0xB48EAD); // purple
    }

    use aurora::*;

    declare_palette! { Aurora = {
        // Data: ,
        // Inst: ,
        // Name: ,
        // Addr: ,
        // Pc: ,
        // Num: ,

        // DataT: ,
        // AddrT: ,
        // NumT: ,

        // Breakpoint: ,
        // Watchpoint: ,
        // BWHighlight: ,

        // Privilege: ,
        // Priority: ,
        // n_bit: ,
        // z_bit: ,
        // p_bit: ,

        MemRegHighlight: NORD13,
        RegHighlight: NORD13,

        // ConsoleIn: ,
        // ConsoleOut: ,
        // ConsoleHelp: ,
        // ConsoleRequest: ,
        // InvalidInput: ,

        // Run: ,
        // Pause: ,
        // Error: ,
        // Success: ,
        // Halted: ,
        // Depth: ,

        // Highlight: ,

        // Focus: ,
        // Reset: ,
        // LoadText: ,
        // LoadNoChanges: ,
        // LoadPendingChanges: ,

        // Title: ,
        // Border: ,
        // Help: ,

        // Modeline: ,
        // StepButtons: ,
        // LoadB: ,
        // mDefault: ,

        // CallStackSupervisorMode: ,
        // CallStackUserMode: ,

        // InProgress: ,
        // Disabled: ,
    }}
}

lazy_static::lazy_static! {
    pub static ref PALETTE: CurrentPalette = {
        if let Some(palette) = std::env::var_os(COLOUR_PALETTE_ENV_VAR) {
            match palette.to_str().unwrap() {
                "nord-polar-night" => nord::PolarNight.into(),
                "nord-snow-storm" => nord::SnowStorm.into(),
                "nord-frost" => nord::Frost.into(),
                "nord" | "nord-aurora" => nord::Aurora.into(),
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
