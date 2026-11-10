//! The body pass: every colony body in the scene, painted together as one melded mass.
//!
//! # How melding works
//!
//! For each pixel, every nearby colony reports its *signed distance* to its own wobbly edge
//! (negative inside, see `body/outline.rs`). The melded edge is a *smooth minimum* of the two
//! nearest distances: like `min`, but where two edges come within the fillet size of each
//! other it dips below both, which grows a neck that fuses the colonies.
//!
//! Colour, texture and fringe then blend between the colonies by how close each one's edge
//! is (weights with compact support, so far colonies contribute exactly zero and field
//! chunks stay seamless). With a single colony all of this reduces to that colony's body.
//!
//! - `body/outline.rs`: one colony's wobbly edge, and the smooth minimum.
//! - `body/meld.rs`: melding the edges at one pixel and blending the colonies there.
//! - `body/surface.rs`: one colony's colour, height and gloss at one pixel.

mod meld;
mod outline;
mod surface;

use std::ops::Range;

use super::Colony;
use super::brush::ColonyBrush;
use crate::Rgba;
use crate::params::{ColorStop, MoldParams, sorted};

use outline::EdgeSample;

/// One colony's body, with what is the same for all of its pixels worked out up front.
struct ColonyBody<'a> {
    brush: ColonyBrush<'a>,
    /// Random rotation of the outline (when the shape spins).
    rotation: f32,
    /// Furrows around the rim: the design's count plus a per-seed extra.
    furrow_count: f32,
    /// Half-size, in reference pixels, of the square outside which this colony cannot
    /// affect a pixel.
    reach: f32,
}

/// What every pixel of the pass shares.
struct Pass<'a> {
    design: &'a MoldParams,
    /// The body gradient, sorted for sampling.
    gradient: Vec<ColorStop>,
    /// Smooth-minimum fillet size (reference pixels).
    fillet: f32,
    /// Blend weights reach zero this far past the nearest edge.
    support: f32,
}

/// Paint the bodies of all `colonies` as one melded mass. Edges up to `shape.meld` apart
/// close; see the top of this file.
pub(super) fn paint_bodies(
    design: &MoldParams,
    scale: f32,
    colonies: &[Colony],
    canvas: &mut Rgba,
) {
    if colonies.is_empty() {
        return;
    }

    // Fillet size: two edges `meld` apart just touch (the smooth min dips by fillet / 4
    // midway).
    let fillet = 2.0 * design.shape.meld.max(0.0);
    let pass = Pass {
        design,
        gradient: sorted(&design.body.gradient),
        fillet,
        support: fillet + 1.0,
    };
    let bodies: Vec<ColonyBody<'_>> = colonies
        .iter()
        .map(|colony| ColonyBody::new(ColonyBrush::new(design, colony, scale), fillet))
        .collect();

    let (columns, rows) = covered_span(&bodies, canvas);
    let mut samples: Vec<(&ColonyBody<'_>, EdgeSample)> = Vec::new();
    for y in rows {
        for x in columns.clone() {
            samples.clear();
            samples.extend(
                bodies
                    .iter()
                    .filter_map(|body| body.sample(x, y).map(|sample| (body, sample))),
            );
            if !samples.is_empty() {
                pass.paint_pixel(x, y, &samples, canvas);
            }
        }
    }
}

/// The canvas pixels any of the bodies can touch: the union of their reach squares.
fn covered_span(bodies: &[ColonyBody<'_>], canvas: &Rgba) -> (Range<i32>, Range<i32>) {
    bodies
        .iter()
        .map(|body| body.brush.span(canvas, body.reach))
        .reduce(|(columns_a, rows_a), (columns_b, rows_b)| {
            (
                columns_a.start.min(columns_b.start)..columns_a.end.max(columns_b.end),
                rows_a.start.min(rows_b.start)..rows_a.end.max(rows_b.end),
            )
        })
        .expect("at least one colony")
}
