//! Turning an image's lighting layers into what the light mode asks for: relit by the cursor
//! light, or one of the maps (albedo, height, normals, specular). Shared by the sheet preview
//! (`sheet/relit.rs`) and the endless field (`field.rs`).

use spore_core::Rgba;
use spore_core::relief::{Normal, PointLight, encode_normals, gloss_map, height_map, relight_rows};

use crate::editor::{LightMode, LightSettings};
use crate::work::parallel_map;

/// `image` (with its `normals`) as RGBA8, the way `light.mode` shows it (the baked mode is
/// the lit picture itself and not handled here).
///
/// `light_position` is in the image's pixels. Heights are shown with white at
/// `height_scale` reference pixels. The cursor light is split across `threads` threads by
/// rows.
pub fn shade(
    image: &Rgba,
    normals: &[Normal],
    light: LightSettings,
    light_position: [f32; 2],
    height_scale: f32,
    threads: usize,
) -> Vec<u8> {
    match light.mode {
        LightMode::Baked => image.to_rgba8(),
        LightMode::Albedo => image.albedo_rgba8(),
        LightMode::Cursor => relight(image, normals, &point_light(light, light_position), threads),
        LightMode::Height => grey(
            height_map(image, height_scale)
                .iter()
                .map(|height| (height >> 8) as u8),
        ),
        LightMode::Normals => encode_normals(normals)
            .chunks(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        LightMode::Gloss => grey(gloss_map(image).into_iter()),
    }
}

fn point_light(light: LightSettings, position: [f32; 2]) -> PointLight {
    PointLight {
        x: position[0],
        y: position[1],
        height: light.height,
        ambient: light.ambient,
        specular: light.specular,
        shininess: light.shininess,
    }
}

/// Relight a whole image, rows split into `threads` bands lit in parallel.
fn relight(image: &Rgba, normals: &[Normal], light: &PointLight, threads: usize) -> Vec<u8> {
    let side = image.side;
    let rows_per_band = side.div_ceil(threads.max(1)).max(1);
    let bands: Vec<_> = (0..side)
        .step_by(rows_per_band)
        .map(|first| first..(first + rows_per_band).min(side))
        .collect();

    parallel_map(bands, |rows| {
        let mut band = vec![0u8; rows.len() * side * 4];
        relight_rows(image, normals, light, rows, &mut band);
        band
    })
    .concat()
}

/// Opaque grey RGBA8 from grey values.
fn grey(values: impl Iterator<Item = u8>) -> Vec<u8> {
    values
        .flat_map(|value| [value, value, value, 255])
        .collect()
}
