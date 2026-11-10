//! Colour gradients: a list of colours at positions `0..1`, blended in between.

use serde::{Deserialize, Serialize};

use super::Rgb;
use crate::math::mix;

/// One colour of a gradient.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    /// Position along the gradient, `0..1`.
    pub t: f32,
    pub rgb: Rgb,
}

/// Sample a gradient whose stops are sorted by `t` (see [`sorted`]). Below the first stop
/// and above the last, the end colours hold. An empty gradient is white.
pub fn sample(stops: &[ColorStop], t: f32) -> Rgb {
    let Some(first) = stops.first() else {
        return [255.0; 3];
    };
    if t <= first.t {
        return first.rgb;
    }

    for pair in stops.windows(2) {
        let (from, to) = (&pair[0], &pair[1]);
        if t <= to.t {
            let within = (t - from.t) / (to.t - from.t).max(1e-6);
            return mix(from.rgb, to.rgb, within);
        }
    }
    stops[stops.len() - 1].rgb
}

/// A copy of the stops ordered by position, as [`sample`] needs. The editor lets stops be
/// dragged past each other, so painting sorts once before sampling many pixels.
pub fn sorted(stops: &[ColorStop]) -> Vec<ColorStop> {
    let mut ordered = stops.to_vec();
    ordered.sort_by(|a, b| a.t.total_cmp(&b.t));
    ordered
}
