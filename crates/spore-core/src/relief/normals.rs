//! Normal maps: which way the surface faces, from the height layer's slope.

use super::Normal;
use crate::Rgba;

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
