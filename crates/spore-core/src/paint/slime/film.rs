//! The metaball field.

use super::Ball;
use crate::math::smooth;
use crate::noise::fbm;

/// Metaball field at reference position `(x, y)`: the sum of `radius² / distance²` over all
/// balls, roughened by ripple noise. Slime is where this reaches 1.
///
/// With a `cutoff`, each ball fades out smoothly by that distance, so a pixel only depends
/// on nearby balls (what an endless canvas needs).
pub(super) fn field(balls: &[Ball], ripple: f32, cutoff: Option<f32>, x: f32, y: f32) -> f32 {
    let sum: f32 = balls
        .iter()
        .map(|ball| {
            let distance_squared = (x - ball.x).powi(2) + (y - ball.y).powi(2);
            let fade = cutoff.map_or(1.0, |cutoff| {
                1.0 - smooth(0.6 * cutoff, cutoff, distance_squared.sqrt())
            });
            fade * ball.radius * ball.radius / (distance_squared + 0.25)
        })
        .sum();

    // Ripples: lumpy edges and a surface that catches the light in many small glints.
    sum * (1.0 - ripple * 0.5 + ripple * fbm(x / 4.5, y / 4.5, 5, 2))
}
