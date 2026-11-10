//! Wet look: a glossy sheen on the side of the colony facing the light, and a few beads of
//! clear liquid. The sheen is pure lighting (the specular map carries the wetness instead);
//! the beads are real bumps with their own shadow and highlight.

use std::f32::consts::TAU;

use super::WHITE;
use crate::Rgba;
use crate::math::smooth;
use crate::noise::vnoise;
use crate::paint::brush::ColonyBrush;

pub(super) fn paint(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    paint_sheen(brush, canvas);
    paint_beads(brush, canvas);
}

/// Specular highlight on the dome, broken up by noise, only where the body is solid.
fn paint_sheen(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    let glisten = &brush.design.glisten;
    let (scale, radius) = (brush.scale, brush.radius);

    let (columns, rows) = brush.span(canvas, radius + 1.0);
    for y in rows {
        for x in columns.clone() {
            let (dx, dy) = (
                ((x as f32 + 0.5) / scale - brush.x) / radius,
                ((y as f32 + 0.5) / scale - brush.y) / radius,
            );
            let normalized_radius = (dx * dx + dy * dy).sqrt();
            if normalized_radius >= 0.95 || canvas.alpha(x, y) <= 0.9 {
                continue;
            }

            // Blinn-Phong: the half-vector between light and viewer against the dome normal.
            let height = (1.0 - normalized_radius * normalized_radius)
                .max(0.0)
                .sqrt();
            let half = (dx * 0.8 * 0.45 - dy * 0.8 * 0.6 + height * 1.66) / 1.79;
            let breakup = vnoise(
                x as f32 / scale / 3.0,
                y as f32 / scale / 3.0,
                brush.seed + 40,
            );
            let sheen =
                half.max(0.0).powf(glisten.power) * glisten.sheen * smooth(0.3, 0.7, breakup);
            canvas.paint_light(x, y, [250.0, 252.0, 240.0], sheen);
        }
    }
}

/// Clear beads: a faint shadow, a translucent body and a bright highlight each.
fn paint_beads(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    let glisten = &brush.design.glisten;
    let mut rng = brush.rng(5);

    for _ in 0..(glisten.beads_base + glisten.beads_growth * brush.growth) as u32 {
        let angle = rng.range(0.0, TAU);
        let distance = rng.unit().sqrt() * 0.8 * brush.radius;
        let (x, y, radius) = (
            brush.x + angle.cos() * distance,
            brush.y + angle.sin() * distance,
            rng.range(0.9, 2.0),
        );

        brush.light_disc(canvas, x + 0.7, y + 0.9, radius, [20.0, 24.0, 18.0], 0.3);
        brush.disc(canvas, x, y, radius, [210.0, 220.0, 205.0], 0.35);
        brush.bump(canvas, x, y, radius, radius * 0.6);
        brush.gloss_disc(canvas, x, y, radius, 1.0);
        brush.light_dot(canvas, x + radius * 0.35, y - radius * 0.45, WHITE, 1.0);
    }
}
