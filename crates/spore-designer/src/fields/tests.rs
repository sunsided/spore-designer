use super::*;
use crate::design::View;
use spore_core::{Field, Patch};
use spore_core::{Rgba, presets, render, render_patch};

/// Asserts every value lies within its range.
struct InRange;

impl Form for InRange {
    fn section(&mut self, _: &str, body: &mut dyn FnMut(&mut dyn Form)) {
        body(self);
    }

    fn float(&mut self, _: &str, value: &mut f32, range: RangeInclusive<f32>) {
        assert!(range.contains(value), "{value} outside {range:?}");
    }

    fn int(&mut self, _: &str, value: &mut u32, range: RangeInclusive<u32>) {
        assert!(range.contains(value), "{value} outside {range:?}");
    }

    fn toggle(&mut self, _: &str, _: &mut bool) {}

    fn color(&mut self, _: &str, value: &mut Rgb) {
        assert!(value.iter().all(|channel| (0.0..=255.0).contains(channel)));
    }

    fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
        assert!(stops.windows(2).all(|pair| pair[0].t <= pair[1].t));
    }
}

/// Strongest alpha on the outermost ring of pixels.
fn edge_alpha(image: &Rgba) -> f32 {
    let side = image.side;
    (0..side)
        .flat_map(|i| [(i, 0), (i, side - 1), (0, i), (side - 1, i)])
        .map(|(x, y)| image.pixels[y * side + x][3])
        .fold(0.0, f32::max)
}

#[test]
fn randomize_stays_in_range_and_changes_things() {
    let mut mold_params = presets::penicillium();
    let mut randomizer = Randomizer(Rng(0x1234_5678_9ABC_DEF1));
    for _ in 0..50 {
        mold(&mut randomizer, &mut mold_params);
        mold(&mut InRange, &mut mold_params);
        // Every random design must still paint.
        let _ = render(&mold_params, 1, 1.0, 32);
    }
    assert!(mold_params != presets::penicillium());
}

#[test]
fn absurd_design_file_is_tamed() {
    let mut design = Design {
        seed: 1,
        size: 1 << 20,
        stages: 1 << 20,
        mold: presets::rhizopus(),
        view: View::Patch,
        patch: Patch {
            count: 1_000_000_000,
            area: f32::NAN,
            stagger: -5.0,
        },
        field: Field::default(),
        relief: f32::NAN,
    };
    design.mold.shape.radius_growth = 100_000.0;
    design.mold.hyphae.length = f32::INFINITY;
    design.mold.slime.lobes = u32::MAX;
    design.mold.body.gradient = vec![
        ColorStop {
            t: 9.0,
            rgb: [f32::NAN, 1e9, -3.0]
        };
        10_000
    ];

    sanitize(&mut design);
    mold(&mut InRange, &mut design.mold);
    assert_eq!(
        (design.size, design.stages, design.patch.count),
        (512, 24, 40)
    );
    assert_eq!(design.patch.area, 1.0);
    assert!(design.mold.body.gradient.len() <= MAX_STOPS);

    // Still paints, at a sane size.
    design.size = 32;
    let image = render_patch(&design.mold, &design.patch, design.seed, 1.0, design.size);
    assert!(image.side < 4096, "image {} px wide", image.side);
}

#[test]
fn random_designs_are_never_cut_off() {
    let mut randomizer = Randomizer(Rng(0x9E37_79B9_7F4A_7C15));
    let mut mold_params = presets::penicillium();
    for seed in 0..300 {
        mold(&mut randomizer, &mut mold_params);
        for growth in [0.5, 1.0] {
            let image = render(&mold_params, seed, growth, 32);
            assert!(
                edge_alpha(&image) < 0.004,
                "design {seed} cut off at g={growth}: {mold_params:?}"
            );
        }

        let random_patch = Patch {
            count: 1 + randomizer.0.next_u32() % 6,
            area: randomizer.0.range(1.0, 3.0),
            stagger: randomizer.0.unit(),
        };
        let image = render_patch(&mold_params, &random_patch, seed, 1.0, 40);
        assert!(
            edge_alpha(&image) < 0.004,
            "patch {seed} cut off: {random_patch:?} {mold_params:?}"
        );
    }
}
