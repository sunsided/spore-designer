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
//! - **The image type:** `canvas.rs` ([`Rgba`]: lit picture, albedo, height, gloss);
//!   sprite sheets in `sheet.rs`.
//! - **Maps for real-time lighting:** `relief.rs` (normal, height and specular maps) and
//!   `relief/light.rs` (relighting with a point light).
//! - **Building blocks:** `noise.rs` (hash, value noise, fbm, RNG) and `math.rs` (blends,
//!   smoothstep, hue shift).

mod canvas;
pub mod math;
pub mod noise;
mod paint;
pub mod params;
pub mod presets;
pub mod relief;
mod sheet;

pub use canvas::Rgba;
pub use paint::PatchLayout;
pub use params::*;
pub use sheet::{crop, grid, sheet};

/// Side length, in pixels, of the sprite the parameters were tuned on (Sporagon's `SPR`).
/// Lengths in a design are measured in this sprite's pixels ("reference pixels").
pub const REFERENCE_SIZE: usize = 64;

/// Cells per side of an endless-field chunk (see [`render_chunk`]).
pub const CHUNK_CELLS: usize = 2;

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
    render_chunk_with_margin(design, field, seed, g, chunk, size, 0)
}

/// Like [`render_chunk`], plus `margin` pixels of the neighbouring chunks on every side: an
/// image `side + 2 * margin` wide whose top-left sits `margin` pixels up and left of the
/// chunk's. Filters that read neighbouring pixels (such as normal maps) then have real
/// neighbours at the chunk's edges; crop the result with [`Rgba::cropped`].
pub fn render_chunk_with_margin(
    design: &MoldParams,
    field: &Field,
    seed: u32,
    g: f32,
    chunk: (i32, i32),
    size: usize,
    margin: usize,
) -> Rgba {
    paint::paint_chunk(
        design,
        field,
        seed,
        g.clamp(0.0, 1.0),
        chunk,
        size.max(1),
        margin,
    )
}

/// Growth of stage `index` of `stages`, matching Sporagon: `(index + 1) / stages`.
pub fn stage_growth(index: usize, stages: usize) -> f32 {
    (index + 1) as f32 / stages.max(1) as f32
}

#[cfg(test)]
mod tests;
