//! File operations: save and load designs as RON, export stages (and their lighting maps)
//! as PNG.
//!
//! Each returns the message shown under the panel's file buttons.
//!
//! - `files/design_file.rs`: save and load.
//! - `files/export.rs`: PNG exports with albedo, height, normal and specular maps.

mod design_file;
mod export;

use spore_core::Rgba;

use crate::design::View;
use crate::editor::{Action, Editor};
use design_file::{load, save};
use export::{export_sheet, export_stage};

/// Run `action` on the editor; `frames` are the stages currently shown (for exports).
pub fn run(action: Action, editor: &mut Editor, frames: &[Rgba]) -> String {
    let result = match action {
        Action::ExportStage | Action::ExportSheet if editor.design.view == View::Field => {
            Ok("Export works in the colony and patch views".into())
        }
        Action::Save => save(&editor.design),
        Action::Load => load(editor),
        Action::ExportStage => export_stage(
            frames,
            editor.stage_index(frames.len().max(1)),
            editor.design.relief,
        ),
        Action::ExportSheet => export_sheet(frames, editor.design.relief),
    };
    result.unwrap_or_else(|error| format!("Error: {error}"))
}

type Outcome = Result<String, String>;

const CANCELLED: &str = "Cancelled";

#[cfg(test)]
mod tests;
