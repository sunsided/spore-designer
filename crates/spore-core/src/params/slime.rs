//! The mucus puddle under a colony. Painted by `paint/slime.rs`.

use serde::{Deserialize, Serialize};

use super::Rgb;

/// The puddle: a central metaball of radius `size` plus `lobes` balls around it, at distance
/// `reach` (+ up to `reach_jitter`) with radius `lobe_size` (+ up to `lobe_jitter`), all
/// scaled by growth. Shaded as a translucent film between `deep` and `light`.
///
/// A *metaball* is a blob whose influence falls off with distance; where the summed
/// influence of all balls is high enough there is slime, so nearby balls flow into one shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slime {
    pub enabled: bool,
    pub size: f32,
    pub lobes: u32,
    pub reach: f32,
    pub reach_jitter: f32,
    pub lobe_size: f32,
    pub lobe_jitter: f32,
    /// Noise on the field: lumpy edges and many small glints.
    pub ripple: f32,
    /// Colour where the film is thin and in shadow, and where it is thick and lit.
    pub deep: Rgb,
    pub light: Rgb,
    pub alpha: f32,
    /// Extra opacity where the film is thick.
    pub depth_alpha: f32,
    /// Specular exponent.
    pub gloss: f32,
    /// Darkening of the thin rim (and brightening of the line just inside it).
    pub meniscus: f32,
    /// Fraction of pixels carrying a bubble (or spore) of colour `bubble`.
    pub bubbles: f32,
    pub bubble: Rgb,
}
