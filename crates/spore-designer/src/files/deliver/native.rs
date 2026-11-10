//! The desktop: a save dialog for the main file (siblings land next to it), an open dialog
//! for loading.

use rfd::FileDialog;

use super::{Bundle, Picked};
use crate::files::Outcome;

/// Ask where to save `bundle`'s main file; write it, and its siblings beside it.
pub fn offer(bundle: Bundle) -> Outcome {
    let Some(path) = FileDialog::new()
        .add_filter(bundle.kind, &[bundle.extension])
        .set_file_name(format!("{}.{}", bundle.name, bundle.extension))
        .save_file()
    else {
        return Ok(crate::files::CANCELLED.into());
    };

    let stem = path
        .file_stem()
        .map_or_else(Default::default, |stem| stem.to_string_lossy().into_owned());
    for (suffix, contents) in &bundle.files {
        let file = path.with_file_name(format!("{stem}{suffix}.{}", bundle.extension));
        std::fs::write(&file, contents).map_err(|error| error.to_string())?;
    }
    Ok(path.display().to_string())
}

/// Ask for a file to open, and read it.
pub fn pick(kind: &'static str, extension: &'static str) -> Result<Picked, String> {
    let Some(path) = FileDialog::new().add_filter(kind, &[extension]).pick_file() else {
        return Ok(Picked::Cancelled);
    };
    let contents = std::fs::read(&path).map_err(|error| error.to_string())?;
    Ok(Picked::File(path.display().to_string(), contents))
}

/// Files arrive straight from [`pick`] here; nothing ever waits.
pub fn take_picked() -> Option<(String, Vec<u8>)> {
    None
}
