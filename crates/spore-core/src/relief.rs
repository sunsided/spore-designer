//! Maps for real-time lighting, made from the layers every painting pass fills in (see
//! [`Rgba`]):
//!
//! - **displacement / height map** ([`height_map`]): surface height as 16-bit grey;
//! - **normal map** ([`normal_map`], [`encode_normals`]): which way the surface faces, in the
//!   common OpenGL convention (tangent space, X right, Y up, Z out of the image);
//! - **specular map** ([`gloss_map`]): how shiny each pixel is;
//! - **albedo** ([`Rgba::albedo_rgba8`]): the colour without baked lighting.
//!
//! Together they let a game (or the designer's cursor light, `relief/light.rs`) light a
//! mould from any direction.

mod light;

use crate::Rgba;

pub use light::{PointLight, relight_rows};

/// A unit surface normal `[x, y, z]`, OpenGL convention (Y up).
pub type Normal = [f32; 3];

// ------------------------------------------------------------------------------------------
// Normals

/// The surface normal at every pixel, from the height layer's slope.
///
/// Slopes are measured with central differences in reference pixels, so a design gives the
/// same normals at any output size. `strength` exaggerates (`> 1`) or flattens (`< 1`) the
/// relief. Image rows run downwards, normal Y upwards: hence the sign flip on Y.
pub fn normal_map(image: &Rgba, strength: f32) -> Vec<Normal> {
    let side = image.side;
    let height_at = |x: usize, y: usize| image.height[y * side + x];
    // Two neighbours are `2 / scale` reference pixels apart.
    let slope_factor = image.scale * 0.5 * strength;

    let mut normals = Vec::with_capacity(side * side);
    for y in 0..side {
        let (up, down) = (y.saturating_sub(1), (y + 1).min(side - 1));
        for x in 0..side {
            let (left, right) = (x.saturating_sub(1), (x + 1).min(side - 1));
            let slope_x = (height_at(right, y) - height_at(left, y)) * slope_factor;
            let slope_y_down = (height_at(x, down) - height_at(x, up)) * slope_factor;

            let (normal_x, normal_y, normal_z) = (-slope_x, slope_y_down, 1.0);
            let length = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();
            normals.push([normal_x / length, normal_y / length, normal_z / length]);
        }
    }
    normals
}

/// Normals as RGB8, the usual normal-map encoding: each component mapped from `-1..1` to
/// `0..255` (a flat surface is `128, 128, 255`, the familiar light blue).
pub fn encode_normals(normals: &[Normal]) -> Vec<u8> {
    normals
        .iter()
        .flat_map(|normal| normal.map(|component| ((component * 0.5 + 0.5) * 255.0).round() as u8))
        .collect()
}

// ------------------------------------------------------------------------------------------
// Height and gloss

/// The tallest point of an image, reference pixels; at least a tiny positive value, so it
/// can always divide.
pub fn max_height(image: &Rgba) -> f32 {
    image.height.iter().copied().fold(1e-3, f32::max)
}

/// Heights as 16-bit grey, `0` at the ground and `65535` at `max_height` (reference pixels).
/// Use one `max_height` for all images that must match, e.g. every stage of a sheet.
pub fn height_map(image: &Rgba, max_height: f32) -> Vec<u16> {
    image
        .height
        .iter()
        .map(|height| ((height / max_height).clamp(0.0, 1.0) * 65535.0).round() as u16)
        .collect()
}

/// Shininess as 8-bit grey, `0` dull to `255` wet.
pub fn gloss_map(image: &Rgba) -> Vec<u8> {
    image
        .gloss
        .iter()
        .map(|gloss| (gloss.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect()
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{presets, render};

    #[test]
    fn domes_tilt_outwards_and_flat_ground_faces_up() {
        let image = render(&presets::penicillium(), 1, 1.0, 64);
        let normals = normal_map(&image, 1.0);
        let side = image.side;
        let center = side / 2;

        // Corner: no mould, flat ground.
        assert_eq!(encode_normals(&normals[..1]), vec![128, 128, 255]);

        // Left of the centre the dome faces left (-X), right of it right (+X); above the
        // centre it faces up (+Y in the normal map).
        let at = |x: usize, y: usize| normals[y * side + x];
        assert!(at(center - 10, center)[0] < -0.05);
        assert!(at(center + 10, center)[0] > 0.05);
        assert!(at(center, center - 10)[1] > 0.05);

        assert!(max_height(&image) > 3.0, "a grown colony stands out");
        assert!(gloss_map(&image).iter().any(|&gloss| gloss > 30));
    }
}
