//! A place for strings used in the TUI.

pub type StringId = usize;

macro_rules! register_strings {
    ( $table:ident <- {
        $($nom:ident => (
            $regular:literal $(,)?
            $(unicode: $unicode:literal)? $(,)?
        ) $(,)?)*
    }) => {
        register_strings! { munch:begin $($nom)* }

        static $table: &[(&'static str, &'static str)] = &[$({
            let reg = $regular;

            #[allow(unused)]
            let uni = $regular;
            $(let uni = $unicode;)?

            (reg, uni)
        },)*];
    };

    ( munch:begin
        $nom1:ident
        $($rest:tt)*
    ) => {
        pub const $nom1: StringId = 0;
        register_strings!{ munch:continue $nom1 $($rest)* }
    };

    ( munch:continue
        $nom1:ident
    ) => { };

    ( munch:continue
        $nom1:ident
        $nom2:ident
        $($rest:tt)*
    ) => {
        pub const $nom2: StringId = $nom1 + 1;
        register_strings!{ munch:continue $nom2 $($rest)* }
    };

    ( munch:continue
        $nom1:ident
        $nom2:ident
    ) => {
        #[allow(non_upper_case_globals)]
        pub const $nom2: StringId = $nom1 + 1;
    };
}

register_strings! { STR_TABLE <- {
    PeripheralsTab => ("FOO"),
}}

#[inline]
pub fn get_string(id: StringId) -> &'static str {
    if *PLATFORM_SUPPORTS_UNICODE {
        STR_TABLE[id].1
    } else {
        STR_TABLE[id].0
    }
}

use crate::env::{UNICODE_DISABLE_ENV_VAR, UNICODE_ENABLE_ENV_VAR};

// It's fine that we use env vars for the wasm impl; wasi supports env vars
// (which we can manually pass in when running in the browser).
lazy_static::lazy_static! {
    static ref PLATFORM_SUPPORTS_UNICODE: bool = {
        // let default = specialize! { [expr]
        //     desktop => {{
        //         match std::env::consts::OS {
        //             "linux" | "macos" => true,
        //             "windows" => false,
        //             _ => true,
        //         }
        //     }}

        //     web => {{ true }}
        // };

        specialize! { [var: default]
            desktop => {
                match std::env::consts::OS {
                    "linux" | "macos" => true,
                    "windows" => false,
                    _ => true,
                }
            }

            web => { true }
        }

        let manually_enabled = std::env::var_os(UNICODE_ENABLE_ENV_VAR)
            .is_some();

        let manually_disabled = std::env::var_os(UNICODE_DISABLE_ENV_VAR)
            .is_some();

        (default || manually_enabled) && !manually_disabled
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! s { ($nom:ident) => { $crate::strings::get_string($nom) }; }
