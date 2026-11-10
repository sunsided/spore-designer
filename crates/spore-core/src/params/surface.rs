//! Texture painted onto the body's colour: rings, dust, furrows, lighting, specks and colour
//! drift. Applied in `paint/body/surface.rs`.

use serde::{Deserialize, Serialize};

use super::Rgb;

/// Concentric growth rings pulling the colour towards `rgb` between normalised radius `start`
/// and `end`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zones {
    pub amount: f32,
    /// Rings per unit of normalised radius (in radians of a sine).
    pub freq: f32,
    pub start: f32,
    pub end: f32,
    pub rgb: Rgb,
}

/// Dusty bloom of conidia (powdery spores): noise patches between `low` and `high` tinted
/// towards `rgb`, out to normalised radius `extent`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dust {
    pub amount: f32,
    /// Size of the dust patches.
    pub scale: f32,
    pub low: f32,
    pub high: f32,
    pub extent: f32,
    pub rgb: Rgb,
}

/// Radial furrows; the count gets `0..=6` added per seed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Furrows {
    /// How much a furrow darkens.
    pub depth: f32,
    pub count: f32,
    /// Higher = narrower furrows.
    pub sharpness: f32,
}

/// Shading of the body as a dome lit from the top right, plus fine brightness grain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lighting {
    /// Dome shading strength, 0 = flat.
    pub dome: f32,
    /// Fine brightness texture amplitude.
    pub grain: f32,
    pub grain_scale: f32,
}

/// Sporulating powder: a fraction `amount` of pixels brightened by `bright`, a bit fewer
/// darkened by `dark`, out to normalised radius `extent`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Specks {
    pub amount: f32,
    pub bright: f32,
    pub dark: f32,
    pub extent: f32,
}

/// Low-frequency hue and saturation drift over the body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Variance {
    /// Peak hue shift in turns.
    pub hue: f32,
    /// Saturation multiplier.
    pub saturation: f32,
    /// Size of the drift patches.
    pub scale: f32,
}
