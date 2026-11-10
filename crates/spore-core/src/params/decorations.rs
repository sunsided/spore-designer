//! What grows on and around the body. The fringe is painted with the body
//! (`paint/body/surface.rs`); everything else by `paint/overlays/`.

use serde::{Deserialize, Serialize};

use super::{ColorStop, Rgb};

/// Lichen-like sub-colonies with banded rings, scattered over and around the body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rosettes {
    pub enabled: bool,
    pub count: u32,
    /// Radii relative to the colony radius.
    pub size_min: f32,
    pub size_max: f32,
    /// How far from the centre they settle, relative to the colony radius.
    pub spread: f32,
    /// Ring colours, sampled back and forth from `t = 0` to `t = 1`.
    pub gradient: Vec<ColorStop>,
    /// Ring bands from centre to edge.
    pub rings: f32,
    /// Noise breaking up the rings into blotches.
    pub blotch: f32,
    /// Random hue shift per rosette, in turns.
    pub hue_jitter: f32,
    /// Darkening towards each rosette's rim.
    pub rim_dark: f32,
    /// How irregular each rosette's outline is.
    pub wobble: f32,
    pub alpha: f32,
}

/// Fuzzy fringe of young hyphae just past the edge, `width` as a fraction of the radius.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fringe {
    pub enabled: bool,
    pub width: f32,
    pub alpha: f32,
    /// Size of the fuzz noise, in output pixels.
    pub scale: f32,
    pub rgb: Rgb,
}

/// Aerial hyphae: random-walk strands, `base + growth * g` of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hyphae {
    pub enabled: bool,
    pub base: f32,
    pub growth: f32,
    /// Turn jitter per step, radians.
    pub jitter: f32,
    /// Strand length relative to the colony radius.
    pub length: f32,
    pub rgb: Rgb,
    /// Random darkening per step, `0..1`.
    pub variance: f32,
    /// Opacity over the body, and past its edge.
    pub alpha: f32,
    pub outer_alpha: f32,
}

/// Spore heads on stalks, `count * g²` of them, ripening from `unripe` to `ripe`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sporangia {
    pub enabled: bool,
    pub count: f32,
    /// Chance that a head is ripe at full growth.
    pub ripe_chance: f32,
    pub ripe: Rgb,
    pub unripe: Rgb,
    pub stalk: Rgb,
    pub size_min: f32,
    pub size_max: f32,
}

/// Guttation: droplets of exudate on colonies grown past `min_growth`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drops {
    pub enabled: bool,
    pub min_growth: f32,
    /// Base count; the seed adds `0..=2`.
    pub count: u32,
    pub rgb: Rgb,
    pub size_min: f32,
    pub size_max: f32,
}

/// Wet look: a specular sheen and clear beads, `beads_base + beads_growth * g` of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Glisten {
    pub enabled: bool,
    pub sheen: f32,
    /// Specular exponent: higher = smaller, sharper highlight.
    pub power: f32,
    pub beads_base: f32,
    pub beads_growth: f32,
}
