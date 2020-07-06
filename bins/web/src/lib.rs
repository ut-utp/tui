//! TODO!

use lc3_tui::DynTui;
use lc3_tui::layout;
use lc3_application_support::init::{BlackBox, SimDevice};

use console_error_panic_hook::set_once as set_panic_hook;
use log::Level;
use wasm_bindgen::prelude::*;
use xterm_js_sys::xterm::{LogLevel, Terminal, TerminalOptions, Theme};

use std::time::Duration;

#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    set_panic_hook();
    console_log::init_with_level(Level::Trace).expect("error initializing log");

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let terminal_div = document
        .get_element_by_id("terminal")
        .expect("should have a terminal div");

    let term = Terminal::new(Some(
        TerminalOptions::default()
            .with_log_level(LogLevel::Debug)
            .with_theme(Theme::nord())
            .with_font_family("'Fira Mono', monospace")
            .with_font_size(11.0),
    ));

    term.open(terminal_div);

    let mut b = BlackBox::new();
    let mut tui = DynTui::new_boxed_from_init::<SimDevice>(&mut b);

    let no_extra_tabs = Vec::new();
    let layout = layout::layout(
        Some("UTP LC-3 Simulator (running locally)"),
        no_extra_tabs,
    );

    tui.set_use_os(true); // TODO: expose as option.
    tui.set_update_period(Duration::from_millis(1000 / 60)); // TODO: expose as option.
    tui.run_with_xtermjs(Some(layout), &term).await.unwrap();

    log::info!("Goodbye! 👋");
    Ok(())
}
