//! Saving and loading designs as RON.

use rfd::FileDialog;

use super::{CANCELLED, Outcome};
use crate::design::Design;
use crate::editor::Editor;
use crate::fields;

pub(super) fn save(design: &Design) -> Outcome {
    let Some(path) = design_dialog().set_file_name("mould.ron").save_file() else {
        return Ok(CANCELLED.into());
    };
    let text = ron::ser::to_string_pretty(design, ron::ser::PrettyConfig::default())
        .map_err(|error| error.to_string())?;
    std::fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(format!("Saved {}", path.display()))
}

pub(super) fn load(editor: &mut Editor) -> Outcome {
    let Some(path) = design_dialog().pick_file() else {
        return Ok(CANCELLED.into());
    };
    let text = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let mut design: Design = ron::from_str(&text).map_err(|error| error.to_string())?;

    // A file is untrusted: bring every value into its slider's range.
    fields::sanitize(&mut design);
    editor.design = design;
    Ok(format!("Loaded {}", path.display()))
}

fn design_dialog() -> FileDialog {
    FileDialog::new().add_filter("Mould design", &["ron"])
}
