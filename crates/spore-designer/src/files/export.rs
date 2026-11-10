//! Exporting stages as PNG, with their lighting maps.

use std::path::{Path, PathBuf};

use image::{ColorType, ImageBuffer, Luma};
use rfd::FileDialog;
use spore_core::relief::{encode_normals, gloss_map, height_map, max_height, normal_map};
use spore_core::{Rgba, grid, sheet};

use super::{CANCELLED, Outcome};

/// The shown stage as a PNG, plus its maps.
pub(super) fn export_stage(frames: &[Rgba], stage: usize, relief: f32) -> Outcome {
    let Some(image) = frames.get(stage) else {
        return Ok("Nothing to export".into());
    };
    let Some(path) = png_dialog(&format!("mould-stage{}.png", stage + 1)) else {
        return Ok(CANCELLED.into());
    };
    export_with_maps(&path, std::slice::from_ref(image), relief)
}

/// All stages side by side in one row, as a PNG, plus their maps.
pub(super) fn export_sheet(frames: &[Rgba], relief: f32) -> Outcome {
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
pub(super) fn export_with_maps(path: &Path, frames: &[Rgba], relief: f32) -> Outcome {
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
pub(super) fn sibling(path: &Path, suffix: &str) -> PathBuf {
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
