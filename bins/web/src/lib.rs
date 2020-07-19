//! TODO!

use lc3_tui::{DynTui, ProgramSource};
use lc3_tui::layout;
use lc3_application_support::init::{BlackBox, SimDevice};

use console_error_panic_hook::set_once as set_panic_hook;
use log::Level;
use js_sys::Promise;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    DataTransferItem,
    DataTransferItemList,
    Document,
    DragEvent,
    Element,
    Event,
    File,
    HtmlElement,
    Url,
};
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
// browser window (probably not). (Update: there's webkitGetAsEntry (here:
// https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem/webkitGetAsEntry)
// but that's non-standard)
//
// In any case, this is a ways off and I'd personally be okay saying that
// browser support is limited to single files only.
//
// # Other considerations
// Rather than updating the URL we could go and change the source in the Tui
// instance (this might need us to stick it in a Mutex of some kind but that's
// fine).

async fn get_data_transfer_item_as_string(dti: &DataTransferItem) -> Result<String, ()> {
    let mut buf = String::new();
    let mut c = None;

    if dti.kind() != "string" {
        return Err(());
    }

    JsFuture::from(Promise::new(&mut |res, _rej| {
        let buf_ref: &'static mut String = unsafe { core::mem::transmute(&mut buf) };

        c = Some(Closure::wrap(Box::new(move |s: String| {
            buf_ref.push_str(s.as_str());

            res.call0(&JsValue::NULL).unwrap();
        }) as Box<dyn FnMut(_)>));

        dti.get_as_string(Some(c.as_ref().unwrap().as_ref().unchecked_ref())).unwrap();
    })).await;

    Ok(buf)
}

// `dropped` indicates whether we should try to get the data out of a "string"
// item (this is not available before drop).
async fn log_data_transfer_items_list_inner(dtl: DataTransferItemList, dropped: bool) {
    let mut s = format!("DT List ({} items):\n", dtl.length());

    for (idx, dt) in (0..dtl.length()).filter_map(|idx| dtl.get(idx).map(|d| (idx, d))) {
        s.push_str(format!(" - {:2}: ({}, {}) → `{}`\n",
            idx,
            dt.kind(),
            dt.type_(),
            match dt.kind().as_str() {
                "file" => {
                    dt.get_as_file()
                        .unwrap()
                        .map(|f| format!("file name: {}", f.name()))
                        .unwrap_or("<failed to get file>".to_string())
                },

                // "string" => {
                //     let mut buf = String::new();
                //     let mut c = None;
                //     JsFuture::from(Promise::new(&mut |res, _rej| {
                //         let buf_ref: &'static mut String = unsafe { core::mem::transmute(&mut buf) };
                //         // let c = Closure::once(move |s: String| {
                //         //     buf_ref.push_str(s.as_str());

                //         //     res.call0(&JsValue::NULL)
                //         // });

                //         c = Some(Closure::wrap(Box::new(move |s: String| {
                //             buf_ref.push_str(s.as_str());

                //             res.call0(&JsValue::NULL).unwrap();
                //         }) as Box<dyn FnMut(_)>));

                //         dt.get_as_string(Some(c.as_ref().unwrap().as_ref().unchecked_ref())).unwrap();
                //         // c.forget();
                //     })).await;

                //     buf
                // },

                "string" => if dropped {
                    get_data_transfer_item_as_string(&dt).await.unwrap()
                } else {
                    String::from("<string>")
                },

                _ => "<invalid kind>".to_string(),
            }
        ).as_ref())
    }

    log::debug!("{}", s)
}

pub fn log_data_transfer_items_list(dtl: &DataTransferItemList, dropped: bool) {
    spawn_local(log_data_transfer_items_list_inner(dtl.clone(), dropped))
}

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
    //  - this is weird bc Firefox gives us a `text/x-moz-url` MIME type but
    //    Chrome gives us nothing..
    // TODO: figure out why strings read as 4 items..
    // TODO: figure out why URLs read as 8 items..
    enum DragItemType {
        AsmFile(Option<File>),
        MemFile(Option<File>),
        String(DataTransferItem), // Can represent a URL or a literal string
    }

    impl DragItemType {
        fn from(item: &DataTransferItem) -> Result<Self, (String, String)> {
            // Note: we have to match on type = "" because Firefox doesn't give
            // us the type until the drop event (Chrome gives it to us right
            // away, along with the file name).
            match (&*item.kind(), &*item.type_()) {
                ("file", "") |
                ("file", "text/plain") |
                ("file", "text/lc3-asm") => {
                    Ok(DragItemType::AsmFile(item.get_as_file().unwrap()))
                },

                ("file", "application/octet-stream") |
                ("file", "application/lc3-bin") => {
                    Ok(DragItemType::MemFile(item.get_as_file().unwrap()))
                },

                ("string", "") |
                ("string", "text/plain") |
                ("string", "text/x-moz-url") |
                ("string", "text/lc3-asm") |
                ("string", "application/octet-stream") |
                ("string", "application/lc3-bin") => {
                    Ok(DragItemType::String(item.clone()))
                },

                (kind, ty) => Err((kind.to_string(), ty.to_string())),
            }
        }

        // Returns a `DragItemType` on success or the (kind, type) pair of the
        // last item in the list on failure (if present).
        //
        // TODO: on multiple valid items this will only take the first and will
        // silently ignore the rest...
        fn from_list(list: &DataTransferItemList) -> Result<Self, Option<(String, String)>> {
            let mut err = None;

            for item in (0..list.length()).filter_map(|idx| list.get(idx)) {
                match Self::from(&item) {
                    Ok(i) => return Ok(i),
                    Err(e) => err = Some(e),
                }
            }

            Err(err)
        }
    }

    enum DragItemResult {
        AsmFile(File),
        MemFile(File),

        AsmUrl(String),
        MemUrl(String),

        AsmLiteral(String),
        // MemLiteral(String), // TODO: does this even make sense? Can you repr memory dumps as a Unicode String without exploding? I think no.
    }

    // enum DragErr {
    //     NoItems,
    //     UnsupportedFormat { kind: String, ty: String },
    //     MultipleItems(u8),
    // }

    enum DragResErr {
        CouldNotGetFile,
        CouldNotGetString,
    }

    impl DragItemResult {
        async fn from(dt: DragItemType) -> Result<Self, DragResErr> {
            use DragItemResult::*;
            use DragResErr::*;
            match dt {
                DragItemType::AsmFile(f) => AsmFile(f.ok_or(CouldNotGetFile)?),
                DragItemType::MemFile(f) => MemFile(f.ok_or(CouldNotGetFile)?),

                DragItemResult::String(i) => match (
                    i.type_(),
                    get_data_transfer_item_as_string(&i).await().ok_or(CouldNotGetString)?,
                ) {
                    // MemLiteral
                    // TODO: replace this with whatever magic memory dumps end
                    // up using.
                    // ("", c) if c.starts_with(".UTP")
                    // ("application/octet-stream", c) |
                    // ("application/lc3-bin", c) => MemLiteral(c),

                    // AsmLiteral
                    ("", c) |
                    ("text/lc3-asm", c) => AsmLiteral(c),

                    // URL
                    ("text/x-moz-url", c) if c.ends_with("mem") => MemUrl(c),
                    ("text/x-moz-url", c) => AsmUrl(c),

                    // URL or AsmLiteral?
                    // Note: this is probably too simplistic a check for URLs.
                    ("text/plain", c) if c.starts_with("http://") || c.starts_with("https://") =>
                        if c.ends_with("mem") {
                            MemUrl(c)
                        } else {
                            AsmUrl(c)
                        }

                    ("text/plain", c) => AsmLiteral(c),

                    (_, _) => unreachable!(),
                }
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
        log_data_transfer_items_list(items, false);

        let msg = match DragItemType::from_list(&items) {
            Ok(item) => match item {
                DragItemType::AsmFile(f) => Ok(
                    format!("Drop To Load {}!", f
                        .map(|f| f.name())
                        .unwrap_or("File".to_string()))
                ),
                DragItemType::MemFile(_) => Err(
                    String::from("❌ Memory Dumps are not yet supported. ❌")
                ),
                DragItemType::String(_) => Ok(
                    String::from("Drop To Load!")
                ),
            },
            Err(None) => Err("Can't drop 0 items. 😕".to_string()),
            Err(Some((kind, ty))) => Err(
                format!("❌ Unsupported format: `{}:{}`. ❌", kind, ty)
            )
        };

        // TODO: figure out why the effects don't take..
        let (effect, msg) = match msg {
            Ok(m) => ("copy", m),
            Err(m) => ("none", m),
        };

        dt.set_drop_effect(effect);
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

        let items = dv.data_transfer().unwrap().items();

        log::debug!("drop");
        log_data_transfer_items_list(items, true);

        macro_rules! failure {
            ($($t:tt)*) => {{
                div.set_inner_html(format!("<strong>🚨 {} 🚨</strong>",
                    format!($(t)*)
                ).as_ref())

                let div = div.clone();
                spawn_local(async {
                    let window = web_sys::window();
                    lc3_application_support::event_loop::timeout(&window, 3_000).await;
                    div.set_class_name("");
                    div.set_inner_html("");
                })

                return;
            }};
        }

        div.set_class_name("dropped");
        let item = match DragItemType::from_list(&items) {
            Ok(item) => item,
            Err(None) => failure!("Can't drop 0 items 😕.".to_string()),
            Err(Some((kind, ty))) => failure!("Unsupported format: `{}:{}`.", kind, ty),
            // Err(err) => {
            //     let msg = match err {
            //         None => "Can't drop 0 items. 😕".to_string(),
            //         Some((kind, ty)) => format!("❌ Unsupported format: `{}:{}`. ❌", kind, ty),
            //     };

            //     div.set_inner_html(format!("<strong>{}</strong>", msg).as_ref());
            //     return;
            // }
        }

        use DragErr::*
        let item = match spawn_local(DragItemResult::from(&item)) {
            Ok(item) = item,
            Err(CouldNotGetFile) => failure!("Could not load file."),
            Err(CouldNotGetString) => failure!("Could not get string."),

            // Err(e) => {
            //     let m = match e {
            //        CouldNotGetFile => "Could not load file.",
            //        CouldNotGetString => "Could not get string.",
            //     };

            //     div.set_inner_html(format!("<strong>{}</strong>", msg).as_ref());
            //     return;
            // }
        };

        use DragItemResult::*;
        let msg = match item {
            AsmFile(f) => format!("📁 Loading `{}` as an assembly file... ⌛", f.name()),
            MemFile(f) => format!("📁 Loading `{}` as a memory dump... ⌛", f.name())
            AsmUrl(u) => format!("🌐 Loading `{}` as a link to an assembly file... ⌛", u),
            MemUrl(u) => format!("🌐 Loading `{}` as a link to a memory dump... ⌛", u),
            AsmLiteral(_) => format!("📜 Loading assembly string... ⌛"),
        };
        div.set_inner_html(format!("<h2>{}</h2>", msg).as_ref());

        spawn_local(async {
            let src = match item {
                AsmFile()
            }
        })

        let src = match item {
            AsmFile(f) =>
        }
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
