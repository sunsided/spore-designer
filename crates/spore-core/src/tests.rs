use super::*;

fn all_presets() -> Vec<(&'static str, MoldParams)> {
    presets::ALL
        .iter()
        .map(|(name, make)| (*name, make()))
        .collect()
}

#[test]
fn deterministic() {
    for (name, design) in all_presets() {
        assert!(
            render(&design, 7, 0.6, 64) == render(&design, 7, 0.6, 64),
            "{name} differs between runs"
        );
    }
}

#[test]
fn coverage_grows() {
    for (name, design) in all_presets() {
        let coverage: Vec<f32> = (0..10)
            .map(|stage| render(&design, 3, stage_growth(stage, 10), 64).coverage())
            .collect();
        for pair in coverage.windows(2) {
            assert!(
                pair[1] >= pair[0] * 0.98,
                "{name}: coverage shrinks {coverage:?}"
            );
        }
        assert!(
            coverage[9] > coverage[0] * 2.0,
            "{name}: barely grows {coverage:?}"
        );
    }
}

#[test]
fn non_empty_at_sizes() {
    for (name, design) in all_presets() {
        for size in [64, 512] {
            let coverage = render(&design, 1, 1.0, size).coverage();
            assert!(coverage > 0.1, "{name} at {size}: coverage {coverage}");
        }
    }
}

#[test]
fn presets_roundtrip_ron() {
    for (name, design) in all_presets() {
        let text = ron::ser::to_string_pretty(&design, Default::default()).unwrap();
        let back: MoldParams = ron::from_str(&text).unwrap();
        assert_eq!(design, back, "{name}");
    }
}

#[test]
fn grown_colony_fits_its_square() {
    for (name, design) in all_presets() {
        let image = render(&design, 2, 1.0, 128);
        let side = image.side;
        let border: f32 = (0..side)
            .flat_map(|i| [(i, 0), (i, side - 1), (0, i), (side - 1, i)])
            .map(|(x, y)| image.pixels[y * side + x][3])
            .sum::<f32>()
            / (4 * side) as f32;
        assert!(border < 0.02, "{name}: clipped at the border ({border})");
    }
}

#[test]
fn patch_grows_and_fills() {
    let patch = Patch::default();
    for (name, design) in all_presets() {
        let at = |g| render_patch(&design, &patch, 5, g, 96);
        assert!(at(0.6) == at(0.6), "{name}: patch not deterministic");

        let (young, old) = (at(0.2).coverage(), at(1.0).coverage());
        assert!(old > young && old > 0.2, "{name}: {young} -> {old}");

        let one_colony = Patch {
            count: 1,
            ..patch.clone()
        };
        let single = render_patch(&design, &one_colony, 5, 1.0, 96);
        assert!(
            old > single.coverage() * 2.0,
            "{name}: patch barely beats one colony"
        );
    }
}
