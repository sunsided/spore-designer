//! Placing a patch's colonies on a square.

use super::delayed_growth;
use crate::noise::Rng;
use crate::paint::Colony;
use crate::params::Patch;

/// `patch.count` colonies on a square `extent` reference pixels wide, centres at least
/// `margin` from its edges.
///
/// Positions use *best-candidate sampling*: for each colony try 12 random spots and keep the
/// one farthest from the colonies already placed. That spreads them evenly without a grid.
/// Positions do not depend on `overall` growth, only which colonies have started does.
pub(in crate::paint) fn place_in_patch(
    patch: &Patch,
    seed: u32,
    extent: f32,
    margin: f32,
    overall: f32,
) -> Vec<Colony> {
    let mut rng = Rng(0xC2B2_AE3D_27D4_EB4F ^ (seed as u64).wrapping_mul(104_729));
    let mut placed: Vec<Colony> = Vec::new();

    for index in 0..patch.count {
        let mut best = (extent * 0.5, extent * 0.5, -1.0);
        for _ in 0..12 {
            let (x, y) = (
                rng.range(margin, extent - margin),
                rng.range(margin, extent - margin),
            );
            let nearest_squared = placed
                .iter()
                .map(|other| (other.x - x).powi(2) + (other.y - y).powi(2))
                .fold(f32::MAX, f32::min);
            if nearest_squared > best.2 {
                best = (x, y, nearest_squared);
            }
        }

        let delay = rng.unit() * patch.stagger.clamp(0.0, 0.95);
        placed.push(Colony {
            x: best.0,
            y: best.1,
            growth: delayed_growth(overall, delay),
            seed: seed.wrapping_add(index * 7919),
        });
    }

    placed.retain(|colony| colony.growth > 0.0);
    placed
}
