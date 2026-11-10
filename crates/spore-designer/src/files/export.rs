//! Exporting stages as PNG, with their lighting maps.

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use spore_core::relief::{encode_normals, gloss_map, height_map, max_height, normal_map};
use spore_core::{Rgba, grid, sheet};

use super::Outcome;
use super::deliver::{self, Bundle};

/// A file's name suffix (see [`Bundle::files`]) and contents.
type NamedFile = (&'static str, Vec<u8>);

/// The shown stage as a PNG, plus its maps.
pub(super) fn export_stage(frames: &[Rgba], stage: usize, relief: f32) -> Outcome {
    let Some(image) = frames.get(stage) else {
        return Ok("Nothing to export".into());
    };
    export(
        format!("mould-stage{}", stage + 1),
        std::slice::from_ref(image),
        relief,
    )
}

/// All stages side by side in one row, as a PNG, plus their maps.
pub(super) fn export_sheet(frames: &[Rgba], relief: f32) -> Outcome {
    if frames.is_empty() {
        return Ok("Nothing to export".into());
    }
    export("mould-sheet".into(), frames, relief)
}

fn export(name: String, frames: &[Rgba], relief: f32) -> Outcome {
    let (files, max) = encode_with_maps(frames, relief)?;
    let saved_to = deliver::offer(Bundle {
        name,
        extension: "png",
        kind: "PNG image",
        files,
    })?;
    Ok(format!(
        "Exported {saved_to} with albedo, height, normal and specular maps (white height = {max:.1} reference px)"
    ))
}

/// `frames` side by side as PNG files, all laid out the same way:
///
/// - `""`: the picture as painted (RGBA);
/// - `"-albedo"`: colour without baked lighting (RGBA);
/// - `"-height"`: displacement, 16-bit grey, one scale for all frames;
/// - `"-normal"`: tangent-space normals, OpenGL convention (Y up);
/// - `"-specular"`: shininess, grey.
///
/// Also returns the height that white stands for, in reference pixels.
pub(super) fn encode_with_maps(
    frames: &[Rgba],
    relief: f32,
) -> Result<(Vec<NamedFile>, f32), String> {
    let side = frames[0].side;
    let count = frames.len();
    let (width, height) = ((side * count) as u32, side as u32);
    let layout = |images: Vec<Vec<u8>>, channels: usize| grid(&images, side, channels, count);
    let png = |data: &[u8], color: ExtendedColorType| encode_png(data, width, height, color);

    let lit = png(&sheet(frames, count), ExtendedColorType::Rgba8)?;

    let albedo = frames.iter().map(Rgba::albedo_rgba8).collect();
    let albedo = png(&layout(albedo, 4), ExtendedColorType::Rgba8)?;

    let max = frames.iter().map(max_height).fold(1e-3, f32::max);
    let heights: Vec<Vec<u16>> = frames.iter().map(|frame| height_map(frame, max)).collect();
    // The encoder takes 16-bit samples as native-endian bytes.
    let height_bytes: Vec<u8> = grid(&heights, side, 1, count)
        .iter()
        .flat_map(|sample| sample.to_ne_bytes())
        .collect();
    let heights = png(&height_bytes, ExtendedColorType::L16)?;

    let normals = frames
        .iter()
        .map(|frame| encode_normals(&normal_map(frame, relief)))
        .collect();
    let normals = png(&layout(normals, 3), ExtendedColorType::Rgb8)?;

    let gloss = frames.iter().map(gloss_map).collect();
    let gloss = png(&layout(gloss, 1), ExtendedColorType::L8)?;

    let files = vec![
        ("", lit),
        ("-albedo", albedo),
        ("-height", heights),
        ("-normal", normals),
        ("-specular", gloss),
    ];
    Ok((files, max))
}

fn encode_png(
    data: &[u8],
    width: u32,
    height: u32,
    color: ExtendedColorType,
) -> Result<Vec<u8>, String> {
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(data, width, height, color)
        .map_err(|error| error.to_string())?;
    Ok(png)
}
