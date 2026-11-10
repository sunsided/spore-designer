//! File operations: save and load designs as RON, export stages (and their lighting maps)
//! as PNG.
//!
//! Each returns the message shown under the panel's file buttons.

use std::path::{Path, PathBuf};

use image::{ColorType, ImageBuffer, Luma};
use rfd::FileDialog;
use spore_core::relief::{encode_normals, gloss_map, height_map, max_height, normal_map};
use spore_core::{Rgba, grid, sheet};

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

/// The shown stage as a PNG, plus its maps.
fn export_stage(frames: &[Rgba], stage: usize, relief: f32) -> Outcome {
    let Some(image) = frames.get(stage) else {
        return Ok("Nothing to export".into());
    };
    let Some(path) = png_dialog(&format!("mould-stage{}.png", stage + 1)) else {
        return Ok(CANCELLED.into());
    };
    export_with_maps(&path, std::slice::from_ref(image), relief)
}

/// All stages side by side in one row, as a PNG, plus their maps.
fn export_sheet(frames: &[Rgba], relief: f32) -> Outcome {
    if frames.is_empty() {
        return Ok("Nothing to export".into());
    }
    let Some(path) = png_dialog("mould-sheet.png") else {
        return Ok(CANCELLED.into());
    };
    export_with_maps(&path, frames, relief)
}

/// Write `frames` side by side to `path`, and next to it, laid out the same way:
///
/// - `<name>-albedo.png`: colour without baked lighting (RGBA);
/// - `<name>-height.png`: displacement, 16-bit grey, one scale for all frames;
/// - `<name>-normal.png`: tangent-space normals, OpenGL convention (Y up);
/// - `<name>-specular.png`: shininess, grey.
fn export_with_maps(path: &Path, frames: &[Rgba], relief: f32) -> Outcome {
    let side = frames[0].side;
    let count = frames.len();
    let (width, height) = ((side * count) as u32, side as u32);
    let layout = |images: Vec<Vec<u8>>, channels: usize| grid(&images, side, channels, count);

    write_png(path, &sheet(frames, count), width, height, ColorType::Rgba8)?;

    let albedo = frames.iter().map(Rgba::albedo_rgba8).collect();
    write_png(
        &sibling(path, "albedo"),
        &layout(albedo, 4),
        width,
        height,
        ColorType::Rgba8,
    )?;

    let max = frames.iter().map(max_height).fold(1e-3, f32::max);
    let heights: Vec<Vec<u16>> = frames.iter().map(|frame| height_map(frame, max)).collect();
    ImageBuffer::<Luma<u16>, Vec<u16>>::from_raw(width, height, grid(&heights, side, 1, count))
        .expect("grid matches its size")
        .save(sibling(path, "height"))
        .map_err(|error| error.to_string())?;

    let normals = frames
        .iter()
        .map(|frame| encode_normals(&normal_map(frame, relief)))
        .collect();
    write_png(
        &sibling(path, "normal"),
        &layout(normals, 3),
        width,
        height,
        ColorType::Rgb8,
    )?;

    let gloss = frames.iter().map(gloss_map).collect();
    write_png(
        &sibling(path, "specular"),
        &layout(gloss, 1),
        width,
        height,
        ColorType::L8,
    )?;

    Ok(format!(
        "Exported {} with albedo, height, normal and specular maps (white height = {max:.1} reference px)",
        path.display()
    ))
}

/// `dir/name.png` -> `dir/name-<suffix>.png`.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .map_or_else(Default::default, |stem| stem.to_string_lossy());
    path.with_file_name(format!("{stem}-{suffix}.png"))
}

fn png_dialog(file_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("PNG image", &["png"])
        .set_file_name(file_name)
        .save_file()
}

fn write_png(path: &Path, data: &[u8], width: u32, height: u32, color: ColorType) -> Outcome {
    image::save_buffer(path, data, width, height, color).map_err(|error| error.to_string())?;
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
            relief: 1.5,
        };
        let text = ron::ser::to_string_pretty(&design, ron::ser::PrettyConfig::default()).unwrap();
        assert!(ron::from_str::<Design>(&text).unwrap() == design);

        let frames = design.frames();
        let side = frames[0].side;
        assert!(side >= 32, "frames are at least one cell");

        let directory = std::env::temp_dir().join(format!("spore-export-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("sheet.png");
        export_with_maps(&path, &frames, design.relief).unwrap();

        let lit = image::open(&path).unwrap().into_rgba8();
        assert_eq!(lit.dimensions(), (side as u32 * 3, side as u32));
        assert_eq!(lit.as_raw(), &sheet(&frames, 3));
        for (suffix, color) in [
            ("albedo", ColorType::Rgba8),
            ("height", ColorType::L16),
            ("normal", ColorType::Rgb8),
            ("specular", ColorType::L8),
        ] {
            let map = image::open(sibling(&path, suffix)).unwrap();
            assert_eq!(
                (map.width(), map.height(), map.color()),
                (lit.width(), lit.height(), color)
            );
        }
        std::fs::remove_dir_all(&directory).ok();
    }
}
