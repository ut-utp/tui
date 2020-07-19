//! TODO!

use lc3_tui::{DynTui, ProgramSource};
use lc3_tui::layout;
use lc3_application_support::init::{BlackBox, SimDevice};

use console_error_panic_hook::set_once as set_panic_hook;
use log::Level;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DataTransferItem, Document, DragEvent, Element, Event, File, HtmlElement, Url};
use xterm_js_sys::xterm::{LogLevel, Terminal, TerminalOptions, Theme};

use std::time::Duration;
use std::str::FromStr;

// Note: right now we only accept asm files for drag 'n drop. Eventually we
// want to support:
//  - Memory Dump Files
//  - Multiple Assembly Files
//
// # Memory Dump Files
// This isn't complicated but is blocked on us picking/implementing some kind
// of magic that we're going to use for such files (i.e. the thing we'll also
// use with binfmt).
//
// When we do this we should:
//  - update `ondragenter` to detect the file type and say what kind of thing
//    we're going to be loading (asm or mem) if the user finishes the drop.
//     * this is also blocked on us being able to work with memory dump
//       immediates
//  - update `ondrop` to do detect the file kind and use update the URL
//    accordingly
//
// # Multiple Assembly Files
// This refers to future plans when we have the ability to reference things in
// other assembly files (`.extern` support) and _is_ complicated.
//
// The problem becomes how do users give us all the files in a project? Dragging
// the full file list seems good except that the way this will probably work is
// that we ask for a 'root' file for the project that goes and tells us what
// files are also part of the project; if we are given a list of files we have
// no way to know which is the 'root'.
//
// I think the least bad solution is to only enable this feature when we're
// granted access to the user file system (iiuc there is a relatively modern
// Web API that allows us this access if the user permits it). Users can just
// drop the root file and we'll, as usual, figure out what other files are
// involved.
//
// This is made a little complicated by the fact that the 'detected what other
// files are used' bit is usually done by the assembler and that I don't know
// if we can actually get the file path out of a file that's dropped onto a
// browser window (probably not).
//
// In any case, this is a ways off and I'd personally be okay saying that
// browser support is limited to single files only.
//
// # Other considerations
// Rather than updating the URL we could go and change the source in the Tui
// instance (this might need us to stick it in a Mutex of some kind but that's
// fine).

// Note: this leaks which we'll say is okay since these closures are used for
// the entire life of the program anyways.
pub fn register_drag_hooks(doc: &Document, drop_div: Element) -> Result<(), JsValue> {
    let drop_div = drop_div.dyn_into::<HtmlElement>()?;

    macro_rules! reg {
        ($elem:ident :: $event:ident($ev_var:ident: $ev_ty:ty) => $handler:block) => {
            {
                let handler = Closure::wrap(
                    Box::new(move |$ev_var: $ev_ty| $handler) as Box<dyn FnMut(_)>
                );

                $elem.$event(Some(handler.as_ref().unchecked_ref()));
                handler.forget();
            }
        };
    }

    // TODO: extend to support URLs!
    // TODO: figure out why strings read as 4 items..
    // TODO: figure out why URLs read as 8 items..
    enum DragItemType {
        AsmFile(Option<File>),
        AsmString,
        MemFile(Option<File>),
        MemString,
    }
    use DragItemType::*;

    impl DragItemType {
        fn from(item: &DataTransferItem) -> Result<Self, (String, String)> {
            match (&*item.kind(), &*item.type_()) {
                ("file", "") |
                ("file", "text/plain") |
                ("file", "text/lc3-asm") => {
                    Ok(AsmFile(item.get_as_file().unwrap()))
                },

                ("string", "") |
                ("string", "text/plain") |
                ("string", "text/lc3-asm") => Ok(AsmString),

                ("file", "") |
                ("file", "application/octet-stream") |
                ("file", "application/lc3-bin") => {
                    Ok(MemFile(item.get_as_file().unwrap()))
                },

                ("string", "") |
                ("string", "application/octet-stream") |
                ("string", "application/lc3-bin") => Ok(MemString),

                (kind, ty) => Err((kind.to_string(), ty.to_string())),
            }
        }
    }

    // We need this event handler for
    // [dumb reasons](https://stackoverflow.com/a/32084240):
    reg!(doc :: set_ondragover(ev: DragEvent) => {
        AsRef::<Event>::as_ref(&ev).prevent_default();
    });

    let div = drop_div.clone();
    reg!(doc :: set_ondragenter(ev: DragEvent) => {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        let dt = if let Some(dt) = ev.data_transfer() {
            dt
        } else {
            log::error!("Failed to get a `DataTransfer` from the `DragEvent`!");
            return;
        };

        let items = dt.items();

        let m;
        let msg = match items.length() {
            0 => Err("Can't drop 0 items. 😕"),
            1 => {
                match DragItemType::from(&items.get(0).expect("one item")) {
                    Ok(f) => match f {
                        AsmFile(f) => m = Ok(format!("Drop To Load `{}`!", f.map(|f| f.name()).unwrap_or("<file>".to_string()))),
                        AsmString => m = Ok(String::from("Drop To Load Assembly String!")),
                        MemFile(_) | MemString => m = Err(String::from("❌ Memory Dumps are not yet supported. ❌")),
                    }
                    Err((kind, ty)) => {
                        m = Err(format!("❌ Unsupported format: `{}:{}`. ❌", kind, ty));
                    }
                }

                // m.as_deref().as_deref_err()
                m.as_ref().map(|s| s.as_ref()).map_err(|e| e.as_ref())
            }
            _ => Err("❌ Only dropping 1 item is currently supported. ❌"),
        };

        // TODO: figure out why the effects don't take..
        let msg = match msg {
            Ok(m) => {
                dt.set_drop_effect("copy");
                m
            },
            Err(m) => {
                dt.set_drop_effect("none");
                m
            }
        };

        div.set_class_name("hover");
        div.set_inner_html(format!("<strong>{}</strong>", msg).as_ref())
    });

    // leave, not exit.
    let div = drop_div.clone();
    reg!(doc :: set_ondragleave(ev: DragEvent) => {
        AsRef::<Event>::as_ref(&ev).prevent_default();

        div.set_class_name("");
        div.set_inner_html("");
    });

    let div = drop_div.clone();
    let document = doc.clone();
    reg!(doc :: set_ondrop(dv: DragEvent) => {
        let ev = AsRef::<Event>::as_ref(&dv);
        ev.prevent_default();
        ev.stop_propagation();

        log::debug!("drop");
        div.set_class_name("dropped");

        let items = dv.data_transfer().unwrap().items();
        let len = items.length();

        if len != 1 {
            log::error!("🚨 Attempted to drop not just 1 item! ({} items)", len);
        } else {
            let item = items.get(0).expect("one item");

            document.location()
        }

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
