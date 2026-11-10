//! Everything that shapes a mould. All lengths are in a 64-pixel reference sprite and scale
//! with the output size; colours are RGB in 0..255.

use serde::{Deserialize, Serialize};

pub type Rgb = [f32; 3];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    /// Position along the gradient, 0..1.
    pub t: f32,
    pub rgb: Rgb,
}

/// Sample a gradient whose stops are sorted by `t`. Clamps outside the stops.
pub fn sample(stops: &[ColorStop], t: f32) -> Rgb {
    let Some(first) = stops.first() else {
        return [255.0; 3];
    };
    if t <= first.t {
        return first.rgb;
    }
    for w in stops.windows(2) {
        if t <= w[1].t {
            let k = (t - w[0].t) / (w[1].t - w[0].t).max(1e-6);
            return crate::noise::mix(w[0].rgb, w[1].rgb, k);
        }
    }
    stops[stops.len() - 1].rgb
}

/// A complete mould design. Every layer of Sporagon's Penicillium and Rhizopus painters is in
/// here, so both species (and anything in between) are just different values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MoldParams {
    pub shape: Shape,
    pub body: Body,
    pub zones: Zones,
    pub dust: Dust,
    pub furrows: Furrows,
    pub lighting: Lighting,
    pub specks: Specks,
    pub variance: Variance,
    pub rosettes: Rosettes,
    pub fringe: Fringe,
    pub hyphae: Hyphae,
    pub sporangia: Sporangia,
    pub drops: Drops,
    pub glisten: Glisten,
    pub slime: Slime,
}

impl Default for MoldParams {
    fn default() -> Self {
        crate::presets::penicillium()
    }
}

/// Colony outline: radius `base + growth * g`, wobbled by angular noise and optional lobes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub radius_base: f32,
    pub radius_growth: f32,
    pub wobble_base: f32,
    pub wobble_amp: f32,
    pub wobble_freq: f32,
    pub lobes: u32,
    pub lobe_amp: f32,
    /// Rotate the outline by a random angle per seed.
    pub spin: bool,
}

/// The colony mass: colour by normalised radius, from hard velvet to soft cotton.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub gradient: Vec<ColorStop>,
    /// 0 = solid velvet disc, 1 = cotton whose coverage follows the fluff noise.
    pub softness: f32,
    pub fluff_scale: f32,
    /// Brightness of the darkest fluff, 1 = no fluff shading.
    pub fluff_low: f32,
    /// How strongly young colonies are still white mycelium (`young` colour).
    pub youth: f32,
    pub young: Rgb,
    /// Old centres grey towards `aged` as spores fall.
    pub age: f32,
    pub aged: Rgb,
}

/// Concentric growth rings pulling the colour towards `rgb` between `start` and `end`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zones {
    pub amount: f32,
    pub freq: f32,
    pub start: f32,
    pub end: f32,
    pub rgb: Rgb,
}

/// Dusty bloom of conidia.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dust {
    pub amount: f32,
    pub scale: f32,
    pub low: f32,
    pub high: f32,
    pub extent: f32,
    pub rgb: Rgb,
}

/// Radial furrows; the count gets `0..=6` added per seed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Furrows {
    pub depth: f32,
    pub count: f32,
    pub sharpness: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lighting {
    /// Dome shading strength, 0 = flat.
    pub dome: f32,
    /// Fine brightness texture amplitude.
    pub grain: f32,
    pub grain_scale: f32,
}

/// Sporulating powder: a fraction `amount` of pixels brightened, a bit fewer darkened.
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
    pub saturation: f32,
    pub scale: f32,
}

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
    pub gradient: Vec<ColorStop>,
    /// Ring bands from centre to edge.
    pub rings: f32,
    /// Noise breaking up the rings into blotches.
    pub blotch: f32,
    /// Random hue shift per rosette, in turns.
    pub hue_jitter: f32,
    /// Darkening towards each rosette's rim.
    pub rim_dark: f32,
    pub wobble: f32,
    pub alpha: f32,
}

/// Fuzzy fringe of young hyphae past the edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fringe {
    pub enabled: bool,
    pub width: f32,
    pub alpha: f32,
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
    /// Random darkening per step, 0..1.
    pub variance: f32,
    pub alpha: f32,
    pub outer_alpha: f32,
}

/// Spore heads on stalks, `count * g²` of them, ripening from `unripe` to `ripe`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sporangia {
    pub enabled: bool,
    pub count: f32,
    pub ripe_chance: f32,
    pub ripe: Rgb,
    pub unripe: Rgb,
    pub stalk: Rgb,
    pub size_min: f32,
    pub size_max: f32,
}

/// Guttation: droplets of exudate on older colonies.
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

/// Wet look: specular sheen and clear beads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Glisten {
    pub enabled: bool,
    pub sheen: f32,
    pub power: f32,
    pub beads_base: f32,
    pub beads_growth: f32,
}

/// The mucus puddle under a colony: a central metaball plus `lobes` around it, all scaled by
/// growth. Shaded as a translucent film between `deep` and `light`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slime {
    pub enabled: bool,
    pub size: f32,
    pub lobes: u32,
    pub reach: f32,
    pub reach_jitter: f32,
    pub lobe_size: f32,
    pub lobe_jitter: f32,
    pub ripple: f32,
    pub deep: Rgb,
    pub light: Rgb,
    pub alpha: f32,
    /// Extra opacity where the film is thick.
    pub depth_alpha: f32,
    /// Specular exponent.
    pub gloss: f32,
    pub meniscus: f32,
    /// Fraction of pixels carrying a bubble (or spore) of colour `bubble`.
    pub bubbles: f32,
    pub bubble: Rgb,
}

/// How a patch of colonies is laid out: `count` colonies on a square `area` colony widths
/// wide, each starting to grow after a random delay of up to `stagger`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub count: u32,
    pub area: f32,
    pub stagger: f32,
}

impl Default for Patch {
    fn default() -> Self {
        Self {
            count: 7,
            area: 2.5,
            stagger: 0.5,
        }
    }
}

/// An endless field of colonies: one per cell of a grid `spacing` colony widths wide (with
/// probability `density`), pushed off-centre by up to `jitter` of a cell, each starting to
/// grow after a random delay of up to `stagger`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub spacing: f32,
    pub jitter: f32,
    pub density: f32,
    pub stagger: f32,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            spacing: 1.1,
            jitter: 0.8,
            density: 0.85,
            stagger: 0.6,
        }
    }
}
