//! The slime pass: the mucus puddles under the colonies, as one glossy translucent film.
//! Ported from Sporagon's `slime.rs`.
//!
//! Each colony contributes a few *metaballs* (a central ball plus lobes around it). Each ball
//! has an influence that falls off with distance; where the summed influence (the *field*)
//! reaches 1 there is slime. Because the field is shared, neighbouring puddles flow into one.
//!
//! - `slime/puddle.rs`: the balls of each colony's puddle.
//! - `slime/film.rs`: the field itself.
//! - `slime/extent.rs`: how far the film can reach (for image padding).
//! - `slime/buckets.rs`: spatial lookup so each pixel only sums nearby balls (endless field).
//! - `slime/shading.rs`: the film's colour and coverage at one pixel.

mod buckets;
mod extent;
mod film;
mod puddle;
mod shading;

use crate::Rgba;
use crate::params::Slime;

use buckets::BallBuckets;
pub(super) use extent::film_extent;
pub(super) use puddle::{field_cutoff, puddles};

/// A metaball, in reference pixels.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ball {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

/// Paint the film of `balls` into `canvas`, `scale` output pixels per reference pixel.
///
/// With a `cutoff` (endless field) the whole window is painted and each pixel only sums the
/// balls in neighbouring buckets. Without one (single colony, patch), only the film's
/// bounding box is painted and every pixel sums every ball.
pub(super) fn paint(
    slime: &Slime,
    balls: &[Ball],
    scale: f32,
    cutoff: Option<f32>,
    canvas: &mut Rgba,
) {
    let (columns, rows) = match cutoff {
        Some(_) => (canvas.columns(), canvas.rows()),
        None => match extent::canvas_bounds(balls, slime.ripple, scale, canvas) {
            Some(bounds) => bounds,
            None => return,
        },
    };

    // A bucket one cutoff (plus the 0.8 normal-sampling step) wide: the 3x3 buckets around
    // a pixel then hold every ball that can reach it.
    let buckets = cutoff.map(|cutoff| BallBuckets::new(balls, cutoff + 1.0));
    let mut nearby = Vec::new();

    for y in rows {
        for x in columns.clone() {
            let (reference_x, reference_y) = ((x as f32 + 0.5) / scale, (y as f32 + 0.5) / scale);
            let local_balls = match &buckets {
                Some(buckets) => {
                    buckets.gather(reference_x, reference_y, &mut nearby);
                    &nearby[..]
                }
                None => balls,
            };
            if local_balls.is_empty() {
                continue;
            }

            if let Some(film) = shading::shade(
                slime,
                local_balls,
                cutoff,
                (x, y),
                (reference_x, reference_y),
            ) {
                canvas.paint_split(
                    x,
                    y,
                    (film.lit, film.lit_alpha),
                    (film.albedo, film.albedo_alpha),
                );
                canvas.lift(x, y, film.height);
                canvas.gloss_over(x, y, film.gloss, film.albedo_alpha);
            }
        }
    }
}
