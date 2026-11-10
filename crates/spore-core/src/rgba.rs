/// A square true-colour sprite with coverage: RGB in 0..255, alpha in 0..1.
///
/// The sprite is a window onto an unbounded canvas: drawing calls take canvas pixel
/// coordinates, and the window's top-left sits at canvas pixel `(ox, oy)`. Painting the same
/// scene into adjacent windows gives seamless chunks.
#[derive(Clone, PartialEq)]
pub struct Rgba {
    pub w: usize,
    pub px: Vec<[f32; 4]>,
    pub ox: i32,
    pub oy: i32,
}

impl Rgba {
    pub fn new(w: usize) -> Self {
        Self::window(w, 0, 0)
    }

    /// A `w`-pixel window whose top-left is canvas pixel `(ox, oy)`.
    pub fn window(w: usize, ox: i32, oy: i32) -> Self {
        Self {
            w,
            px: vec![[0.0; 4]; w * w],
            ox,
            oy,
        }
    }

    /// Canvas pixel columns (or rows) covered by the window along x (or y).
    pub fn xs(&self) -> std::ops::Range<i32> {
        self.ox..self.ox + self.w as i32
    }

    pub fn ys(&self) -> std::ops::Range<i32> {
        self.oy..self.oy + self.w as i32
    }

    /// Coverage at canvas pixel `(x, y)`, 0 outside the window.
    pub fn alpha(&self, x: i32, y: i32) -> f32 {
        let (x, y, w) = (x - self.ox, y - self.oy, self.w as i32);
        if x < 0 || y < 0 || x >= w || y >= w {
            return 0.0;
        }
        self.px[(y * w + x) as usize][3]
    }

    /// Paint `c` over canvas pixel `(x, y)` with coverage `a`.
    pub fn over(&mut self, x: i32, y: i32, c: [f32; 3], a: f32) {
        let (x, y, w) = (x - self.ox, y - self.oy, self.w as i32);
        if x < 0 || y < 0 || x >= w || y >= w || a <= 0.0 {
            return;
        }
        let p = &mut self.px[(y * w + x) as usize];
        let a = a.min(1.0);
        let na = a + p[3] * (1.0 - a);
        for i in 0..3 {
            p[i] = (c[i] * a + p[i] * p[3] * (1.0 - a)) / na.max(1e-6);
        }
        p[3] = na;
    }

    /// Anti-aliased filled disc, in canvas pixels.
    pub fn disk(&mut self, cx: f32, cy: f32, r: f32, c: [f32; 3], a: f32) {
        let (x0, x1) = ((cx - r - 1.0).floor() as i32, (cx + r + 1.0) as i32);
        let (y0, y1) = ((cy - r - 1.0).floor() as i32, (cy + r + 1.0) as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                self.over(x, y, c, a * (r + 0.5 - d).clamp(0.0, 1.0));
            }
        }
    }

    /// Straight (non-premultiplied) RGBA8, row-major.
    pub fn to_rgba8(&self) -> Vec<u8> {
        self.px
            .iter()
            .flat_map(|p| {
                [
                    p[0].clamp(0.0, 255.0) as u8,
                    p[1].clamp(0.0, 255.0) as u8,
                    p[2].clamp(0.0, 255.0) as u8,
                    (p[3].clamp(0.0, 1.0) * 255.0).round() as u8,
                ]
            })
            .collect()
    }

    /// Mean alpha, 0..1.
    pub fn coverage(&self) -> f32 {
        self.px.iter().map(|p| p[3]).sum::<f32>() / self.px.len() as f32
    }
}
