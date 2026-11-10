//! View 3: an endless field, painted chunk by chunk.

use super::{paint_scene, placement, reach, slime};
use crate::params::{Field, MoldParams};
use crate::{CHUNK_CELLS, REFERENCE_SIZE, Rgba};

/// Side of a field chunk in reference pixels.
const CHUNK: f32 = (CHUNK_CELLS * REFERENCE_SIZE) as f32;

/// Chunk `(column, row)` of an endless field, `size` pixels per cell, plus `margin` pixels of
/// its neighbours on every side. Each pixel only depends on its canvas position, so chunks
/// tile without seams.
pub(crate) fn paint_chunk(
    design: &MoldParams,
    field: &Field,
    seed: u32,
    growth: f32,
    (column, row): (i32, i32),
    size: usize,
    margin: usize,
) -> Rgba {
    let side = (CHUNK * size as f32 / REFERENCE_SIZE as f32).round() as usize;
    let window = Rgba::window(
        side + 2 * margin,
        column * side as i32 - margin as i32,
        row * side as i32 - margin as i32,
    );
    paint_field_window(design, field, seed, growth, size, window)
}

/// Paint the field into any window of the canvas, `size` pixels per cell.
pub(super) fn paint_field_window(
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
