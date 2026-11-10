//! Randomness that is the same on every run: an integer hash, smooth value noise built on it,
//! layered fractal noise, and a tiny random-number generator. Ported from Sporagon's `art.rs`.
//!
//! Everything here is deterministic: the same inputs always give the same output, so a design
//! with a given seed always paints the same pixels.

mod rng;

use crate::math::lerp;

pub use rng::Rng;

/// A pseudo-random value in `0..1` for the integer point `(x, y)`, different for every `seed`.
pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut bits = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    bits = (bits ^ (bits >> 13)).wrapping_mul(1_274_126_177);
    bits ^= bits >> 16;

    // Keep 24 bits: exactly representable as an f32 fraction.
    (bits & 0xFF_FFFF) as f32 / 16_777_216.0
}

/// Value noise: [`hash`] values at the integer corners around `(x, y)`, blended smoothly in
/// between. Gives soft blobs about one unit across.
pub fn vnoise(x: f32, y: f32, seed: u32) -> f32 {
    let (cell_x, cell_y) = (x.floor(), y.floor());
    let (frac_x, frac_y) = (x - cell_x, y - cell_y);

    // Smoothstep the fractions so the blend has no visible creases at cell borders.
    let (ease_x, ease_y) = (
        frac_x * frac_x * (3.0 - 2.0 * frac_x),
        frac_y * frac_y * (3.0 - 2.0 * frac_y),
    );

    let (ix, iy) = (cell_x as i32, cell_y as i32);
    let top_left = hash(ix, iy, seed);
    let top_right = hash(ix + 1, iy, seed);
    let bottom_left = hash(ix, iy + 1, seed);
    let bottom_right = hash(ix + 1, iy + 1, seed);

    lerp(
        lerp(top_left, top_right, ease_x),
        lerp(bottom_left, bottom_right, ease_x),
        ease_y,
    )
}

/// Fractal noise ("fbm"): `octaves` layers of [`vnoise`], each about twice as fine and half
/// as strong as the one before, normalised back to `0..1`. Looks like natural texture.
pub fn fbm(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let (mut sum, mut amplitude, mut total_amplitude, mut frequency) = (0.0, 1.0, 0.0, 1.0);
    for octave in 0..octaves.max(1) {
        sum += amplitude
            * vnoise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * 101),
            );
        total_amplitude += amplitude;
        amplitude *= 0.5;
        frequency *= 2.03;
    }
    sum / total_amplitude
}
