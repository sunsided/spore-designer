//! Small numeric and colour helpers shared by every painting pass. Colours are RGB triples
//! with channels in `0..255`.

mod color;

pub use color::{mix, scale, shift_hue};

/// Linear blend: `a` at `t = 0`, `b` at `t = 1`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smoothstep: 0 below `edge0`, 1 above `edge1`, an S-curve in between.
pub fn smooth(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests;
