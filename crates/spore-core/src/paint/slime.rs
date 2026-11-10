//! The slime pass: the mucus puddles under the colonies, as one glossy translucent film.
//! Ported from Sporagon's `slime.rs`.
//!
//! Each colony contributes a few *metaballs* (a central ball plus lobes around it). Each ball
//! has an influence that falls off with distance; where the summed influence (the *field*)
//! reaches 1 there is slime. Because the field is shared, neighbouring puddles flow into one.
//!
//! - `slime/film.rs`: the field itself, and how far the film can reach (for image padding).
//! - `slime/buckets.rs`: spatial lookup so each pixel only sums nearby balls (endless field).
//! - `slime/shading.rs`: the film's colour and coverage at one pixel.

mod buckets;
mod film;
mod shading;

use std::f32::consts::TAU;

use super::Colony;
use crate::Rgba;
use crate::noise::hash;
use crate::params::Slime;

use buckets::BallBuckets;
pub(super) use film::film_extent;

/// A metaball, in reference pixels.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ball {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

// ------------------------------------------------------------------------------------------
// Puddles

/// The balls of every colony's puddle, colony by colony.
pub(super) fn puddles(slime: &Slime, colonies: &[Colony]) -> Vec<Ball> {
    colonies
        .iter()
        .flat_map(|colony| puddle(slime, colony))
        .collect()
}

/// One colony's puddle: a central ball plus `lobes` balls evenly around it (with a little
/// random twist), all scaled by the colony's growth.
fn puddle(slime: &Slime, colony: &Colony) -> Vec<Ball> {
    let (seed, growth) = (colony.seed, colony.growth);
    let mut balls = vec![Ball {
        x: colony.x,
        y: colony.y,
        radius: slime.size * growth,
    }];

    for lobe in 0..slime.lobes as i32 {
        let angle = lobe as f32 / slime.lobes as f32 * TAU + hash(lobe, 0, seed) * 0.8;
        let distance = (slime.reach + slime.reach_jitter * hash(lobe, 1, seed)) * growth;
        balls.push(Ball {
            x: colony.x + angle.cos() * distance,
            y: colony.y + angle.sin() * distance,
            radius: (slime.lobe_size + slime.lobe_jitter * hash(lobe, 2, seed)) * growth,
        });
    }
    balls
}

/// How far each ball's influence reaches in the endless field (reference pixels). Past it a
/// ball contributes nothing, so a chunk only needs colonies within reach of its pixels.
pub(super) fn field_cutoff(slime: &Slime) -> f32 {
    3.0 * slime.size.max(slime.lobe_size + slime.lobe_jitter) + 1.0
}

// ------------------------------------------------------------------------------------------
// Painting

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
        None => match film::canvas_bounds(balls, slime.ripple, scale, canvas) {
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
