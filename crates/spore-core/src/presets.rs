//! Starting points, from Sporagon's two species to plain fluff and a lurid lichen bloom.

use crate::params::*;

pub type Preset = fn() -> MoldParams;

pub const ALL: &[(&str, Preset)] = &[
    ("Penicillium", penicillium),
    ("Rhizopus", rhizopus),
    ("Fluffy", fluffy),
    ("Annihilation", annihilation),
];

fn stops(s: &[(f32, Rgb)]) -> Vec<ColorStop> {
    s.iter().map(|&(t, rgb)| ColorStop { t, rgb }).collect()
}

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
