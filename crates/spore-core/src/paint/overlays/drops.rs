//! Guttation: droplets of exudate on older colonies, each with a shadow and a highlight.

use std::f32::consts::TAU;

use super::WHITE;
use crate::Rgba;
use crate::math::mix;
use crate::paint::brush::ColonyBrush;

pub(super) fn paint(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    let drops = &brush.design.drops;
    let mut rng = brush.rng(4);

    for _ in 0..(drops.count + brush.seed % 3) {
        let angle = rng.range(0.0, TAU);
        let distance = rng.range(0.2, 0.6) * brush.radius;
        let (x, y) = (
            brush.x + angle.cos() * distance,
            brush.y + angle.sin() * distance,
        );
        let radius = rng.range(drops.size_min, drops.size_max.max(drops.size_min));

        brush.disc(canvas, x + 0.6, y + 0.8, radius, [40.0, 50.0, 30.0], 0.5);
        brush.disc(canvas, x, y, radius, drops.rgb, 1.0);
        brush.dot(
            canvas,
            x + radius * 0.4,
            y - radius * 0.5,
            mix(drops.rgb, WHITE, 0.8),
            1.0,
        );
    }
}
