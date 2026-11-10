//! Sporagon's Penicillium: velvety blue-green with concentric zones, radial furrows, a white

use super::{no_rosettes, stops};
use crate::params::*;

/// Sporagon's Penicillium: velvety blue-green with concentric zones, radial furrows, a white
/// advancing margin and amber guttation droplets.
pub fn penicillium() -> MoldParams {
    let margin = [238.0, 240.0, 228.0];
    MoldParams {
        shape: Shape {
            radius_base: 4.0,
            radius_growth: 19.0,
            wobble_base: 0.9,
            wobble_amp: 0.22,
            wobble_freq: 2.0,
            lobes: 5,
            lobe_amp: 0.0,
            spin: true,
            meld: 6.0,
        },
        body: Body {
            gradient: stops(&[
                (0.0, [118.0, 132.0, 104.0]),
                (0.3, [52.0, 104.0, 90.0]),
                (0.72, [82.0, 132.0, 108.0]),
                (0.88, [158.0, 196.0, 160.0]),
                (1.0, margin),
            ]),
            softness: 0.0,
            fluff_scale: 2.3,
            fluff_low: 1.0,
            youth: 1.4,
            young: margin,
            age: 0.0,
            aged: [96.0, 94.0, 90.0],
        },
        zones: Zones {
            amount: 1.0,
            freq: 7.0,
            start: 0.3,
            end: 0.72,
            rgb: [82.0, 132.0, 108.0],
        },
        dust: Dust {
            amount: 0.5,
            scale: 4.0,
            low: 0.55,
            high: 0.8,
            extent: 0.85,
            rgb: [150.0, 176.0, 158.0],
        },
        furrows: Furrows {
            depth: 0.25,
            count: 9.0,
            sharpness: 6.0,
        },
        lighting: Lighting {
            dome: 1.0,
            grain: 0.32,
            grain_scale: 1.7,
        },
        specks: Specks {
            amount: 0.07,
            bright: 1.25,
            dark: 0.7,
            extent: 0.85,
        },
        variance: Variance {
            hue: 0.0,
            saturation: 1.0,
            scale: 12.0,
        },
        rosettes: no_rosettes(),
        fringe: Fringe {
            enabled: true,
            width: 0.25,
            alpha: 0.85,
            scale: 1.3,
            rgb: [222.0, 228.0, 216.0],
        },
        hyphae: Hyphae {
            enabled: false,
            base: 20.0,
            growth: 70.0,
            jitter: 0.35,
            length: 1.1,
            rgb: [245.0, 245.0, 235.0],
            variance: 0.16,
            alpha: 0.6,
            outer_alpha: 0.35,
        },
        sporangia: Sporangia {
            enabled: false,
            count: 44.0,
            ripe_chance: 0.95,
            ripe: [22.0, 22.0, 26.0],
            unripe: [214.0, 210.0, 196.0],
            stalk: [176.0, 172.0, 160.0],
            size_min: 0.9,
            size_max: 2.2,
        },
        drops: Drops {
            enabled: true,
            min_growth: 0.6,
            count: 2,
            rgb: [196.0, 150.0, 60.0],
            size_min: 1.0,
            size_max: 2.2,
        },
        glisten: Glisten {
            enabled: true,
            sheen: 0.8,
            power: 40.0,
            beads_base: 2.0,
            beads_growth: 5.0,
        },
        // Penicillium weeps a clear, yellowish exudate with trapped air bubbles.
        slime: Slime {
            enabled: true,
            size: 16.0,
            lobes: 7,
            reach: 14.0,
            reach_jitter: 9.0,
            lobe_size: 6.0,
            lobe_jitter: 4.5,
            ripple: 0.2,
            deep: [92.0, 92.0, 54.0],
            light: [200.0, 196.0, 140.0],
            alpha: 0.42,
            depth_alpha: 0.3,
            gloss: 70.0,
            meniscus: 0.3,
            bubbles: 0.015,
            bubble: [250.0, 246.0, 220.0],
        },
    }
}
