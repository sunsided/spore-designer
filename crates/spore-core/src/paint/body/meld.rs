//! Melding at one pixel: combine the nearby colonies' edges into one, then paint body (a
//! blend of every colony's surface) or fringe there.

use super::outline::{EdgeSample, smooth_min};
use super::{ColonyBody, Pass, surface};
use crate::Rgba;

impl Pass<'_> {
    /// Meld the colonies' edges at canvas pixel `(x, y)` and paint body or fringe there.
    pub(super) fn paint_pixel(
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
