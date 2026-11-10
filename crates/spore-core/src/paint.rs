//! Painting: turning a design into pixels.
//!
//! # The scene
//!
//! Every view paints the same *scene*, in the same order ([`paint_scene`]):
//!
//! 1. **Slime** (`paint/slime.rs`): the puddles under all colonies, as one metaball film, so
//!    neighbouring puddles flow together.
//! 2. **Bodies** (`paint/body.rs`): all colony bodies at once, melded into one mass where
//!    they come close; the fuzzy fringe is part of this pass.
//! 3. **Overlays** (`paint/overlays.rs`): per colony, what sits on top: lichen rosettes,
//!    hyphae, spore heads, drops and gloss.
//!
//! # The views
//!
//! The views differ only in where colonies sit and how large the image is:
//!
//! - [`paint_colony`]: one colony centred in its cell, padded so growth may spill out.
//! - [`PatchLayout`]: several colonies scattered over a square area.
//! - [`paint_chunk`]: one tile of an endless field; tiles meet without seams.
//!
//! Where colonies go is `paint/placement.rs`; how far they can reach (and so how much
//! padding an image needs) is `paint/reach.rs`.
//!
//! # Coordinates
//!
//! Geometry lives in *reference pixels*: pixels of the 64-pixel sprite the parameters were
//! tuned on ([`REFERENCE_SIZE`]). `scale` is output pixels per reference pixel. Images are
//! windows onto an unbounded *canvas* of output pixels (see [`Rgba`]); a canvas pixel `p` is
//! at reference position `(p + 0.5) / scale`. Noise is sampled in reference units so a
//! design looks the same at any size; per-pixel specks use canvas pixels directly.

mod body;
mod brush;
mod lighting;
mod overlays;
mod placement;
mod reach;
mod slime;

use crate::params::{Field, MoldParams, Patch};
use crate::{CHUNK_CELLS, REFERENCE_SIZE, Rgba};

/// One colony in a scene: where it sits, how far it has grown, and the seed that makes it
/// look different from its neighbours.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Colony {
    /// Centre in reference pixels.
    pub x: f32,
    pub y: f32,
    /// `0..1`; a colony that has not started yet is left out of the scene instead.
    pub growth: f32,
    pub seed: u32,
}

// ------------------------------------------------------------------------------------------
// The scene

/// Paint `colonies` into `canvas` at `scale` output pixels per reference pixel, in the order
/// described at the top of this file.
///
/// `slime_cutoff` limits how far each slime ball reaches. The endless field needs that so a
/// pixel only depends on nearby colonies; single colonies and patches pass `None` and get
/// the exact, unlimited metaball field.
fn paint_scene(
    design: &MoldParams,
    scale: f32,
    colonies: &[Colony],
    slime_cutoff: Option<f32>,
    canvas: &mut Rgba,
) {
    canvas.scale = scale;
    if design.slime.enabled {
        let balls = slime::puddles(&design.slime, colonies);
        slime::paint(&design.slime, &balls, scale, slime_cutoff, canvas);
    }

    body::paint_bodies(design, scale, colonies, canvas);

    for colony in colonies {
        overlays::paint(design, colony, scale, canvas);
    }
}

// ------------------------------------------------------------------------------------------
// View 1: one colony

/// One colony in its slime puddle, `size` pixels per 64-pixel reference cell. The image is
/// centred on the cell and padded on all sides, so a fully grown design can spill out of its
/// cell without being cut off.
pub(crate) fn paint_colony(design: &MoldParams, seed: u32, growth: f32, size: usize) -> Rgba {
    let scale = size as f32 / REFERENCE_SIZE as f32;
    let cell_half = REFERENCE_SIZE as f32 / 2.0;

    // Padding is sized for the fully grown colony, so every stage has the same image size.
    let grown = [Colony {
        x: cell_half,
        y: cell_half,
        growth: 1.0,
        seed,
    }];
    let overflow = reach::overflow(
        design,
        &grown,
        REFERENCE_SIZE as f32,
        reach::colony_reach(design) - cell_half,
    );
    let padding = (overflow * scale).ceil() as usize;
    let mut canvas = Rgba::new(size + 2 * padding);

    let center = (canvas.side as f32 / scale) / 2.0;
    let colony = Colony {
        x: center,
        y: center,
        growth,
        seed,
    };
    paint_scene(design, scale, &[colony], None, &mut canvas);
    canvas
}

// ------------------------------------------------------------------------------------------
// View 2: a patch of colonies

/// Where a patch's colonies sit and how much padding its image needs. Neither depends on the
/// growth stage, so it is worked out once per sprite sheet (the padding needs a scan of the
/// grown puddle, which is slow).
pub struct PatchLayout {
    patch: Patch,
    seed: u32,
    size: usize,
    /// Side of the patch area in reference pixels.
    extent: f32,
    /// How far colony centres stay inside the area.
    margin: f32,
    /// Output pixels added on every side.
    padding: usize,
}

impl PatchLayout {
    /// Lay out `patch` for `design`, `size` pixels across the patch area.
    pub fn new(design: &MoldParams, patch: &Patch, seed: u32, size: usize) -> Self {
        let size = size.max(1);
        let extent = REFERENCE_SIZE as f32 * patch.area.max(1.0);
        let scale = size as f32 / extent;

        // Colonies stay at least `margin` inside the area; whatever reaches further gets
        // padding. The grown layout bounds every stage.
        let margin = reach::colony_reach(design).min(extent * 0.5);
        let grown = placement::place_in_patch(patch, seed, extent, margin, 1.0);
        let overflow =
            reach::overflow(design, &grown, extent, reach::colony_reach(design) - margin);
        let padding = (overflow * scale).ceil() as usize;

        Self {
            patch: patch.clone(),
            seed,
            size,
            extent,
            margin,
            padding,
        }
    }

    /// The patch at overall growth `g`: several colonies scattered over a square
    /// `patch.area` colony widths wide, growing at staggered times. Their puddles share one
    /// metaball field and run together. `design` must be the one the layout was made for.
    pub fn render(&self, design: &MoldParams, g: f32) -> Rgba {
        let growth = g.clamp(0.0, 1.0);
        let scale = self.size as f32 / self.extent;
        let mut canvas = Rgba::new(self.size + 2 * self.padding);

        // Shift the area right and down by the padding.
        let offset = self.padding as f32 / scale;
        let colonies: Vec<Colony> =
            placement::place_in_patch(&self.patch, self.seed, self.extent, self.margin, growth)
                .into_iter()
                .map(|colony| Colony {
                    x: colony.x + offset,
                    y: colony.y + offset,
                    ..colony
                })
                .collect();

        paint_scene(design, scale, &colonies, None, &mut canvas);
        canvas
    }
}

// ------------------------------------------------------------------------------------------
// View 3: an endless field

/// Side of a field chunk in reference pixels.
const CHUNK: f32 = (CHUNK_CELLS * REFERENCE_SIZE) as f32;

/// Chunk `(column, row)` of an endless field, `size` pixels per cell. Each pixel only
/// depends on its canvas position, so chunks tile without seams.
pub(crate) fn paint_chunk(
    design: &MoldParams,
    field: &Field,
    seed: u32,
    growth: f32,
    (column, row): (i32, i32),
    size: usize,
) -> Rgba {
    let side = (CHUNK * size as f32 / REFERENCE_SIZE as f32).round() as usize;
    let window = Rgba::window(side, column * side as i32, row * side as i32);
    paint_field_window(design, field, seed, growth, size, window)
}

/// Paint the field into any window of the canvas, `size` pixels per cell.
fn paint_field_window(
    design: &MoldParams,
    field: &Field,
    seed: u32,
    growth: f32,
    size: usize,
    mut canvas: Rgba,
) -> Rgba {
    let scale = size as f32 / REFERENCE_SIZE as f32;

    // The window in reference pixels.
    let (x0, y0) = (canvas.left as f32 / scale, canvas.top as f32 / scale);
    let (x1, y1) = (
        x0 + canvas.side as f32 / scale,
        y0 + canvas.side as f32 / scale,
    );

    // Every colony that can touch the window: its body, or (with slime) its puddle balls,
    // which reach `cutoff` past their lobes.
    let cutoff = slime::field_cutoff(&design.slime);
    let puddle_reach = design.slime.reach + design.slime.reach_jitter + cutoff;
    let reach = if design.slime.enabled {
        reach::colony_reach(design).max(puddle_reach)
    } else {
        reach::colony_reach(design)
    };
    let colonies = placement::field_colonies(field, seed, growth, (x0, y0, x1, y1), reach);

    paint_scene(design, scale, &colonies, Some(cutoff), &mut canvas);
    canvas
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_are_seamless() {
        let field = Field::default();
        for (name, make) in crate::presets::ALL {
            let design = make();
            let (size, growth) = (24, 0.8);
            let left = paint_chunk(&design, &field, 3, growth, (-1, 0), size);
            let right = paint_chunk(&design, &field, 3, growth, (0, 0), size);
            let side = left.side;

            // One window two chunks wide, upper half: must match both chunks exactly.
            let wide = paint_field_window(
                &design,
                &field,
                3,
                growth,
                size,
                Rgba::window(2 * side, -(side as i32), 0),
            );
            for y in 0..side {
                for x in 0..2 * side {
                    let (canvas_x, canvas_y) = (x as i32 - side as i32, y as i32);
                    let chunk = if canvas_x < 0 { &left } else { &right };
                    let expected = chunk.pixels
                        [(canvas_y - chunk.top) as usize * side + (canvas_x - chunk.left) as usize];
                    assert_eq!(
                        wide.pixels[y * 2 * side + x],
                        expected,
                        "{name}: seam at ({canvas_x}, {canvas_y})"
                    );
                }
            }
            assert!(right.coverage() > 0.05, "{name}: empty chunk");
        }
    }
}
