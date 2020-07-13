//! TODO!

use lc3_tui::{DynTui, ProgramSource};
use lc3_tui::layout;
use lc3_application_support::init::{BlackBox, SimDevice};

use console_error_panic_hook::set_once as set_panic_hook;
use log::Level;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Document, DragEvent, Element, Event, HtmlElement, Url};
use xterm_js_sys::xterm::{LogLevel, Terminal, TerminalOptions, Theme};

use std::time::Duration;
use std::str::FromStr;

// Note: this leaks which we'll say is okay since these closures are used for
// the entire life of the program anyways.
pub fn register_drag_hooks(doc: &Document, drop_div: Element) -> Result<(), JsValue> {
    let drop_div = drop_div.dyn_into::<HtmlElement>()?;

    macro_rules! reg {
        ($elem:ident :: $event:ident($ev_var:ident: $ev_ty:ty) => $handler:block) => {
            {
                let handler = Closure::wrap(
                    Box::new(move |$ev_var| $handler) as Box<dyn FnMut($ev_ty)>
                );

                $elem.$event(Some(handler.as_ref().unchecked_ref()));
                handler.forget();
            }
        };
    }

    // We need this for [dumb reasons](https://stackoverflow.com/a/32084240):
    reg!(doc :: set_ondragover(ev: DragEvent) => {
        AsRef::<Event>::as_ref(&ev).prevent_default();
    });

    let div = drop_div.clone();
    reg!(doc :: set_ondragenter(ev: DragEvent) => {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        // TODO: get the file name and make a message here!
        log::debug!("enter");

        div.set_class_name("hover");
    });

    // leave, not exit.
    let div = drop_div.clone();
    reg!(doc :: set_ondragleave(ev: DragEvent) => {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        log::debug!("exit");
        div.set_class_name("");
    });

    let div = drop_div.clone();
    reg!(doc :: set_ondrop(dv: DragEvent) => {
        let ev = AsRef::<Event>::as_ref(&dv);
        ev.prevent_default();
        ev.stop_propagation();

        log::debug!("drop");
        div.set_class_name("dropped");

        // TODO: loading message, load the file, update the URL, reload the page.
    });

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
    let drop_div = document
        .get_element_by_id("drop")
        .expect("should have a drop div");

    register_drag_hooks(&document, drop_div)?;

    let term = Terminal::new(Some(
        TerminalOptions::default()
            .with_log_level(LogLevel::Off)
            .with_theme(Theme::nord())
            .with_font_family("'Fira Mono', monospace")
            .with_font_size(11.0),
    ));

    term.open(terminal_div);
    term.resize(200, 45);

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
