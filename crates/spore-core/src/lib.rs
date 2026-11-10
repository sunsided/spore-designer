//! Procedural mould painter: a [`MoldParams`] design, a seed and a growth stage in, a
//! true-colour [`Rgba`] sprite out. Pure and deterministic, so it runs anywhere (native,
//! wasm) and serves as the reference for a future shader port.

mod layers;
pub mod noise;
pub mod params;
pub mod presets;
mod rgba;
mod slime;

pub use params::*;
pub use rgba::Rgba;

/// Side length of the sprite the parameters were tuned on (Sporagon's `SPR`).
pub const REFERENCE_SIZE: usize = 64;

/// Paint one colony at growth `g` (0..1), `size` pixels per cell. The returned square is
/// `size` plus symmetric padding, so the colony may grow out of its cell. The padding depends
/// only on the design, so all stages of a design come out the same size.
pub fn render(p: &MoldParams, seed: u32, g: f32, size: usize) -> Rgba {
    layers::paint(p, seed, g.clamp(0.0, 1.0), size.max(1))
}

/// Paint a patch of several colonies growing together at overall growth `g` into a
/// `size`-pixel square.
pub fn render_patch(p: &MoldParams, patch: &Patch, seed: u32, g: f32, size: usize) -> Rgba {
    layers::paint_patch(p, patch, seed, g.clamp(0.0, 1.0), size.max(1))
}

/// Chunk `(cx, cy)` of an endless field of colonies at overall growth `g`, `size` pixels per
/// cell. Chunks are [`CHUNK_CELLS`] cells square and tile seamlessly: the chunk at `(cx, cy)`
/// covers canvas pixels from `(cx, cy) * w` for its width `w`.
pub fn render_chunk(
    p: &MoldParams,
    field: &Field,
    seed: u32,
    g: f32,
    chunk: (i32, i32),
    size: usize,
) -> Rgba {
    layers::paint_chunk(p, field, seed, g.clamp(0.0, 1.0), chunk, size.max(1))
}

/// Cells per field chunk side.
pub const CHUNK_CELLS: usize = 2;

/// Growth of stage `i` of `stages`, matching Sporagon: `(i + 1) / stages`.
pub fn stage_growth(i: usize, stages: usize) -> f32 {
    (i + 1) as f32 / stages.max(1) as f32
}

/// All growth stages side by side as one straight-alpha RGBA8 image, `size * stages` wide.
pub fn render_sheet(p: &MoldParams, seed: u32, stages: usize, size: usize) -> Vec<u8> {
    let frames: Vec<Rgba> = (0..stages)
        .map(|i| render(p, seed, stage_growth(i, stages), size))
        .collect();
    sheet(&frames)
}

/// Lay equally sized frames out in one row as straight-alpha RGBA8.
pub fn sheet(frames: &[Rgba]) -> Vec<u8> {
    let Some(first) = frames.first() else {
        return Vec::new();
    };
    let w = first.w;
    let rows: Vec<Vec<u8>> = frames.iter().map(Rgba::to_rgba8).collect();
    let mut out = Vec::with_capacity(rows.len() * w * w * 4);
    for y in 0..w {
        for row in &rows {
            out.extend_from_slice(&row[y * w * 4..(y + 1) * w * 4]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_presets() -> Vec<(&'static str, MoldParams)> {
        presets::ALL.iter().map(|(n, f)| (*n, f())).collect()
    }

    #[test]
    fn deterministic() {
        for (name, p) in all_presets() {
            assert!(
                render(&p, 7, 0.6, 64) == render(&p, 7, 0.6, 64),
                "{name} differs between runs"
            );
        }
    }

    #[test]
    fn coverage_grows() {
        for (name, p) in all_presets() {
            let cov: Vec<f32> = (0..10)
                .map(|i| render(&p, 3, stage_growth(i, 10), 64).coverage())
                .collect();
            for w in cov.windows(2) {
                assert!(w[1] >= w[0] * 0.98, "{name}: coverage shrinks {cov:?}");
            }
            assert!(cov[9] > cov[0] * 2.0, "{name}: barely grows {cov:?}");
        }
    }

    #[test]
    fn non_empty_at_sizes() {
        for (name, p) in all_presets() {
            for size in [64, 512] {
                let c = render(&p, 1, 1.0, size).coverage();
                assert!(c > 0.1, "{name} at {size}: coverage {c}");
            }
        }
    }

    #[test]
    fn presets_roundtrip_ron() {
        for (name, p) in all_presets() {
            let s = ron::ser::to_string_pretty(&p, Default::default()).unwrap();
            let back: MoldParams = ron::from_str(&s).unwrap();
            assert_eq!(p, back, "{name}");
        }
    }

    #[test]
    fn grown_colony_fits_its_square() {
        for (name, p) in all_presets() {
            let img = render(&p, 2, 1.0, 128);
            let w = img.w;
            let border: f32 = (0..w)
                .flat_map(|i| [(i, 0), (i, w - 1), (0, i), (w - 1, i)])
                .map(|(x, y)| img.px[y * w + x][3])
                .sum::<f32>()
                / (4 * w) as f32;
            assert!(border < 0.02, "{name}: clipped at the border ({border})");
        }
    }

    #[test]
    fn patch_grows_and_fills() {
        let patch = Patch::default();
        for (name, p) in all_presets() {
            let at = |g| render_patch(&p, &patch, 5, g, 96);
            assert!(at(0.6) == at(0.6), "{name}: patch not deterministic");
            let (young, old) = (at(0.2).coverage(), at(1.0).coverage());
            assert!(old > young && old > 0.2, "{name}: {young} -> {old}");
            let single = render_patch(
                &p,
                &Patch {
                    count: 1,
                    ..patch.clone()
                },
                5,
                1.0,
                96,
            );
            assert!(
                old > single.coverage() * 2.0,
                "{name}: patch barely beats one colony"
            );
        }
    }

    #[test]
    fn sheet_layout() {
        let p = presets::penicillium();
        let frames = [render(&p, 1, 0.3, 8), render(&p, 1, 1.0, 8)];
        let w = frames[0].w;
        assert_eq!(frames[1].w, w, "stages of one design share a size");
        let s = sheet(&frames);
        assert_eq!(s.len(), 2 * w * w * 4);
        // Row 3 of the sheet: frame 0's row 3, then frame 1's row 3.
        let (row, out) = (w * 4, 2 * w * 4);
        assert_eq!(
            &s[3 * out..3 * out + row],
            &frames[0].to_rgba8()[3 * row..4 * row]
        );
        assert_eq!(
            &s[3 * out + row..4 * out],
            &frames[1].to_rgba8()[3 * row..4 * row]
        );
    }
}
