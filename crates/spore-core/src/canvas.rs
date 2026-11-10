//! [`Rgba`]: the image every painting pass draws into.

use std::ops::Range;

// ------------------------------------------------------------------------------------------
// The image type

/// A square true-colour image with coverage, in three layers:
///
/// - `pixels`: the finished picture, lit by Sporagon's fixed light from the top right
///   (RGB channels `0..255`, alpha `0..1`);
/// - `albedo`: the same surfaces without that light (no dome shading, glints, shadows or
///   highlights), for relighting with any other light (see `relief.rs`);
/// - `height`: how high the surface stands at each pixel, in reference pixels (see
///   [`REFERENCE_SIZE`](crate::REFERENCE_SIZE)); the source of bump and normal maps;
/// - `gloss`: how shiny the surface is, `0` (dry velvet) to `1` (wet slime): a specular map.
///
/// It is a *window* onto an unbounded canvas. Drawing calls take canvas pixel coordinates;
/// the window's top-left corner sits at canvas pixel `(left, top)`, and anything outside the
/// window is silently skipped. Painting one scene into adjacent windows therefore gives
/// tiles that meet without seams (that is how the endless field works).
#[derive(Clone, PartialEq)]
pub struct Rgba {
    /// Width and height in pixels.
    pub side: usize,
    /// Row-major lit pixels: `[r, g, b, alpha]`.
    pub pixels: Vec<[f32; 4]>,
    /// Row-major unlit pixels: `[r, g, b, alpha]`.
    pub albedo: Vec<[f32; 4]>,
    /// Row-major surface height, reference pixels.
    pub height: Vec<f32>,
    /// Row-major shininess, `0..1`.
    pub gloss: Vec<f32>,
    /// Canvas position of the top-left pixel.
    pub left: i32,
    pub top: i32,
    /// Output pixels per reference pixel: converts `height` slopes into true slopes.
    pub scale: f32,
}

/// Which colour layers a drawing call paints.
#[derive(Clone, Copy)]
enum Layers {
    /// Surface colour: the lit picture and the albedo alike.
    Both,
    /// Lighting only (shadows, highlights, glints): the lit picture, not the albedo.
    LitOnly,
}

impl Rgba {
    /// A transparent, flat `side`-pixel image at the canvas origin.
    pub fn new(side: usize) -> Self {
        Self::window(side, 0, 0)
    }

    /// A transparent, flat `side`-pixel window whose top-left is canvas pixel `(left, top)`.
    pub fn window(side: usize, left: i32, top: i32) -> Self {
        Self {
            side,
            pixels: vec![[0.0; 4]; side * side],
            albedo: vec![[0.0; 4]; side * side],
            height: vec![0.0; side * side],
            gloss: vec![0.0; side * side],
            left,
            top,
            scale: 1.0,
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

    /// Index into the layers for canvas pixel `(x, y)`, or `None` outside the window.
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let (x, y, side) = (x - self.left, y - self.top, self.side as i32);
        if x < 0 || y < 0 || x >= side || y >= side {
            return None;
        }
        Some((y * side + x) as usize)
    }

    /// Coverage of the lit picture at canvas pixel `(x, y)`; 0 outside the window.
    pub fn alpha(&self, x: i32, y: i32) -> f32 {
        self.index(x, y).map_or(0.0, |index| self.pixels[index][3])
    }

    // --------------------------------------------------------------------------------------
    // Colour

    /// Composite surface `color` with coverage `alpha` over canvas pixel `(x, y)`, in the lit
    /// picture and the albedo alike.
    pub fn paint_over(&mut self, x: i32, y: i32, color: [f32; 3], alpha: f32) {
        self.paint_split(x, y, (color, alpha), (color, alpha));
    }

    /// Composite a surface whose lit colour and albedo differ but share one coverage.
    pub fn paint_over_split(
        &mut self,
        x: i32,
        y: i32,
        lit: [f32; 3],
        albedo: [f32; 3],
        alpha: f32,
    ) {
        self.paint_split(x, y, (lit, alpha), (albedo, alpha));
    }

    /// Composite a surface whose lit and unlit looks differ: `(colour, coverage)` for the lit
    /// picture and for the albedo.
    pub fn paint_split(
        &mut self,
        x: i32,
        y: i32,
        (lit, lit_alpha): ([f32; 3], f32),
        (albedo, albedo_alpha): ([f32; 3], f32),
    ) {
        let Some(index) = self.index(x, y) else {
            return;
        };
        if lit_alpha > 0.0 {
            composite(&mut self.pixels[index], lit, lit_alpha);
        }
        if albedo_alpha > 0.0 {
            composite(&mut self.albedo[index], albedo, albedo_alpha);
        }
    }

    /// Composite pure lighting (a shadow, highlight or glint): only the lit picture changes.
    pub fn paint_light(&mut self, x: i32, y: i32, color: [f32; 3], alpha: f32) {
        let Some(index) = self.index(x, y) else {
            return;
        };
        if alpha <= 0.0 {
            return;
        }
        composite(&mut self.pixels[index], color, alpha);
    }

    /// Anti-aliased filled disc of surface colour centred at canvas position
    /// `(center_x, center_y)`.
    pub fn fill_disc(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        self.disc(Layers::Both, center_x, center_y, radius, color, alpha);
    }

    /// Anti-aliased filled disc of pure lighting (a shadow or highlight).
    pub fn fill_light_disc(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        self.disc(Layers::LitOnly, center_x, center_y, radius, color, alpha);
    }

    fn disc(
        &mut self,
        layers: Layers,
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
                let coverage = alpha * (radius + 0.5 - distance).clamp(0.0, 1.0);
                match layers {
                    Layers::Both => self.paint_over(x, y, color, coverage),
                    Layers::LitOnly => self.paint_light(x, y, color, coverage),
                }
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Height

    /// Raise canvas pixel `(x, y)` to at least `height` (reference pixels). Used by surfaces
    /// that rest on the ground: slime, bodies, fringe.
    pub fn lift(&mut self, x: i32, y: i32, height: f32) {
        if let Some(index) = self.index(x, y) {
            self.height[index] = self.height[index].max(height);
        }
    }

    /// Add `amount` (reference pixels) to canvas pixel `(x, y)`. Used by things that sit on
    /// top of whatever is below: rosettes, hyphae.
    pub fn raise(&mut self, x: i32, y: i32, amount: f32) {
        if let Some(index) = self.index(x, y) {
            self.height[index] += amount;
        }
    }

    /// Add a hemisphere-shaped bump: `peak` reference pixels high at canvas position
    /// `(center_x, center_y)`, falling to zero at `radius` canvas pixels. Used for droplets,
    /// beads and spore heads.
    pub fn raise_dome(&mut self, center_x: f32, center_y: f32, radius: f32, peak: f32) {
        let (x0, x1) = (
            (center_x - radius).floor() as i32,
            (center_x + radius) as i32,
        );
        let (y0, y1) = (
            (center_y - radius).floor() as i32,
            (center_y + radius) as i32,
        );
        for y in y0..=y1 {
            for x in x0..=x1 {
                let distance = ((x as f32 + 0.5 - center_x).powi(2)
                    + (y as f32 + 0.5 - center_y).powi(2))
                .sqrt()
                    / radius.max(1e-3);
                if distance < 1.0 {
                    self.raise(x, y, peak * (1.0 - distance * distance).sqrt());
                }
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Gloss

    /// Lay a surface of shininess `gloss` with coverage `alpha` over canvas pixel `(x, y)`.
    pub fn gloss_over(&mut self, x: i32, y: i32, gloss: f32, alpha: f32) {
        if let Some(index) = self.index(x, y) {
            let alpha = alpha.clamp(0.0, 1.0);
            self.gloss[index] += (gloss - self.gloss[index]) * alpha;
        }
    }

    /// A disc of shininess `gloss` centred at canvas position `(center_x, center_y)`.
    pub fn gloss_disc(&mut self, center_x: f32, center_y: f32, radius: f32, gloss: f32) {
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
                self.gloss_over(x, y, gloss, (radius + 0.5 - distance).clamp(0.0, 1.0));
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Export

    /// The lit picture as straight (non-premultiplied) RGBA8, row-major.
    pub fn to_rgba8(&self) -> Vec<u8> {
        encode_rgba8(&self.pixels)
    }

    /// The albedo as straight RGBA8, row-major.
    pub fn albedo_rgba8(&self) -> Vec<u8> {
        encode_rgba8(&self.albedo)
    }

    /// Mean alpha, `0..1`: how much of the image is covered.
    pub fn coverage(&self) -> f32 {
        self.pixels.iter().map(|pixel| pixel[3]).sum::<f32>() / self.pixels.len() as f32
    }
}

// ------------------------------------------------------------------------------------------

/// The classic "over" operator on straight (non-premultiplied) colour.
fn composite(pixel: &mut [f32; 4], color: [f32; 3], alpha: f32) {
    let alpha = alpha.min(1.0);
    let new_alpha = alpha + pixel[3] * (1.0 - alpha);
    for channel in 0..3 {
        pixel[channel] = (color[channel] * alpha + pixel[channel] * pixel[3] * (1.0 - alpha))
            / new_alpha.max(1e-6);
    }
    pixel[3] = new_alpha;
}

fn encode_rgba8(pixels: &[[f32; 4]]) -> Vec<u8> {
    pixels
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
