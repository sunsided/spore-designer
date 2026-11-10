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
//! The colour of one colony at one pixel is `body/surface.rs`.

mod outline;
mod surface;

use std::ops::Range;

use super::Colony;
use super::brush::ColonyBrush;
use crate::Rgba;
use crate::params::{ColorStop, MoldParams, sorted};

use outline::{EdgeSample, smooth_min};

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

// ------------------------------------------------------------------------------------------

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

// ------------------------------------------------------------------------------------------

impl Pass<'_> {
    /// Meld the colonies' edges at canvas pixel `(x, y)` and paint body or fringe there.
    fn paint_pixel(
        &self,
        x: i32,
        y: i32,
        samples: &[(&ColonyBody<'_>, EdgeSample)],
        canvas: &mut Rgba,
    ) {
        // Smooth only the two nearest edges: chaining the smooth minimum over every colony
        // would let the bulge grow with the number of neighbours.
        let (mut nearest, mut second) = (f32::MAX, f32::MAX);
        for (_, sample) in samples {
            if sample.signed_distance < nearest {
                (nearest, second) = (sample.signed_distance, nearest);
            } else if sample.signed_distance < second {
                second = sample.signed_distance;
            }
        }
        let melded_distance = smooth_min(nearest, second, self.fillet);

        // Blend weights: 1 for the nearest edge, falling to exactly 0 at `support` beyond it.
        let weight = |sample: &EdgeSample| {
            (1.0 - (sample.signed_distance - nearest) / self.support)
                .max(0.0)
                .powi(2)
        };
        let total_weight: f32 = samples.iter().map(|(_, sample)| weight(sample)).sum();
        let edge_distance = samples
            .iter()
            .map(|(_, sample)| weight(sample) * sample.edge_distance)
            .sum::<f32>()
            / total_weight;

        // Normalised radius of the melded mass: 0 at a centre, 1 on the melded edge.
        let normalized_radius = (1.0 + melded_distance / edge_distance).max(0.0);

        if normalized_radius < 1.0 {
            // Inside: blend every colony's surface by its share.
            let blend = |sum: [f32; 3], color: [f32; 3], share: f32| {
                [
                    sum[0] + color[0] * share,
                    sum[1] + color[1] * share,
                    sum[2] + color[2] * share,
                ]
            };
            let (mut lit, mut albedo) = ([0.0; 3], [0.0; 3]);
            let (mut alpha, mut height, mut gloss) = (0.0, 0.0, 0.0);
            for (body, sample) in samples {
                let share = weight(sample) / total_weight;
                if share > 0.0 {
                    let surface = body.surface(&self.gradient, x, y, sample, normalized_radius);
                    lit = blend(lit, surface.lit, share);
                    albedo = blend(albedo, surface.albedo, share);
                    alpha += surface.alpha * share;
                    height += surface.height * share;
                    gloss += surface.gloss * share;
                }
            }
            canvas.paint_over_split(x, y, lit, albedo, alpha);
            canvas.lift(x, y, height);
            canvas.gloss_over(x, y, gloss, alpha);
        } else if self.design.fringe.enabled && normalized_radius < 1.0 + self.design.fringe.width {
            // Just outside: the fuzzy fringe, blended the same way.
            let alpha: f32 = samples
                .iter()
                .map(|(body, sample)| {
                    weight(sample) / total_weight * body.fringe(x, y, normalized_radius)
                })
                .sum();
            canvas.paint_over(x, y, self.design.fringe.rgb, alpha);
            canvas.lift(x, y, alpha * surface::FRINGE_RELIEF);
        }
    }
}
