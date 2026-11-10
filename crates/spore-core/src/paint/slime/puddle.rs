//! Each colony's puddle as metaballs.

use std::f32::consts::TAU;

use super::Ball;
use crate::noise::hash;
use crate::paint::Colony;
use crate::params::Slime;

/// The balls of every colony's puddle, colony by colony.
pub(in crate::paint) fn puddles(slime: &Slime, colonies: &[Colony]) -> Vec<Ball> {
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
pub(in crate::paint) fn field_cutoff(slime: &Slime) -> f32 {
    3.0 * slime.size.max(slime.lobe_size + slime.lobe_jitter) + 1.0
}
