//! [`ColonyBrush`]: everything a painting pass needs to know about one colony, plus drawing
//! helpers that take reference-pixel coordinates.

use std::ops::Range;

use super::Colony;
use crate::Rgba;
use crate::noise::Rng;
use crate::params::MoldParams;

/// One colony, ready to paint.
pub(super) struct ColonyBrush<'a> {
    pub design: &'a MoldParams,
    pub seed: u32,
    pub growth: f32,
    /// Output pixels per reference pixel.
    pub scale: f32,
    /// Centre and current radius, in reference pixels.
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl<'a> ColonyBrush<'a> {
    pub fn new(design: &'a MoldParams, colony: &Colony, scale: f32) -> Self {
        let shape = &design.shape;
        Self {
            design,
            seed: colony.seed,
            growth: colony.growth,
            scale,
            x: colony.x,
            y: colony.y,
            radius: (shape.radius_base + shape.radius_growth * colony.growth).max(0.5),
        }
    }

    /// An independent random stream per painting layer, so changing one layer's counts (or
    /// the output size) leaves the others where they were.
    pub fn rng(&self, layer: u64) -> Rng {
        Rng(0x9E37_79B9_7F4A_7C15 ^ (self.seed as u64).wrapping_mul(7919) ^ (layer << 48))
    }

    /// Canvas pixel columns and rows within `reach` reference pixels of the centre.
    pub fn span(&self, canvas: &Rgba, reach: f32) -> (Range<i32>, Range<i32>) {
        canvas_span(canvas, self.scale, (self.x, self.y), reach)
    }

    /// Anti-aliased disc at reference position `(x, y)`.
    pub fn disc(
        &self,
        canvas: &mut Rgba,
        x: f32,
        y: f32,
        radius: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        canvas.fill_disc(
            x * self.scale,
            y * self.scale,
            radius * self.scale,
            color,
            alpha,
        );
    }

    /// Anti-aliased disc of pure lighting (a cast shadow) at reference position `(x, y)`;
    /// only the lit picture changes, not the albedo.
    pub fn light_disc(
        &self,
        canvas: &mut Rgba,
        x: f32,
        y: f32,
        radius: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        canvas.fill_light_disc(
            x * self.scale,
            y * self.scale,
            radius * self.scale,
            color,
            alpha,
        );
    }

    /// One reference pixel of surface colour at reference position `(x, y)`: a single canvas
    /// pixel at small scales, a round dot of the same size when magnified.
    pub fn dot(&self, canvas: &mut Rgba, x: f32, y: f32, color: [f32; 3], alpha: f32) {
        if self.scale <= 1.5 {
            canvas.paint_over(
                (x * self.scale).floor() as i32,
                (y * self.scale).floor() as i32,
                color,
                alpha,
            );
        } else {
            canvas.fill_disc(
                x * self.scale,
                y * self.scale,
                0.5 * self.scale,
                color,
                alpha,
            );
        }
    }

    /// Like [`Self::dot`], but pure lighting (a highlight): only the lit picture changes.
    pub fn light_dot(&self, canvas: &mut Rgba, x: f32, y: f32, color: [f32; 3], alpha: f32) {
        if self.scale <= 1.5 {
            canvas.paint_light(
                (x * self.scale).floor() as i32,
                (y * self.scale).floor() as i32,
                color,
                alpha,
            );
        } else {
            canvas.fill_light_disc(
                x * self.scale,
                y * self.scale,
                0.5 * self.scale,
                color,
                alpha,
            );
        }
    }

    /// A disc of shininess `gloss` (`0..1`) at reference position `(x, y)`.
    pub fn gloss_disc(&self, canvas: &mut Rgba, x: f32, y: f32, radius: f32, gloss: f32) {
        canvas.gloss_disc(x * self.scale, y * self.scale, radius * self.scale, gloss);
    }

    /// A hemisphere `peak` reference pixels high and `radius` reference pixels wide at
    /// reference position `(x, y)`, added on top of what is there.
    pub fn bump(&self, canvas: &mut Rgba, x: f32, y: f32, radius: f32, peak: f32) {
        canvas.raise_dome(x * self.scale, y * self.scale, radius * self.scale, peak);
    }
}

/// Canvas pixel columns and rows of `canvas` within `reach` reference pixels of `center`
/// (a square, clipped to the window).
pub(super) fn canvas_span(
    canvas: &Rgba,
    scale: f32,
    (center_x, center_y): (f32, f32),
    reach: f32,
) -> (Range<i32>, Range<i32>) {
    let clip = |center: f32, window: Range<i32>| {
        let low = (((center - reach) * scale).floor() as i32).max(window.start);
        let high = (((center + reach) * scale).ceil() as i32 + 1).min(window.end);
        low..high.max(low)
    };
    (
        clip(center_x, canvas.columns()),
        clip(center_y, canvas.rows()),
    )
}
