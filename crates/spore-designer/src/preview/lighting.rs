//! Turning an image's lighting layers into what the light mode asks for: relit by the cursor
//! light, or one of the maps (albedo, height, normals, specular). Shared by the sheet preview
//! (`sheet/relit.rs`) and the endless field (`field.rs`).

use spore_core::Rgba;
use spore_core::relief::{Normal, PointLight, encode_normals, gloss_map, height_map, relight_rows};

use crate::editor::{LightMode, LightSettings};

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

/// Relight a whole image, rows split across `threads` threads.
fn relight(image: &Rgba, normals: &[Normal], light: &PointLight, threads: usize) -> Vec<u8> {
    let side = image.side;
    let rows_per_thread = side.div_ceil(threads.max(1)).max(1);
    let mut out = vec![0u8; side * side * 4];

    std::thread::scope(|scope| {
        for (index, rows) in out.chunks_mut(rows_per_thread * side * 4).enumerate() {
            let first = index * rows_per_thread;
            let last = (first + rows_per_thread).min(side);
            scope.spawn(move || relight_rows(image, normals, light, first..last, rows));
        }
    });
    out
}

/// Opaque grey RGBA8 from grey values.
fn grey(values: impl Iterator<Item = u8>) -> Vec<u8> {
    values
        .flat_map(|value| [value, value, value, 255])
        .collect()
}
