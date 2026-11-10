//! The browser: every file of a bundle becomes a download; loading opens the browser's file
//! picker, whose answer arrives asynchronously and is collected by [`take_picked`].

use std::cell::RefCell;

use wasm_bindgen::JsCast;

use super::{Bundle, Picked};
use crate::files::Outcome;

thread_local! {
    /// A file the picker delivered, waiting for the next frame to collect it.
    static PICKED: RefCell<Option<(String, Vec<u8>)>> = const { RefCell::new(None) };
}

/// Download every file of `bundle`.
pub fn offer(bundle: Bundle) -> Outcome {
    for (suffix, contents) in &bundle.files {
        let name = format!("{}{suffix}.{}", bundle.name, bundle.extension);
        download(&name, contents)
            .map_err(|error| format!("download of {name} failed: {error:?}"))?;
    }
    Ok(format!("{}.{} (downloads)", bundle.name, bundle.extension))
}

/// Open the browser's file picker; the chosen file arrives later via [`take_picked`].
pub fn pick(kind: &'static str, extension: &'static str) -> Result<Picked, String> {
    wasm_bindgen_futures::spawn_local(async move {
        let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter(kind, &[extension])
            .pick_file()
            .await
        else {
            return;
        };
        let contents = file.read().await;
        PICKED.with(|picked| *picked.borrow_mut() = Some((file.file_name(), contents)));
    });
    Ok(Picked::Later)
}

/// The file the picker delivered since the last call, if any.
pub fn take_picked() -> Option<(String, Vec<u8>)> {
    PICKED.with(|picked| picked.borrow_mut().take())
}

/// Hand `contents` to the browser as a download called `name`: a blob behind a temporary
/// link that is clicked once.
fn download(name: &str, contents: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    let bytes = js_sys::Uint8Array::from(contents);
    let parts = js_sys::Array::of1(&bytes.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or("no document")?;
    let link: web_sys::HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    link.set_href(&url);
    link.set_download(name);
    link.click();

    web_sys::Url::revoke_object_url(&url)
}
