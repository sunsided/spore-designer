//! Saving and loading designs as RON.

use super::Outcome;
use super::deliver::{self, Bundle, Picked};
use crate::design::Design;
use crate::editor::Editor;
use crate::fields;

const KIND: &str = "Mould design";
const EXTENSION: &str = "ron";

pub(super) fn save(design: &Design) -> Outcome {
    let text = ron::ser::to_string_pretty(design, ron::ser::PrettyConfig::default())
        .map_err(|error| error.to_string())?;
    let saved_to = deliver::offer(Bundle {
        name: "mould".into(),
        extension: EXTENSION,
        kind: KIND,
        files: vec![("", text.into_bytes())],
    })?;
    Ok(format!("Saved {saved_to}"))
}

/// Ask for a design file. On the desktop it is loaded right away; in the browser it arrives
/// a few frames later (see [`apply_picked`]).
pub(super) fn load(editor: &mut Editor) -> Outcome {
    match deliver::pick(KIND, EXTENSION)? {
        Picked::File(name, contents) => apply(editor, &name, &contents),
        Picked::Cancelled => Ok(super::CANCELLED.into()),
        Picked::Later => Ok("Choose a design file…".into()),
    }
}

/// Load a design file that arrived since the last frame (browser), if any.
pub(super) fn apply_picked(editor: &mut Editor) -> Option<Outcome> {
    let (name, contents) = deliver::take_picked()?;
    Some(apply(editor, &name, &contents))
}

/// Read a design from `contents` and make it the current one.
fn apply(editor: &mut Editor, name: &str, contents: &[u8]) -> Outcome {
    let text = std::str::from_utf8(contents).map_err(|error| error.to_string())?;
    let mut design: Design = ron::from_str(text).map_err(|error| error.to_string())?;

    // A file is untrusted: bring every value into its slider's range.
    fields::sanitize(&mut design);
    editor.design = design;
    Ok(format!("Loaded {name}"))
}
