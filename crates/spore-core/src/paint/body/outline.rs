//! The shape of one colony's body: its wobbly, optionally lobed edge, measured from a pixel.

use std::f32::consts::TAU;

use super::ColonyBody;
use crate::noise::fbm;
use crate::paint::brush::ColonyBrush;

/// One colony's edge as seen from one pixel.
pub(super) struct EdgeSample {
    /// Pixel position relative to the colony centre (reference pixels).
    pub dx: f32,
    pub dy: f32,
    /// Direction from the centre, including the colony's rotation.
    pub angle: f32,
    /// Angular noise `0..1` that pushes the edge in and out in this direction.
    pub wobble: f32,
    /// Distance from the centre to the edge in this direction.
    pub edge_distance: f32,
    /// Distance from the pixel to that edge; negative inside.
    pub signed_distance: f32,
}

impl<'a> ColonyBody<'a> {
    /// Prepare `brush`'s colony for a body pass melding with fillet size `fillet`.
    pub(super) fn new(brush: ColonyBrush<'a>, fillet: f32) -> Self {
        let design = brush.design;
        let shape = &design.shape;

        let rotation = if shape.spin {
            brush.rng(0).range(0.0, TAU)
        } else {
            0.0
        };
        let furrow_count = design.furrows.count + (brush.seed % 4) as f32 * 2.0;

        // Painted pixels lie within the fringe band of the melded edge, and a colony only
        // influences one within `fillet + 1` of the nearest edge; see `body.rs`.
        let fringe = if design.fringe.enabled {
            design.fringe.width
        } else {
            0.0
        };
        let reach =
            brush.radius * (shape.wobble_base + shape.wobble_amp + shape.lobe_amp) * (1.0 + fringe)
                + 1.25 * fillet
                + 3.0;

        Self {
            brush,
            rotation,
            furrow_count,
            reach,
        }
    }

    /// The edge seen from canvas pixel `(x, y)`, or `None` if the pixel is out of reach.
    pub(super) fn sample(&self, x: i32, y: i32) -> Option<EdgeSample> {
        let brush = &self.brush;
        let (dx, dy) = (
            (x as f32 + 0.5) / brush.scale - brush.x,
            (y as f32 + 0.5) / brush.scale - brush.y,
        );
        if dx.abs() > self.reach || dy.abs() > self.reach {
            return None;
        }

        let shape = &brush.design.shape;
        let angle = dy.atan2(dx) + self.rotation;

        // Noise sampled on a circle, so the outline wobbles smoothly and closes on itself.
        let wobble = fbm(
            angle.cos() * shape.wobble_freq + 5.0,
            angle.sin() * shape.wobble_freq + 5.0,
            brush.seed,
            3,
        );
        let lobe = shape.lobe_amp * (angle * shape.lobes as f32).cos();
        let edge_distance =
            (brush.radius * (shape.wobble_base + shape.wobble_amp * wobble + lobe)).max(0.1);
        let signed_distance = (dx * dx + dy * dy).sqrt() - edge_distance;

        Some(EdgeSample {
            dx,
            dy,
            angle,
            wobble,
            edge_distance,
            signed_distance,
        })
    }
}

/// Polynomial smooth minimum: `min(a, b)`, except where `a` and `b` are within `fillet` of
/// each other it dips below both (by up to `fillet / 4`), rounding the corner between them.
pub(super) fn smooth_min(a: f32, b: f32, fillet: f32) -> f32 {
    if fillet <= 0.0 {
        return a.min(b);
    }
    let closeness = (fillet - (a - b).abs()).max(0.0) / fillet;
    a.min(b) - closeness * closeness * fillet * 0.25
}
