//! Procedural mould painter: a [`MoldParams`] design, a seed and a growth stage in, a
//! true-colour [`Rgba`] image out. Pure and deterministic, so it runs anywhere (native,
//! wasm) and serves as the reference for a future shader port.
//!
//! # Where to find what
//!
//! - **What a design is:** `params.rs` (and `params/`), ready-made designs in `presets.rs`.
//! - **How pixels are made:** `paint.rs` explains the painting order and holds the three
//!   views ([`render`], [`render_patch`] / [`PatchLayout`], [`render_chunk`]); each layer
//!   lives in `paint/` (`slime`, `body`, `overlays/`).
//! - **The image type:** `canvas.rs` ([`Rgba`]); sprite sheets in `sheet.rs`.
//! - **Building blocks:** `noise.rs` (hash, value noise, fbm, RNG) and `math.rs` (blends,
//!   smoothstep, hue shift).

mod canvas;
pub mod math;
pub mod noise;
mod paint;
pub mod params;
pub mod presets;
mod sheet;

pub use canvas::Rgba;
pub use paint::PatchLayout;
pub use params::*;
pub use sheet::sheet;

/// Side length, in pixels, of the sprite the parameters were tuned on (Sporagon's `SPR`).
/// Lengths in a design are measured in this sprite's pixels ("reference pixels").
pub const REFERENCE_SIZE: usize = 64;

/// Cells per side of an endless-field chunk (see [`render_chunk`]).
pub const CHUNK_CELLS: usize = 2;

// ------------------------------------------------------------------------------------------
// The three views

/// Paint one colony at growth `g` (`0..1`), `size` pixels per cell. The returned square is
/// `size` plus symmetric padding, so the colony may grow out of its cell. The padding depends
/// only on the design, so all stages of a design come out the same size.
pub fn render(design: &MoldParams, seed: u32, g: f32, size: usize) -> Rgba {
    paint::paint_colony(design, seed, g.clamp(0.0, 1.0), size.max(1))
}

/// Paint a patch of several colonies growing together at overall growth `g`, `size` pixels
/// across the patch area (plus padding, as for [`render`]). For several stages of one patch,
/// build a [`PatchLayout`] once and paint each stage from it.
pub fn render_patch(design: &MoldParams, patch: &Patch, seed: u32, g: f32, size: usize) -> Rgba {
    PatchLayout::new(design, patch, seed, size).render(design, g)
}

/// Chunk `(column, row)` of an endless field of colonies at overall growth `g`, `size` pixels
/// per cell. Chunks are [`CHUNK_CELLS`] cells square and tile seamlessly: the chunk at
/// `(column, row)` covers canvas pixels from `(column, row) * side` for its side length.
pub fn render_chunk(
    design: &MoldParams,
    field: &Field,
    seed: u32,
    g: f32,
    chunk: (i32, i32),
    size: usize,
) -> Rgba {
    paint::paint_chunk(design, field, seed, g.clamp(0.0, 1.0), chunk, size.max(1))
}

/// Growth of stage `index` of `stages`, matching Sporagon: `(index + 1) / stages`.
pub fn stage_growth(index: usize, stages: usize) -> f32 {
    (index + 1) as f32 / stages.max(1) as f32
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
}
