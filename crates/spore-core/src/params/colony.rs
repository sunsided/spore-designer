//! The colony itself: its outline ([`Shape`]) and the colour and coverage of its mass
//! ([`Body`]). Painted by `paint/body.rs`.

use serde::{Deserialize, Serialize};

use super::{ColorStop, Rgb};

/// Colony outline. At growth `g` the radius is `radius_base + radius_growth * g`, and the
/// edge in each direction is that radius times `wobble_base + wobble_amp * noise + lobes`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub radius_base: f32,
    pub radius_growth: f32,
    /// Edge distance as a fraction of the radius, before wobble.
    pub wobble_base: f32,
    /// How far angular noise pushes the edge in and out.
    pub wobble_amp: f32,
    /// How many bumps the wobble noise has around the rim (roughly).
    pub wobble_freq: f32,
    /// Regular lobes around the rim, like a flower.
    pub lobes: u32,
    pub lobe_amp: f32,
    /// Rotate the outline by a random angle per seed.
    pub spin: bool,
    /// Widest gap (reference pixels) between neighbouring colonies that fuses them into one
    /// mass in patches and fields; 0 keeps every colony a separate disc.
    #[serde(default)]
    pub meld: f32,
}

/// The colony mass: colour by normalised radius (0 centre, 1 edge), from hard velvet to soft
/// cotton.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    /// Colour from the centre (`t = 0`) to the edge (`t = 1`).
    pub gradient: Vec<ColorStop>,
    /// 0 = solid velvet disc, 1 = cotton whose coverage follows the fluff noise.
    pub softness: f32,
    /// Size of the fluff noise blobs.
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
