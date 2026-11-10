//! The mucus puddle a colony sits in: metaballs shaded as a glossy, translucent film, ported
//! from Sporagon's `slime.rs`. Balls of neighbouring colonies share one field, so their
//! puddles flow into each other.

use std::collections::HashMap;
use std::f32::consts::TAU;
use std::ops::Range;

use crate::Rgba;
use crate::noise::{fbm, hash, mix, scale, smooth};
use crate::params::Slime;

/// A metaball in reference pixels.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ball {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

/// The puddle under a colony at `(cx, cy)` with growth `g`: one central ball plus lobes.
pub(crate) fn puddle(sl: &Slime, cx: f32, cy: f32, g: f32, seed: u32) -> Vec<Ball> {
    let mut balls = vec![Ball {
        x: cx,
        y: cy,
        r: sl.size * g,
    }];
    for k in 0..sl.lobes as i32 {
        let a = k as f32 / sl.lobes as f32 * TAU + hash(k, 0, seed) * 0.8;
        let d = (sl.reach + sl.reach_jitter * hash(k, 1, seed)) * g;
        balls.push(Ball {
            x: cx + a.cos() * d,
            y: cy + a.sin() * d,
            r: (sl.lobe_size + sl.lobe_jitter * hash(k, 2, seed)) * g,
        });
    }
    balls
}

/// Upper bound on the film radius around any ball: the field is at most
/// `(1 + ripple / 2) * sum(r^2) / d^2` for `d` the distance to the nearest ball.
fn film_radius(sum_r2: f32, ripple: f32) -> f32 {
    (sum_r2 * (1.0 + ripple * 0.5)).sqrt() + 1.0
}

/// Metaball field at `(x, y)`. With a `cutoff`, each ball fades out smoothly by that distance,
/// so a pixel only depends on nearby balls (what an endless canvas needs).
fn field(balls: &[Ball], ripple: f32, cutoff: Option<f32>, x: f32, y: f32) -> f32 {
    let f: f32 = balls
        .iter()
        .map(|b| {
            let d2 = (x - b.x).powi(2) + (y - b.y).powi(2);
            let w = cutoff.map_or(1.0, |c| 1.0 - smooth(0.6 * c, c, d2.sqrt()));
            w * b.r * b.r / (d2 + 0.25)
        })
        .sum();
    // Ripples: lumpy edges and a surface that catches the light in many small glints.
    f * (1.0 - ripple * 0.5 + ripple * fbm(x / 4.5, y / 4.5, 5, 2))
}

/// Paint the film of `balls` into `img`, `s` output pixels per reference pixel. With a
/// `cutoff` (see [`field`]) the whole window is painted; otherwise only the film's bounds.
pub(crate) fn paint(sl: &Slime, balls: &[Ball], s: f32, cutoff: Option<f32>, img: &mut Rgba) {
    let (xs, ys) = match cutoff {
        Some(_) => (img.xs(), img.ys()),
        None => match bounds(balls, sl.ripple, s, img) {
            Some(b) => b,
            None => return,
        },
    };
    // With a cutoff, only balls in the surrounding buckets can reach a pixel. Buckets keep
    // the input order, so every window sums a pixel's balls in the same order (seamless).
    let cell = cutoff.map(|c| c + 1.0);
    let mut buckets: HashMap<(i32, i32), Vec<Ball>> = HashMap::new();
    if let Some(cell) = cell {
        for b in balls {
            let key = ((b.x / cell).floor() as i32, (b.y / cell).floor() as i32);
            buckets.entry(key).or_default().push(*b);
        }
    }
    let mut near = Vec::new();
    let z = |f: f32| smooth(1.0, 6.0, f).sqrt();
    for py in ys {
        for px in xs.clone() {
            let (x, y) = ((px as f32 + 0.5) / s, (py as f32 + 0.5) / s);
            let local = match cell {
                Some(cell) => {
                    let (bx, by) = ((x / cell).floor() as i32, (y / cell).floor() as i32);
                    near.clear();
                    for gy in by - 1..=by + 1 {
                        for gx in bx - 1..=bx + 1 {
                            if let Some(v) = buckets.get(&(gx, gy)) {
                                near.extend_from_slice(v);
                            }
                        }
                    }
                    &near[..]
                }
                None => balls,
            };
            if local.is_empty() {
                continue;
            }
            let field = |x, y| field(local, sl.ripple, cutoff, x, y);
            let f = field(x, y);
            if f < 1.0 {
                continue;
            }
            let h = z(f);
            let zx = z(field(x + 0.8, y)) - h;
            let zy = z(field(x, y + 0.8)) - h;
            let (nx, ny, nz) = (-zx * 9.0, -zy * 9.0, 1.0);
            let l = (nx * nx + ny * ny + nz * nz).sqrt();
            let (nx, ny, nz) = (nx / l, ny / l, nz / l);
            // Light from the top right; viewer straight above.
            let diffuse = (nx * 0.45 - ny * 0.6 + nz * 0.66).max(0.0);
            let half = (nx * 0.45 - ny * 0.6 + nz * 1.66) / 1.82;
            let spec = half.max(0.0).powf(sl.gloss);
            let mut col = scale(
                mix(sl.deep, sl.light, h * 0.4 + diffuse * 0.5),
                0.6 + 0.55 * diffuse,
            );
            // Darker meniscus at the edge, a thin bright refraction line inside it.
            if h < 0.12 {
                col = scale(col, 1.0 - sl.meniscus);
            } else if h < 0.2 {
                col = scale(col, 1.0 + sl.meniscus * 0.5);
            }
            if h > 0.25 && hash(px, py, 900) < sl.bubbles {
                col = sl.bubble;
            }
            col = mix(col, [255.0, 255.0, 246.0], (spec * 2.5).min(1.0));
            let a = sl.alpha + sl.depth_alpha * smooth(0.0, 0.6, h);
            img.over(px, py, col, a.max(spec));
        }
    }
}

/// Bounding box of the film in reference pixels, `(x0, y0, x1, y1)`. Sampled on a one-pixel
/// grid within the analytic bound, using the strongest possible ripple, plus a safety pixel.
pub(crate) fn extent(balls: &[Ball], ripple: f32) -> Option<(f32, f32, f32, f32)> {
    let balls: Vec<Ball> = balls.iter().copied().filter(|b| b.r > 0.0).collect();
    let p = film_radius(balls.iter().map(|b| b.r * b.r).sum(), ripple);
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for b in &balls {
        (x0, y0) = (x0.min(b.x - p), y0.min(b.y - p));
        (x1, y1) = (x1.max(b.x + p), y1.max(b.y + p));
    }
    if x0 > x1 {
        return None;
    }
    let peak = 1.0 + ripple * 0.5;
    let mut found = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            let f: f32 = balls
                .iter()
                .map(|b| b.r * b.r / ((x - b.x).powi(2) + (y - b.y).powi(2) + 0.25))
                .sum();
            if f * peak >= 1.0 {
                found = (
                    found.0.min(x),
                    found.1.min(y),
                    found.2.max(x),
                    found.3.max(y),
                );
            }
            x += 1.0;
        }
        y += 1.0;
    }
    (found.0 <= found.2).then_some((found.0 - 1.5, found.1 - 1.5, found.2 + 1.5, found.3 + 1.5))
}

/// Canvas pixel columns and rows of the film, clipped to the window.
fn bounds(balls: &[Ball], ripple: f32, s: f32, img: &Rgba) -> Option<(Range<i32>, Range<i32>)> {
    let (x0, y0, x1, y1) = extent(balls, ripple)?;
    let clip = |a: f32, b: f32, r: Range<i32>| {
        let lo = ((a * s).floor() as i32).max(r.start);
        lo..((b * s).ceil() as i32 + 1).min(r.end).max(lo)
    };
    Some((clip(x0, x1, img.xs()), clip(y0, y1, img.ys())))
}
