//! Hash, value noise, fbm and colour helpers, ported from Sporagon's `art.rs`.

pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xFF_FFFF) as f32 / 16_777_216.0
}

pub fn vnoise(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let a = hash(ix, iy, seed);
    let b = hash(ix + 1, iy, seed);
    let c = hash(ix, iy + 1, seed);
    let d = hash(ix + 1, iy + 1, seed);
    lerp(lerp(a, b, sx), lerp(c, d, sx), sy)
}

pub fn fbm(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut f) = (0.0, 1.0, 0.0, 1.0);
    for o in 0..octaves.max(1) {
        sum += amp * vnoise(x * f, y * f, seed.wrapping_add(o * 101));
        norm += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    sum / norm
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

pub fn scale(c: [f32; 3], k: f32) -> [f32; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

pub fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Rotate the hue of an RGB (0..255) colour by `turns` and scale its saturation by `sat`.
pub fn shift_hue(c: [f32; 3], turns: f32, sat: f32) -> [f32; 3] {
    let [r, g, b] = c.map(|v| (v / 255.0).clamp(0.0, 1.0));
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let mut h = if d <= 1e-6 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    let s = if max <= 1e-6 {
        0.0
    } else {
        (d / max * sat).clamp(0.0, 1.0)
    };
    h = (h + turns).rem_euclid(1.0);
    let v = max;
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let rgb = match i as i32 % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    };
    rgb.map(|x| x * 255.0)
}

/// Small deterministic RNG (xorshift64) for scattering things.
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }

    pub fn f(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_shift_roundtrips() {
        let c = [200.0, 80.0, 40.0];
        let back = shift_hue(shift_hue(c, 0.3, 1.0), -0.3, 1.0);
        for i in 0..3 {
            assert!((c[i] - back[i]).abs() < 0.5, "{c:?} vs {back:?}");
        }
        let grey = shift_hue([128.0; 3], 0.5, 2.0);
        assert!(grey.iter().all(|v| (v - 128.0).abs() < 0.5));
    }
}
