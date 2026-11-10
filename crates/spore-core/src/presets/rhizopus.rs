//! Sporagon's Rhizopus, black bread mould: white cotton with wandering hyphae and sporangia

use super::{penicillium, stops};
use crate::params::*;

/// Sporagon's Rhizopus, black bread mould: white cotton with wandering hyphae and sporangia
/// that ripen from white to black.
pub fn rhizopus() -> MoldParams {
    let base = penicillium();
    MoldParams {
        shape: Shape {
            radius_base: 4.0,
            radius_growth: 20.0,
            wobble_base: 0.85,
            wobble_amp: 0.3,
            wobble_freq: 1.6,
            spin: false,
            meld: 10.0,
            ..base.shape
        },
        body: Body {
            gradient: stops(&[(0.0, [246.0, 243.5, 231.2])]),
            softness: 1.0,
            fluff_low: 150.0 / 246.0,
            youth: 0.0,
            age: 0.8,
            ..base.body
        },
        zones: Zones {
            amount: 0.0,
            ..base.zones
        },
        dust: Dust {
            amount: 0.0,
            ..base.dust
        },
        furrows: Furrows {
            depth: 0.0,
            ..base.furrows
        },
        lighting: Lighting {
            grain: 0.0,
            ..base.lighting
        },
        specks: Specks {
            amount: 0.0,
            ..base.specks
        },
        fringe: Fringe {
            enabled: false,
            ..base.fringe
        },
        hyphae: Hyphae {
            enabled: true,
            ..base.hyphae
        },
        sporangia: Sporangia {
            enabled: true,
            ..base.sporangia
        },
        drops: Drops {
            enabled: false,
            ..base.drops
        },
        slime: Slime {
            deep: [96.0, 92.0, 70.0],
            light: [204.0, 198.0, 164.0],
            alpha: 0.68,
            bubbles: 0.05,
            bubble: [34.0, 32.0, 34.0],
            ..base.slime
        },
        ..base
    }
}
