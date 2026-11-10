//! Everything that shapes a mould, as plain data that saves to and loads from RON files.
//!
//! [`MoldParams`] is one complete design. Its parts are grouped by what they paint:
//!
//! - `params/gradient.rs`: [`ColorStop`] gradients and how they are sampled.
//! - `params/colony.rs`: the colony outline ([`Shape`]) and its mass ([`Body`]).
//! - `params/surface.rs`: texture on the body: [`Zones`], [`Dust`], [`Furrows`],
//!   [`Lighting`], [`Specks`], [`Variance`].
//! - `params/decorations.rs`: what grows on top: [`Rosettes`], [`Fringe`], [`Hyphae`],
//!   [`Sporangia`], [`Drops`], [`Glisten`].
//! - `params/slime.rs`: the puddle under the colony ([`Slime`]).
//! - `params/layout.rs`: how many colonies sit where: [`Patch`], [`Field`].
//!
//! Units: lengths are *reference pixels*, measured in the 64-pixel sprite the parameters were
//! tuned on, and scale with the output size. Colours are RGB with channels in `0..255`.
//!
//! The field names are the save-file format: renaming one breaks existing design files.

mod colony;
mod decorations;
mod gradient;
mod layout;
mod slime;
mod surface;

use serde::{Deserialize, Serialize};

pub use colony::{Body, Shape};
pub use decorations::{Drops, Fringe, Glisten, Hyphae, Rosettes, Sporangia};
pub use gradient::{ColorStop, sample, sorted};
pub use layout::{Field, Patch};
pub use slime::Slime;
pub use surface::{Dust, Furrows, Lighting, Specks, Variance, Zones};

/// An RGB colour, channels in `0..255`.
pub type Rgb = [f32; 3];

// ------------------------------------------------------------------------------------------

/// A complete mould design. Every layer of Sporagon's Penicillium and Rhizopus painters is in
/// here, so both species (and anything in between) are just different values.
///
/// Missing sections in a design file fall back to the Penicillium preset.
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
