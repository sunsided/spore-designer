//! Sporangia: spore heads on short slanted stalks, with shadows. Ripe ones are dark, unripe
//! ones pale; more ripen as the colony grows.

use std::f32::consts::TAU;

use super::WHITE;
use crate::Rgba;
use crate::math::mix;
use crate::paint::brush::ColonyBrush;

pub(super) fn paint(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    let sporangia = &brush.design.sporangia;
    let growth = brush.growth;
    let mut rng = brush.rng(3);

    for _ in 0..(sporangia.count * growth * growth) as u32 {
        let angle = rng.range(0.0, TAU);
        let distance = rng.unit().sqrt() * 0.9 * brush.radius;
        let (x, y) = (
            brush.x + angle.cos() * distance,
            brush.y + angle.sin() * distance,
        );
        let radius = rng.range(
            sporangia.size_min,
            sporangia.size_max.max(sporangia.size_min),
        );
        let ripe = rng.unit() < growth * sporangia.ripe_chance;

        // The stalk, seen at a slant: three dots leaning down to the left.
        for step in 0..3 {
            let step = step as f32;
            brush.dot(
                canvas,
                x - 0.5 - step * 0.4,
                y + 1.0 + step,
                sporangia.stalk,
                0.7,
            );
        }

        // Shadow, head, highlight.
        brush.disc(canvas, x + 1.2, y + 1.5, radius, [50.0, 48.0, 44.0], 0.35);
        let head = if ripe {
            sporangia.ripe
        } else {
            sporangia.unripe
        };
        brush.disc(canvas, x, y, radius, head, 1.0);
        brush.dot(
            canvas,
            x + radius * 0.35,
            y - radius * 0.45,
            mix(head, WHITE, 0.45),
            0.9,
        );
    }
}
