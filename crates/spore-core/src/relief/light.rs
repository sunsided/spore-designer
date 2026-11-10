//! Lighting a mould from its maps with a point light: the reference for what a game shader
//! would do with the exported albedo, normal, height and specular maps.

use super::Normal;
use crate::Rgba;

/// A small light hovering over the image.
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    /// Position over the image, in image pixels (`0, 0` is the top-left corner).
    pub x: f32,
    pub y: f32,
    /// Height above the ground, reference pixels.
    pub height: f32,
    /// Light reaching every surface regardless of direction, `0..1`.
    pub ambient: f32,
    /// Strength of the glints on glossy surfaces.
    pub specular: f32,
    /// Glint sharpness on the wettest surfaces (Blinn-Phong exponent).
    pub shininess: f32,
}

/// Light rows `rows` of `image` with `light`, writing straight RGBA8 into `out` (exactly
/// those rows: `rows.len() * side * 4` bytes). Rows are independent, so callers can split an
/// image across threads.
///
/// Per pixel: albedo times (ambient + diffuse), plus a white Blinn-Phong glint scaled by the
/// specular map. The light direction comes from the light's position relative to the
/// surface point, including that point's height.
pub fn relight_rows(
    image: &Rgba,
    normals: &[Normal],
    light: &PointLight,
    rows: std::ops::Range<usize>,
    out: &mut [u8],
) {
    let side = image.side;
    let light_z = light.height * image.scale;

    for (row_offset, y) in rows.enumerate() {
        for x in 0..side {
            let index = y * side + x;
            let albedo = image.albedo[index];
            let normal = normals[index];

            // Direction to the light, in the normal map's frame (Y up, so image Y flips).
            let surface_z = image.height[index] * image.scale;
            let to_light = normalize([
                light.x - (x as f32 + 0.5),
                (y as f32 + 0.5) - light.y,
                light_z - surface_z,
            ]);
            let diffuse = dot(normal, to_light).max(0.0);

            // Blinn-Phong: the half-vector between light and viewer (straight above).
            let half = normalize([to_light[0], to_light[1], to_light[2] + 1.0]);
            let gloss = image.gloss[index];
            let exponent = 4.0 + gloss * light.shininess;
            let glint = if diffuse > 0.0 {
                dot(normal, half).max(0.0).powf(exponent) * gloss * light.specular
            } else {
                0.0
            };

            let brightness = light.ambient + (1.0 - light.ambient) * diffuse;
            let at = (row_offset * side + x) * 4;
            for channel in 0..3 {
                let value = albedo[channel] * brightness + glint * 255.0;
                out[at + channel] = value.clamp(0.0, 255.0) as u8;
            }
            out[at + 3] = (albedo[3].clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt().max(1e-6);
    [v[0] / length, v[1] / length, v[2] / length]
}
