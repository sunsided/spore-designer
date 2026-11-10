//! [`Rgba`]: the image every painting pass draws into.

use std::ops::Range;

// ------------------------------------------------------------------------------------------
// The image type

/// A square true-colour image with coverage: RGB channels in `0..255`, alpha in `0..1`.
///
/// It is a *window* onto an unbounded canvas. Drawing calls take canvas pixel coordinates;
/// the window's top-left corner sits at canvas pixel `(left, top)`, and anything outside the
/// window is silently skipped. Painting one scene into adjacent windows therefore gives
/// tiles that meet without seams (that is how the endless field works).
#[derive(Clone, PartialEq)]
pub struct Rgba {
    /// Width and height in pixels.
    pub side: usize,
    /// Row-major pixels: `[r, g, b, alpha]`.
    pub pixels: Vec<[f32; 4]>,
    /// Canvas position of the top-left pixel.
    pub left: i32,
    pub top: i32,
}

impl Rgba {
    /// A transparent `side`-pixel image at the canvas origin.
    pub fn new(side: usize) -> Self {
        Self::window(side, 0, 0)
    }

    /// A transparent `side`-pixel window whose top-left is canvas pixel `(left, top)`.
    pub fn window(side: usize, left: i32, top: i32) -> Self {
        Self {
            side,
            pixels: vec![[0.0; 4]; side * side],
            left,
            top,
        }
    }

    /// Canvas pixel columns covered by the window.
    pub fn columns(&self) -> Range<i32> {
        self.left..self.left + self.side as i32
    }

    /// Canvas pixel rows covered by the window.
    pub fn rows(&self) -> Range<i32> {
        self.top..self.top + self.side as i32
    }

    /// Coverage at canvas pixel `(x, y)`; 0 outside the window.
    pub fn alpha(&self, x: i32, y: i32) -> f32 {
        let (x, y, side) = (x - self.left, y - self.top, self.side as i32);
        if x < 0 || y < 0 || x >= side || y >= side {
            return 0.0;
        }
        self.pixels[(y * side + x) as usize][3]
    }

    // --------------------------------------------------------------------------------------
    // Drawing

    /// Composite `color` with coverage `alpha` over canvas pixel `(x, y)` (the classic
    /// "over" operator on straight, non-premultiplied colour).
    pub fn paint_over(&mut self, x: i32, y: i32, color: [f32; 3], alpha: f32) {
        let (x, y, side) = (x - self.left, y - self.top, self.side as i32);
        if x < 0 || y < 0 || x >= side || y >= side || alpha <= 0.0 {
            return;
        }

        let pixel = &mut self.pixels[(y * side + x) as usize];
        let alpha = alpha.min(1.0);
        let new_alpha = alpha + pixel[3] * (1.0 - alpha);
        for channel in 0..3 {
            pixel[channel] = (color[channel] * alpha + pixel[channel] * pixel[3] * (1.0 - alpha))
                / new_alpha.max(1e-6);
        }
        pixel[3] = new_alpha;
    }

    /// Anti-aliased filled disc centred at canvas position `(center_x, center_y)`.
    pub fn fill_disc(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        let (x0, x1) = (
            (center_x - radius - 1.0).floor() as i32,
            (center_x + radius + 1.0) as i32,
        );
        let (y0, y1) = (
            (center_y - radius - 1.0).floor() as i32,
            (center_y + radius + 1.0) as i32,
        );
        for y in y0..=y1 {
            for x in x0..=x1 {
                let distance = ((x as f32 + 0.5 - center_x).powi(2)
                    + (y as f32 + 0.5 - center_y).powi(2))
                .sqrt();
                // Pixels on the rim get partial coverage: that is the anti-aliasing.
                self.paint_over(
                    x,
                    y,
                    color,
                    alpha * (radius + 0.5 - distance).clamp(0.0, 1.0),
                );
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Export

    /// Straight (non-premultiplied) RGBA8, row-major.
    pub fn to_rgba8(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|pixel| {
                [
                    pixel[0].clamp(0.0, 255.0) as u8,
                    pixel[1].clamp(0.0, 255.0) as u8,
                    pixel[2].clamp(0.0, 255.0) as u8,
                    (pixel[3].clamp(0.0, 1.0) * 255.0).round() as u8,
                ]
            })
            .collect()
    }

    /// Mean alpha, `0..1`: how much of the image is covered.
    pub fn coverage(&self) -> f32 {
        self.pixels.iter().map(|pixel| pixel[3]).sum::<f32>() / self.pixels.len() as f32
    }
}
