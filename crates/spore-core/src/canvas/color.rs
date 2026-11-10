//! Drawing colour: compositing into the lit picture and the albedo.

use super::Rgba;

/// Which colour layers a drawing call paints.
#[derive(Clone, Copy)]
enum Layers {
    /// Surface colour: the lit picture and the albedo alike.
    Both,
    /// Lighting only (shadows, highlights, glints): the lit picture, not the albedo.
    LitOnly,
}

impl Rgba {
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
}

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
