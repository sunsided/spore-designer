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

/// Paint a patch of several colonies growing together at overall growth `g`, `size` pixels
/// across the patch area (plus padding, as for [`render`]). For several stages of one patch,
/// build a [`PatchLayout`] once and paint each stage from it.
pub fn render_patch(p: &MoldParams, patch: &Patch, seed: u32, g: f32, size: usize) -> Rgba {
    PatchLayout::new(p, patch, seed, size).render(p, g)
}

pub use layers::PatchLayout;

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

/// Lay equally sized frames out as straight-alpha RGBA8, `cols` per row (rows fill up from
/// the top left; a short last row is transparent). The image is `cols * w` wide.
pub fn sheet(frames: &[Rgba], cols: usize) -> Vec<u8> {
    let Some(first) = frames.first() else {
        return Vec::new();
    };
    let (w, cols) = (first.w, cols.clamp(1, frames.len()));
    let rows = frames.len().div_ceil(cols);
    let mut out = vec![0; rows * cols * w * w * 4];
    for (i, f) in frames.iter().enumerate() {
        let px = f.to_rgba8();
        let (fx, fy) = (i % cols * w, i / cols * w);
        for y in 0..w {
            let at = ((fy + y) * cols * w + fx) * 4;
            out[at..at + w * 4].copy_from_slice(&px[y * w * 4..(y + 1) * w * 4]);
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
    fn sheet_wraps_into_rows() {
        let p = presets::penicillium();
        let frames: Vec<Rgba> = (0..3)
            .map(|i| render(&p, 1, 0.3 + 0.3 * i as f32, 8))
            .collect();
        let w = frames[0].w;
        let s = sheet(&frames, 2);
        // Two columns, two rows; frame 2 starts the second row, the last cell stays empty.
        assert_eq!(s.len(), 2 * w * 2 * w * 4);
        let row =
            |y: usize, col: usize| &s[(y * 2 * w + col * w) * 4..(y * 2 * w + (col + 1) * w) * 4];
        assert_eq!(row(w, 0), &frames[2].to_rgba8()[..w * 4]);
        assert!(row(w, 1).iter().all(|&b| b == 0));
    }

    #[test]
    fn sheet_layout() {
        let p = presets::penicillium();
        let frames = [render(&p, 1, 0.3, 8), render(&p, 1, 1.0, 8)];
        let w = frames[0].w;
        assert_eq!(frames[1].w, w, "stages of one design share a size");
        let s = sheet(&frames, 2);
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
