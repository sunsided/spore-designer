//! Plain fluff: soft white cotton, dense hyphae, no spore heads, no gloss.

use super::{rhizopus, stops};
use crate::params::*;

/// Plain fluff: soft white cotton, dense hyphae, no spore heads, no gloss.
pub fn fluffy() -> MoldParams {
    let base = rhizopus();
    MoldParams {
        body: Body {
            gradient: stops(&[(0.0, [236.0, 236.0, 230.0]), (1.0, [252.0, 252.0, 248.0])]),
            fluff_low: 0.78,
            fluff_scale: 3.0,
            age: 0.15,
            ..base.body
        },
        lighting: Lighting {
            dome: 0.6,
            ..base.lighting
        },
        fringe: Fringe {
            enabled: true,
            width: 0.35,
            alpha: 0.7,
            rgb: [248.0, 248.0, 244.0],
            ..base.fringe
        },
        hyphae: Hyphae {
            base: 40.0,
            growth: 140.0,
            length: 1.4,
            jitter: 0.45,
            alpha: 0.45,
            outer_alpha: 0.3,
            ..base.hyphae
        },
        sporangia: Sporangia {
            enabled: false,
            ..base.sporangia
        },
        glisten: Glisten {
            enabled: false,
            ..base.glisten
        },
        slime: Slime {
            light: [228.0, 226.0, 214.0],
            bubbles: 0.0,
            ..base.slime
        },
        ..base
    }
}
