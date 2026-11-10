//! View 1: one colony, centred in its cell.

use super::{Colony, paint_scene, reach};
use crate::params::MoldParams;
use crate::{REFERENCE_SIZE, Rgba};

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
