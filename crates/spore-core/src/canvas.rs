//! [`Rgba`]: the image every painting pass draws into.
//!
//! - `canvas/color.rs`: compositing colour into the lit picture and the albedo.
//! - `canvas/surface.rs`: building up the height and gloss layers.

mod color;
mod surface;

use std::ops::Range;

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

    /// The image without `margin` pixels on every side (all layers; the window moves
    /// accordingly).
    pub fn cropped(&self, margin: usize) -> Self {
        use crate::crop;
        Self {
            side: self.side.saturating_sub(2 * margin),
            pixels: crop(&self.pixels, self.side, 1, margin),
            albedo: crop(&self.albedo, self.side, 1, margin),
            height: crop(&self.height, self.side, 1, margin),
            gloss: crop(&self.gloss, self.side, 1, margin),
            left: self.left + margin as i32,
            top: self.top + margin as i32,
            scale: self.scale,
        }
    }

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
