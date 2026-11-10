//! Ready-made designs, from Sporagon's two species to plain fluff and a lurid lichen bloom.
//!
//! Each preset is a complete [`MoldParams`]. Most start from another one and override only
//! what differs (`..base`), so reading a preset tells you what makes it distinct.

mod annihilation;
mod fluffy;
mod penicillium;
mod rhizopus;

use crate::params::*;

pub use annihilation::annihilation;
pub use fluffy::fluffy;
pub use penicillium::penicillium;
pub use rhizopus::rhizopus;

/// A function that builds a preset.
pub type Preset = fn() -> MoldParams;

/// Every preset with its display name, in menu order.
pub const ALL: &[(&str, Preset)] = &[
    ("Penicillium", penicillium),
    ("Rhizopus", rhizopus),
    ("Fluffy", fluffy),
    ("Annihilation", annihilation),
];

/// A gradient from `(position, colour)` pairs.
fn stops(pairs: &[(f32, Rgb)]) -> Vec<ColorStop> {
    pairs.iter().map(|&(t, rgb)| ColorStop { t, rgb }).collect()
}

/// Rosettes switched off, with sensible values ready for when they are switched on.
fn no_rosettes() -> Rosettes {
    Rosettes {
        enabled: false,
        count: 8,
        size_min: 0.15,
        size_max: 0.4,
        spread: 1.0,
        gradient: stops(&[(0.0, [230.0, 120.0, 170.0]), (1.0, [250.0, 220.0, 90.0])]),
        rings: 3.0,
        blotch: 0.5,
        hue_jitter: 0.1,
        rim_dark: 0.3,
        wobble: 0.3,
        alpha: 1.0,
    }
}
