//! TODO!

use lc3_tui::{DynTui, ProgramSource};
use lc3_tui::layout;
use lc3_application_support::init::{BlackBox, SimDevice};

use console_error_panic_hook::set_once as set_panic_hook;
use log::Level;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Document, DragEvent, Event, HtmlElement, Url};
use xterm_js_sys::xterm::{LogLevel, Terminal, TerminalOptions, Theme};

use std::time::Duration;
use std::str::FromStr;

// Note: this leaks which we'll say is okay since these closures are used for
// the entire life of the program anyways.
pub fn register_drag_hooks(document: &Document) -> Result<(), JsValue> {
    let drop_div = document
        .get_element_by_id("drop")
        .expect("should have a drop div")
        .dyn_into::<HtmlElement>()?;

    let enter = Closure::wrap(Box::new({ let div = drop_div.clone(); move |ev: DragEvent| {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        // TODO: get the file name and make a message here!

        log::error!("enter");

        div.set_class_name("hover");
    }}) as Box<dyn FnMut(_)>);

    let exit = Closure::wrap(Box::new({ let div = drop_div.clone(); move |ev: DragEvent| {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        log::error!("exit");
        div.set_class_name("");
    }}) as Box<dyn FnMut(_)>);

    let drop = Closure::wrap(Box::new({ let div = drop_div.clone(); move |ev: DragEvent| {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        log::error!("drop");
        div.set_class_name("dropped");

        // TODO: load the file, update the URL, reload the page.
    }}) as Box<dyn FnMut(_)>);

    drop_div.set_ondragenter(Some(enter.as_ref().unchecked_ref()));
    drop_div.set_ondragexit(Some(exit.as_ref().unchecked_ref()));
    drop_div.set_ondrop(Some(drop.as_ref().unchecked_ref()));

    enter.forget();
    exit.forget();
    drop.forget();

    Ok(())
}

#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    set_panic_hook();
    console_log::init_with_level(Level::Debug).expect("error initializing log");

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let terminal_div = document
        .get_element_by_id("terminal")
        .expect("should have a terminal div");

    let term = Terminal::new(Some(
        TerminalOptions::default()
            .with_log_level(LogLevel::Off)
            .with_theme(Theme::nord())
            .with_font_family("'Fira Mono', monospace")
            .with_font_size(11.0),
    ));

    term.open(terminal_div);
    term.resize(200, 45);

    register_drag_hooks(&document)?;

    let mut b = BlackBox::new();
    let mut tui = DynTui::new_boxed_from_init::<SimDevice>(&mut b);

    let no_extra_tabs = Vec::new();
    let layout = layout::layout(
        Some("UTP LC-3 Simulator (running locally)"),
        no_extra_tabs,
    );

    let src = document.location()
        .and_then(|l| l.href().ok())
        .and_then(|h| Url::new(h.as_ref()).ok())
        .and_then(|u| u.search_params().get("src"));

    if let Some(src) = src {
        let mut src = ProgramSource::from_str(&src)?;
        src.normalize().await.unwrap(); // TODO: don't unwrap here?

        tui.set_program_source(src);
    }

    tui.set_use_os(true); // TODO: expose as option.
    tui.set_update_period(Duration::from_millis(1000 / 60)); // TODO: expose as option.

    tui.run_with_xtermjs(Some(layout), &term).await.unwrap();

    log::info!("Goodbye! 👋");
    Ok(())
}
