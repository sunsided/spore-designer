//! Placing the endless field's colonies on a jittered grid.

use super::delayed_growth;
use crate::noise::hash;
use crate::paint::Colony;
use crate::params::Field;

/// Every colony of the endless field that can touch the window `(x0, y0, x1, y1)` (reference
/// pixels), given that a colony reaches `reach` from its centre.
///
/// The field is a grid of cells `field.spacing` colony widths wide; each cell holds a colony
/// with probability `field.density`, pushed off-centre by up to `field.jitter` of a cell.
/// Everything about a cell's colony comes from hashing the cell's grid position, so any
/// window finds the same colonies (in the same row-major order) without shared state.
pub(in crate::paint) fn field_colonies(
    field: &Field,
    seed: u32,
    overall: f32,
    (x0, y0, x1, y1): (f32, f32, f32, f32),
    reach: f32,
) -> Vec<Colony> {
    let cell = crate::REFERENCE_SIZE as f32 * field.spacing.max(0.2);
    // Cells whose colony could reach into `low..high` along one axis (one spare cell each
    // side covers the jitter).
    let cells = |low: f32, high: f32| {
        ((low - reach) / cell).floor() as i32 - 1..=((high + reach) / cell).ceil() as i32
    };

    let mut colonies = Vec::new();
    for row in cells(y0, y1) {
        for column in cells(x0, x1) {
            // Independent random values for this cell, one per `key`.
            let random = |key: u32| hash(column, row, seed.wrapping_add(key * 977));
            if random(0) >= field.density {
                continue;
            }

            let x = (column as f32 + 0.5 + (random(1) - 0.5) * field.jitter) * cell;
            let y = (row as f32 + 0.5 + (random(2) - 0.5) * field.jitter) * cell;
            let delay = random(3) * field.stagger.clamp(0.0, 0.95);
            let growth = delayed_growth(overall, delay);
            if growth > 0.0 {
                colonies.push(Colony {
                    x,
                    y,
                    growth,
                    seed: (random(4) * 16_777_216.0) as u32 ^ seed,
                });
            }
        }
    }
    colonies
}
