//! Building up the surface: the height layer (bumps) and the gloss layer (shine).

use super::Rgba;

impl Rgba {
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
}
