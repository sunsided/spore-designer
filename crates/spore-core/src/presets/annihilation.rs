//! The colourful growth from the film's swimming pool: a bruise-violet and coral body

use super::{penicillium, stops};
use crate::params::*;

/// The colourful growth from the film's swimming pool: a bruise-violet and coral body
/// overrun by banded lichen rosettes in pink, ochre and teal.
pub fn annihilation() -> MoldParams {
    let base = penicillium();
    MoldParams {
        shape: Shape {
            radius_growth: 22.0,
            wobble_amp: 0.3,
            lobes: 7,
            lobe_amp: 0.08,
            ..base.shape
        },
        body: Body {
            gradient: stops(&[
                (0.0, [92.0, 40.0, 96.0]),
                (0.35, [186.0, 70.0, 128.0]),
                (0.65, [236.0, 132.0, 110.0]),
                (0.88, [240.0, 210.0, 110.0]),
                (1.0, [246.0, 240.0, 222.0]),
            ]),
            youth: 1.2,
            young: [246.0, 240.0, 222.0],
            ..base.body
        },
        zones: Zones {
            amount: 0.5,
            freq: 11.0,
            start: 0.2,
            end: 0.85,
            rgb: [120.0, 200.0, 190.0],
        },
        dust: Dust {
            amount: 0.35,
            rgb: [250.0, 190.0, 210.0],
            ..base.dust
        },
        variance: Variance {
            hue: 0.12,
            saturation: 1.25,
            scale: 10.0,
        },
        rosettes: Rosettes {
            enabled: true,
            count: 14,
            size_min: 0.12,
            size_max: 0.32,
            spread: 1.0,
            gradient: stops(&[
                (0.0, [60.0, 170.0, 168.0]),
                (0.3, [236.0, 226.0, 140.0]),
                (0.55, [240.0, 120.0, 170.0]),
                (0.8, [200.0, 90.0, 60.0]),
                (1.0, [250.0, 246.0, 236.0]),
            ]),
            rings: 1.4,
            blotch: 1.6,
            hue_jitter: 0.15,
            rim_dark: 0.35,
            wobble: 0.35,
            alpha: 1.0,
        },
        fringe: Fringe {
            rgb: [246.0, 226.0, 236.0],
            ..base.fringe
        },
        drops: Drops {
            rgb: [120.0, 60.0, 150.0],
            ..base.drops
        },
        slime: Slime {
            deep: [110.0, 70.0, 110.0],
            light: [236.0, 200.0, 220.0],
            alpha: 0.5,
            bubbles: 0.02,
            bubble: [255.0, 240.0, 250.0],
            ..base.slime
        },
        ..base
    }
}
