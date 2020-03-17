//! TODO!

// TODO: forbid
#![warn(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
// TODO: deny
#![warn(
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    missing_docs,
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// macro_rules! specialize {
//     (desktop => { $($d:item)* } web => { $($w:item)* }) => {
//         $(
//             #[cfg(not(target = "wasm32"))]
//             $d
//         )*

//         $(
//             #[cfg(target = "wasm32")]
//             $w
//         )*
//     };
// }

// specialize!{
//     desktop => { extern crate tui_desktop as tui; }
//     web => { extern crate tui_web as tui; }
// }

pub mod debug;
pub mod layout;
pub mod widgets;

// mod tui_lib;
// pub use crate::tui_lib::{DynTui, Tui};

mod tui;
pub use crate::tui::{DynTui, Tui};
