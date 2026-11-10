//! File operations: save and load designs as RON, export stages as PNG.
//!
//! Each returns the message shown under the panel's file buttons.

use std::path::{Path, PathBuf};

use rfd::FileDialog;
use spore_core::{Rgba, sheet};

use crate::design::{Design, View};
use crate::editor::{Action, Editor};
use crate::fields;

/// Run `action` on the editor; `frames` are the stages currently shown (for exports).
pub fn run(action: Action, editor: &mut Editor, frames: &[Rgba]) -> String {
    let result = match action {
        Action::ExportStage | Action::ExportSheet if editor.design.view == View::Field => {
            Ok("Export works in the colony and patch views".into())
        }
        Action::Save => save(&editor.design),
        Action::Load => load(editor),
        Action::ExportStage => export_stage(frames, editor.stage_index(frames.len().max(1))),
        Action::ExportSheet => export_sheet(frames),
    };
    result.unwrap_or_else(|error| format!("Error: {error}"))
}

type Outcome = Result<String, String>;

const CANCELLED: &str = "Cancelled";

// ------------------------------------------------------------------------------------------
// Designs

fn save(design: &Design) -> Outcome {
    let Some(path) = design_dialog().set_file_name("mould.ron").save_file() else {
        return Ok(CANCELLED.into());
    };
    let text = ron::ser::to_string_pretty(design, ron::ser::PrettyConfig::default())
        .map_err(|error| error.to_string())?;
    std::fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(format!("Saved {}", path.display()))
}

fn load(editor: &mut Editor) -> Outcome {
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

// ------------------------------------------------------------------------------------------
// Images

/// The shown stage as one PNG.
fn export_stage(frames: &[Rgba], stage: usize) -> Outcome {
    let Some(image) = frames.get(stage) else {
        return Ok("Nothing to export".into());
    };
    let Some(path) = png_dialog(&format!("mould-stage{}.png", stage + 1)) else {
        return Ok(CANCELLED.into());
    };
    save_png(&path, &image.to_rgba8(), image.side, image.side)
}

/// All stages side by side in one row, as one PNG.
fn export_sheet(frames: &[Rgba]) -> Outcome {
    let Some(path) = png_dialog("mould-sheet.png") else {
        return Ok(CANCELLED.into());
    };
    let side = frames.first().map_or(0, |frame| frame.side);
    let count = frames.len();
    save_png(&path, &sheet(frames, count), side * count, side)
}

fn png_dialog(file_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("PNG image", &["png"])
        .set_file_name(file_name)
        .save_file()
}

fn save_png(path: &Path, rgba: &[u8], width: usize, height: usize) -> Outcome {
    image::save_buffer(
        path,
        rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|error| error.to_string())?;
    Ok(format!("Exported {}", path.display()))
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use spore_core::{Field, Patch, presets};

    #[test]
    fn design_file_and_png_roundtrip() {
        let design = Design {
            seed: 42,
            size: 32,
            stages: 3,
            mold: presets::annihilation(),
            view: View::Patch,
            patch: Patch::default(),
            field: Field::default(),
        };
        let text = ron::ser::to_string_pretty(&design, ron::ser::PrettyConfig::default()).unwrap();
        assert!(ron::from_str::<Design>(&text).unwrap() == design);

        let frames = design.frames();
        let side = frames[0].side;
        assert!(side >= 32, "frames are at least one cell");

        let path = std::env::temp_dir().join(format!("spore-sheet-{}.png", std::process::id()));
        save_png(&path, &sheet(&frames, 3), side * 3, side).unwrap();
        let back = image::open(&path).unwrap().into_rgba8();
        std::fs::remove_file(&path).ok();
        assert_eq!(back.dimensions(), (side as u32 * 3, side as u32));
        assert_eq!(back.as_raw(), &sheet(&frames, 3));
    }
}
