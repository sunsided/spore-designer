//! How far a design can paint from a colony's centre, so images get enough padding and the
//! endless field knows which colonies can touch a chunk.
//!
//! These are *upper bounds*: a bit too much padding only costs empty pixels, too little cuts
//! the mould off at the image edge (a randomized test in the editor guards this).

use super::{Colony, slime};
use crate::REFERENCE_SIZE;
use crate::params::MoldParams;

/// Anti-aliased discs and dots spill a little past their nominal reach.
pub(super) const EDGE: f32 = 2.0;

/// How far a fully grown colony (without its puddle) reaches from its centre, in reference
/// pixels: the furthest of every layer's reach.
pub(super) fn colony_reach(design: &MoldParams) -> f32 {
    let shape = &design.shape;
    // Every layer grows with the colony radius, so the fully grown radius bounds all stages.
    let radius = (shape.radius_base + shape.radius_growth).max(0.5);

    // Body and fringe: the widest wobble and lobe, plus the fringe band past it.
    let fringe = if design.fringe.enabled {
        design.fringe.width
    } else {
        0.0
    };
    let mut reach =
        radius * (shape.wobble_base + shape.wobble_amp + shape.lobe_amp).max(0.0) * (1.0 + fringe);
    // Melding fills the gap between neighbours; the smooth union bulges by up to meld / 2.
    reach += 0.5 * shape.meld.max(0.0);

    let rosettes = &design.rosettes;
    if rosettes.enabled && rosettes.count > 0 {
        // Rosettes are pulled in to stay within the colony's reference square, unless they
        // are bigger than it; see `overlays/rosettes.rs`.
        let biggest = rosettes.size_max.max(rosettes.size_min) * radius;
        let half = REFERENCE_SIZE as f32 * 0.5 - 1.0;
        reach =
            reach.max((half + 0.05 * biggest).max(1.05 * biggest * (1.0 + rosettes.wobble * 0.5)));
    }

    if design.hyphae.enabled {
        // A strand starts within 0.6 radius and walks at most its length.
        reach = reach.max(radius * (0.6 + design.hyphae.length));
    }

    let sporangia = &design.sporangia;
    if sporangia.enabled {
        reach = reach.max(0.9 * radius + sporangia.size_max.max(sporangia.size_min) + 3.0);
    }

    let drops = &design.drops;
    if drops.enabled {
        reach = reach.max(0.6 * radius + drops.size_max.max(drops.size_min) + 1.0);
    }

    if design.glisten.enabled {
        reach = reach.max(0.8 * radius + 3.0);
    }

    reach + EDGE
}

/// How far the fully grown `colonies` reach past a square `extent` reference pixels wide:
/// `colony_overflow` for the colonies themselves, or further if their joint puddle runs out
/// beyond that. Never negative.
pub(super) fn overflow(
    design: &MoldParams,
    colonies: &[Colony],
    extent: f32,
    colony_overflow: f32,
) -> f32 {
    let mut overflow = colony_overflow;

    // Puddles add up: several neighbours make a film wider than any single puddle, so the
    // actual film of all of them is measured.
    if design.slime.enabled {
        let balls = slime::puddles(&design.slime, colonies);
        if let Some((x0, y0, x1, y1)) = slime::film_extent(&balls, design.slime.ripple) {
            overflow = overflow
                .max(-x0.min(y0) + EDGE)
                .max(x1.max(y1) - extent + EDGE);
        }
    }

    overflow.max(0.0)
}
