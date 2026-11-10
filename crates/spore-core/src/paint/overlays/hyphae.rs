//! Aerial hyphae: thin strands that wander out from the colony as random walks, past its
//! edge too.

use std::f32::consts::TAU;

use crate::Rgba;
use crate::math::scale;
use crate::paint::brush::ColonyBrush;

/// Height a strand adds where it passes, reference pixels (overlapping strands stack up).
const RELIEF: f32 = 0.15;

/// Strand step length, reference pixels.
const STEP: f32 = 0.7;

pub(super) fn paint(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    let hyphae = &brush.design.hyphae;
    let mut rng = brush.rng(2);

    // More strands at larger sizes keep the same density of one-pixel hairs. Each step is
    // drawn as `substeps` pixels so magnified strands stay connected.
    let strand_count = ((hyphae.base + hyphae.growth * brush.growth) * brush.scale.max(1.0)) as u32;
    let substeps = brush.scale.ceil().max(1.0) as u32;

    for _ in 0..strand_count {
        // Start somewhere in the inner 60 %, heading roughly outward.
        let start_angle = rng.range(0.0, TAU);
        let start_distance = rng.range(0.0, 0.6) * brush.radius;
        let (mut x, mut y) = (
            brush.x + start_angle.cos() * start_distance,
            brush.y + start_angle.sin() * start_distance,
        );
        let mut heading = start_angle + rng.range(-0.8, 0.8);
        let length = rng.range(0.5, 1.0) * brush.radius * hyphae.length;

        let mut walked = 0.0;
        while walked < length {
            heading += rng.range(-hyphae.jitter, hyphae.jitter);
            let color = scale(hyphae.rgb, rng.range(1.0 - hyphae.variance, 1.0));

            for _ in 0..substeps {
                x += heading.cos() * STEP / substeps as f32;
                y += heading.sin() * STEP / substeps as f32;

                // Fainter once the strand has left the body.
                let from_center =
                    ((x - brush.x).powi(2) + (y - brush.y).powi(2)).sqrt() / brush.radius;
                let alpha = if from_center > 0.95 {
                    hyphae.outer_alpha
                } else {
                    hyphae.alpha
                };
                let (canvas_x, canvas_y) = (
                    (x * brush.scale).floor() as i32,
                    (y * brush.scale).floor() as i32,
                );
                canvas.paint_over(canvas_x, canvas_y, color, alpha);
                canvas.raise(canvas_x, canvas_y, alpha * RELIEF);
            }
            walked += STEP;
        }
    }
}
