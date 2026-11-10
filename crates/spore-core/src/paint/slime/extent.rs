//! How far the slime film can reach: bounds for image padding and for painting.

use std::ops::Range;

use super::Ball;
use crate::Rgba;

/// Upper bound on how far the film reaches from the nearest ball. The field is at most
/// `(1 + ripple / 2) * sum(radius²) / distance²` for the distance to the nearest ball, so
/// past this distance it stays below 1 everywhere.
fn film_radius(sum_radius_squared: f32, ripple: f32) -> f32 {
    (sum_radius_squared * (1.0 + ripple * 0.5)).sqrt() + 1.0
}

/// Bounding box of the film in reference pixels, `(x0, y0, x1, y1)`, or `None` without balls.
///
/// [`film_radius`] alone is far too generous for big puddles, so the box it gives is sampled
/// on a one-pixel grid (assuming the strongest possible ripple) and one and a half safety
/// pixels are added around what was found.
pub(in crate::paint) fn film_extent(balls: &[Ball], ripple: f32) -> Option<(f32, f32, f32, f32)> {
    let balls: Vec<Ball> = balls
        .iter()
        .copied()
        .filter(|ball| ball.radius > 0.0)
        .collect();

    // The analytic bound: a box around all balls, widened by the film radius.
    let margin = film_radius(
        balls.iter().map(|ball| ball.radius * ball.radius).sum(),
        ripple,
    );
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for ball in &balls {
        (x0, y0) = (x0.min(ball.x - margin), y0.min(ball.y - margin));
        (x1, y1) = (x1.max(ball.x + margin), y1.max(ball.y + margin));
    }
    if x0 > x1 {
        return None;
    }

    // Sample it: where could the field reach 1 with the ripple at its peak?
    let peak_ripple = 1.0 + ripple * 0.5;
    let mut found = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            let sum: f32 = balls
                .iter()
                .map(|ball| {
                    ball.radius * ball.radius / ((x - ball.x).powi(2) + (y - ball.y).powi(2) + 0.25)
                })
                .sum();
            if sum * peak_ripple >= 1.0 {
                found = (
                    found.0.min(x),
                    found.1.min(y),
                    found.2.max(x),
                    found.3.max(y),
                );
            }
            x += 1.0;
        }
        y += 1.0;
    }

    (found.0 <= found.2).then_some((found.0 - 1.5, found.1 - 1.5, found.2 + 1.5, found.3 + 1.5))
}

/// Canvas pixel columns and rows of the film, clipped to the window.
pub(super) fn canvas_bounds(
    balls: &[Ball],
    ripple: f32,
    scale: f32,
    canvas: &Rgba,
) -> Option<(Range<i32>, Range<i32>)> {
    let (x0, y0, x1, y1) = film_extent(balls, ripple)?;
    let clip = |low: f32, high: f32, window: Range<i32>| {
        let first = ((low * scale).floor() as i32).max(window.start);
        first
            ..((high * scale).ceil() as i32 + 1)
                .min(window.end)
                .max(first)
    };
    Some((clip(x0, x1, canvas.columns()), clip(y0, y1, canvas.rows())))
}
