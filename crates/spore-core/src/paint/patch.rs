//! View 2: a patch of colonies on one square.

use super::{Colony, paint_scene, placement, reach};
use crate::params::{MoldParams, Patch};
use crate::{REFERENCE_SIZE, Rgba};

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
