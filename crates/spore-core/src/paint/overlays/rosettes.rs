//! Lichen rosettes: small banded sub-colonies scattered over and around the body, the
//! colourful "Annihilation" look.
//!
//! Each rosette is born at a random growth stage, then spreads with the colony. Its rings run
//! back and forth along the gradient (a triangle wave), so bands stay smooth instead of
//! jumping from the last colour back to the first.

use std::f32::consts::TAU;

use crate::math::{lerp, scale, shift_hue, smooth};
use crate::noise::fbm;
use crate::paint::brush::{ColonyBrush, canvas_span};
use crate::paint::lighting::dome;
use crate::params::{Rgb, sample, sorted};
use crate::{REFERENCE_SIZE, Rgba};

/// One rosette, placed and sized for the current growth.
struct Rosette {
    /// Centre and current radius, reference pixels.
    x: f32,
    y: f32,
    radius: f32,
    /// Offset into the ring bands, and hue shift, both random per rosette.
    phase: f32,
    hue: f32,
    seed: u32,
}

pub(super) fn paint(brush: &ColonyBrush<'_>, canvas: &mut Rgba) {
    let rosettes = &brush.design.rosettes;
    let gradient = sorted(&rosettes.gradient);
    let mut rng = brush.rng(1);

    for index in 0..rosettes.count {
        // Draw all random values first, so every rosette consumes the same amount of the
        // stream whether or not it is visible yet.
        let angle = rng.range(0.0, TAU);
        let distance = rng.unit().sqrt() * rosettes.spread * brush.radius;
        let size = rng.range(rosettes.size_min, rosettes.size_max.max(rosettes.size_min));
        let born = rng.range(0.0, 0.6);
        let phase = rng.unit();
        let hue = rng.range(-1.0, 1.0) * rosettes.hue_jitter;

        let own_growth = ((brush.growth - born) / (1.0 - born)).clamp(0.0, 1.0);
        let radius = size * brush.radius * own_growth;
        if radius < 0.5 {
            continue;
        }

        // Keep the fully grown rosette inside the colony's reference square.
        let max_distance =
            (REFERENCE_SIZE as f32 * 0.5 - 1.0 - size * brush.radius * (1.0 + rosettes.wobble))
                .max(0.0);
        let distance = distance.min(max_distance);

        let rosette = Rosette {
            x: brush.x + angle.cos() * distance,
            y: brush.y + angle.sin() * distance,
            radius,
            phase,
            hue,
            seed: brush.seed.wrapping_add(31 + index * 17),
        };
        paint_rosette(brush, &rosette, &gradient, canvas);
    }
}

/// Paint one rosette pixel by pixel.
fn paint_rosette(
    brush: &ColonyBrush<'_>,
    rosette: &Rosette,
    gradient: &[crate::params::ColorStop],
    canvas: &mut Rgba,
) {
    let design = brush.design;
    let rosettes = &design.rosettes;
    let lighting = &design.lighting;
    let scale_factor = brush.scale;

    let reach = rosette.radius * (1.0 + rosettes.wobble) * 1.1;
    let (columns, rows) = canvas_span(canvas, scale_factor, (rosette.x, rosette.y), reach);

    for y in rows.clone() {
        for x in columns.clone() {
            // Position relative to the rosette, and its own wobbly edge in that direction.
            let (dx, dy) = (
                (x as f32 + 0.5) / scale_factor - rosette.x,
                (y as f32 + 0.5) / scale_factor - rosette.y,
            );
            let angle = dy.atan2(dx);
            let wobble = fbm(
                angle.cos() * 1.5 + 3.0,
                angle.sin() * 1.5 + 3.0,
                rosette.seed,
                2,
            );
            let edge = rosette.radius * (1.0 - rosettes.wobble * 0.5 + rosettes.wobble * wobble);
            let normalized_radius = (dx * dx + dy * dy).sqrt() / edge.max(0.1);
            if normalized_radius >= 1.05 {
                continue;
            }

            // Ring colour: bands broken up by blotch noise, back and forth along the gradient.
            let (ux, uy) = (x as f32 / scale_factor, y as f32 / scale_factor);
            let blotch = fbm(ux / 2.5, uy / 2.5, rosette.seed + 3, 3);
            let band = normalized_radius * rosettes.rings
                + rosette.phase
                + (blotch - 0.5) * rosettes.blotch;
            let triangle = 1.0 - (2.0 * band.fract() - 1.0).abs();
            let mut color: Rgb = sample(gradient, triangle);
            if rosette.hue != 0.0 {
                color = shift_hue(color, rosette.hue, 1.0);
            }

            // Darker rim, a gentler dome than the body, and the body's grain.
            let grain = fbm(
                ux / lighting.grain_scale,
                uy / lighting.grain_scale,
                rosette.seed,
                2,
            );
            let brightness = (1.0 - rosettes.rim_dark * smooth(0.6, 1.0, normalized_radius))
                * lerp(
                    1.0,
                    dome(dx / edge, dy / edge, normalized_radius.min(1.0)),
                    lighting.dome * 0.6,
                )
                * (1.0 - lighting.grain * 0.5 + lighting.grain * grain);

            // Soft, fuzzy rim (output-pixel noise, so it stays fine at any size).
            let fuzz = fbm(x as f32 / 1.3, y as f32 / 1.3, rosette.seed + 7, 2);
            let alpha = 1.0 - smooth(0.9, 1.05, normalized_radius) * (0.4 + 0.6 * (1.0 - fuzz));
            canvas.paint_over(
                x,
                y,
                scale(color, brightness),
                alpha.max(0.0) * rosettes.alpha,
            );
        }
    }
}
