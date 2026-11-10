//! Getting files to and from the user, whatever the platform:
//!
//! - `deliver/native.rs`: the desktop's file dialogs and disk;
//! - `deliver/web.rs`: the browser's downloads and file picker.
//!
//! Both offer the same three functions; everything else in `files/` only sees bytes.

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub(super) use native::{offer, pick, take_picked};
#[cfg(target_arch = "wasm32")]
pub(super) use web::{offer, pick, take_picked};

/// Files to hand to the user, as one bundle: a main file and its siblings.
pub(super) struct Bundle {
    /// Suggested name of the main file, without extension (e.g. `mould-sheet`).
    pub name: String,
    pub extension: &'static str,
    /// What the files are, for the save dialog's filter (e.g. "PNG image"). Browsers just
    /// download.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub kind: &'static str,
    /// Name suffix (`""` for the main file, e.g. `"-albedo"` for a sibling) and contents.
    pub files: Vec<(&'static str, Vec<u8>)>,
}

/// What picking a file to open gave.
pub(super) enum Picked {
    /// A file: its name and contents. (Browsers always answer later.)
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    File(String, Vec<u8>),
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Cancelled,
    /// The browser's picker is open; the file arrives later through `take_picked`.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Later,
}
