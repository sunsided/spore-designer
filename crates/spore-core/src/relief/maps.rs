//! Height (displacement) and gloss (specular) maps.

use crate::Rgba;

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
