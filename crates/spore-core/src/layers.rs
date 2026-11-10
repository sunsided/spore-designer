//! The painting passes, generalised from Sporagon's `penicillium` and `rhizopus`.
//!
//! Geometry is computed in the 64-pixel reference space (`u = pixel / s`) so a design looks
//! the same at any output size; per-pixel hash specks stay one output pixel big.

use std::f32::consts::TAU;
use std::ops::Range;

use crate::noise;
use crate::noise::{Rng, fbm, hash, lerp, mix, scale, shift_hue, smooth, vnoise};
use crate::params::{ColorStop, Field, MoldParams, Patch, sample};
use crate::slime;
use crate::{REFERENCE_SIZE, Rgba};

const WHITE: [f32; 3] = [255.0, 255.0, 250.0];

struct Ctx<'a> {
    p: &'a MoldParams,
    seed: u32,
    g: f32,
    /// Output pixels per reference pixel.
    s: f32,
    /// Colony centre and radius in reference pixels.
    cx: f32,
    cy: f32,
    rc: f32,
}

impl Ctx<'_> {
    /// Independent RNG stream per layer, so changing one layer's counts (or the output size)
    /// leaves the others where they were.
    fn rng(&self, layer: u64) -> Rng {
        Rng(0x9E37_79B9_7F4A_7C15 ^ (self.seed as u64).wrapping_mul(7919) ^ (layer << 48))
    }

    /// Disc at reference coordinates.
    fn disk(&self, img: &mut Rgba, x: f32, y: f32, r: f32, c: [f32; 3], a: f32) {
        img.disk(x * self.s, y * self.s, r * self.s, c, a);
    }

    /// Canvas pixel columns and rows within `reach` reference pixels of the centre.
    fn span(&self, img: &Rgba, reach: f32) -> (Range<i32>, Range<i32>) {
        span(img, self.s, (self.cx, self.cy), reach)
    }

    /// One reference pixel at reference coordinates.
    fn dot(&self, img: &mut Rgba, x: f32, y: f32, c: [f32; 3], a: f32) {
        if self.s <= 1.5 {
            img.over(
                (x * self.s).floor() as i32,
                (y * self.s).floor() as i32,
                c,
                a,
            );
        } else {
            img.disk(x * self.s, y * self.s, 0.5 * self.s, c, a);
        }
    }
}

/// How far a fully grown colony (without its puddle) reaches from its centre, in reference
/// pixels.
fn reach(p: &MoldParams) -> f32 {
    let sh = &p.shape;
    // Every layer grows with the colony radius, so the fully grown radius bounds all stages.
    let rc = (sh.radius_base + sh.radius_growth).max(0.5);
    let fringe = if p.fringe.enabled {
        p.fringe.width
    } else {
        0.0
    };
    let mut r = rc * (sh.wobble_base + sh.wobble_amp + sh.lobe_amp).max(0.0) * (1.0 + fringe);
    // Melding fills the gap between neighbours; the smooth union bulges by up to meld / 2.
    r += 0.5 * sh.meld.max(0.0);
    let ro = &p.rosettes;
    if ro.enabled && ro.count > 0 {
        // Rosettes are pulled in to stay within the colony's reference square, unless they
        // are bigger than it; see `rosettes`.
        let big = ro.size_max.max(ro.size_min) * rc;
        let half = REFERENCE_SIZE as f32 * 0.5 - 1.0;
        r = r.max((half + 0.05 * big).max(1.05 * big * (1.0 + ro.wobble * 0.5)));
    }
    if p.hyphae.enabled {
        // A strand starts within 0.6 rc and walks at most its length.
        r = r.max(rc * (0.6 + p.hyphae.length));
    }
    let sp = &p.sporangia;
    if sp.enabled {
        r = r.max(0.9 * rc + sp.size_max.max(sp.size_min) + 3.0);
    }
    let dr = &p.drops;
    if dr.enabled {
        r = r.max(0.6 * rc + dr.size_max.max(dr.size_min) + 1.0);
    }
    if p.glisten.enabled {
        r = r.max(0.8 * rc + 3.0);
    }
    r + EDGE
}

/// Canvas pixel columns and rows of `img` within `reach` reference pixels of `(cx, cy)`.
fn span(img: &Rgba, s: f32, (cx, cy): (f32, f32), reach: f32) -> (Range<i32>, Range<i32>) {
    let clip = |c: f32, r: Range<i32>| {
        let lo = (((c - reach) * s).floor() as i32).max(r.start);
        let hi = (((c + reach) * s).ceil() as i32 + 1).min(r.end);
        lo..hi.max(lo)
    };
    (clip(cx, img.xs()), clip(cy, img.ys()))
}

/// Anti-aliased discs and dots spill a little past their nominal reach.
const EDGE: f32 = 2.0;

/// One colony sitting in its slime puddle, at `size` pixels per 64-pixel reference cell. The
/// image is centred on the cell and padded on all sides, so a fully grown design can spill out
/// of its cell without being cut off.
pub(crate) fn paint(p: &MoldParams, seed: u32, g: f32, size: usize) -> Rgba {
    let s = size as f32 / REFERENCE_SIZE as f32;
    let half = REFERENCE_SIZE as f32 / 2.0;
    let colonies = [(half, half, 1.0, seed)];
    let over = overflow(p, &colonies, REFERENCE_SIZE as f32, reach(p) - half);
    let pad = (over * s).ceil() as usize;
    let mut img = Rgba::new(size + 2 * pad);
    let c = (img.w as f32 / s) / 2.0;
    if p.slime.enabled {
        slime::paint(
            &p.slime,
            &slime::puddle(&p.slime, c, c, g, seed),
            s,
            None,
            &mut img,
        );
    }
    bodies(p, s, &[(c, c, g, seed)], &mut img);
    overlays(p, seed, g, s, (c, c), &mut img);
    img
}

/// Where a patch's colonies sit and how much padding its image needs. Neither depends on the
/// growth stage, so it is worked out once per sheet (the padding scans the grown puddle).
pub struct PatchLayout {
    patch: Patch,
    seed: u32,
    size: usize,
    extent: f32,
    margin: f32,
    pad: usize,
}

impl PatchLayout {
    pub fn new(p: &MoldParams, patch: &Patch, seed: u32, size: usize) -> Self {
        let size = size.max(1);
        let extent = REFERENCE_SIZE as f32 * patch.area.max(1.0);
        let s = size as f32 / extent;
        // Colonies stay at least `margin` inside the area; whatever reaches further gets
        // padding. The grown layout bounds every stage.
        let margin = reach(p).min(extent * 0.5);
        let grown = place(patch, seed, extent, margin, 1.0);
        let pad = (overflow(p, &grown, extent, reach(p) - margin) * s).ceil() as usize;
        Self {
            patch: patch.clone(),
            seed,
            size,
            extent,
            margin,
            pad,
        }
    }

    /// The patch at overall growth `g`: several colonies scattered over a square
    /// `patch.area` colony widths wide, growing at staggered times. Their puddles share one
    /// metaball field and run together. `p` must be the design the layout was made for.
    pub fn render(&self, p: &MoldParams, g: f32) -> Rgba {
        let g = g.clamp(0.0, 1.0);
        let s = self.size as f32 / self.extent;
        let mut img = Rgba::new(self.size + 2 * self.pad);
        let offset = self.pad as f32 / s;
        let colonies: Vec<_> = place(&self.patch, self.seed, self.extent, self.margin, g)
            .into_iter()
            .map(|(x, y, g, seed)| (x + offset, y + offset, g, seed))
            .collect();
        if p.slime.enabled {
            let balls: Vec<_> = colonies
                .iter()
                .flat_map(|&(x, y, g, seed)| slime::puddle(&p.slime, x, y, g, seed))
                .collect();
            slime::paint(&p.slime, &balls, s, None, &mut img);
        }
        bodies(p, s, &colonies, &mut img);
        for &(x, y, g, seed) in &colonies {
            overlays(p, seed, g, s, (x, y), &mut img);
        }
        img
    }
}

/// Side of a field chunk in reference pixels (two cells).
pub(crate) const CHUNK: f32 = (crate::CHUNK_CELLS * REFERENCE_SIZE) as f32;

/// Chunk `(cx, cy)` of an endless field, `size` pixels per cell. Each pixel only depends on
/// its canvas position, so chunks tile without seams.
pub(crate) fn paint_chunk(
    p: &MoldParams,
    field: &Field,
    seed: u32,
    g: f32,
    (cx, cy): (i32, i32),
    size: usize,
) -> Rgba {
    let side = (CHUNK * size as f32 / REFERENCE_SIZE as f32).round() as usize;
    let img = Rgba::window(side, cx * side as i32, cy * side as i32);
    paint_field(p, field, seed, g, size, img)
}

/// Paint the field into any window `img`, `size` pixels per cell.
fn paint_field(
    p: &MoldParams,
    field: &Field,
    seed: u32,
    g: f32,
    size: usize,
    mut img: Rgba,
) -> Rgba {
    let s = size as f32 / REFERENCE_SIZE as f32;
    let side = img.w;
    let (x0, y0) = (img.ox as f32 / s, img.oy as f32 / s);
    let (x1, y1) = (x0 + side as f32 / s, y0 + side as f32 / s);

    let sl = &p.slime;
    // Balls fade out by `cutoff`, so a chunk needs colonies whose balls come that close.
    let cutoff = 3.0 * sl.size.max(sl.lobe_size + sl.lobe_jitter) + 1.0;
    let puddle = sl.reach + sl.reach_jitter + cutoff;
    let r = if sl.enabled {
        reach(p).max(puddle)
    } else {
        reach(p)
    };

    let cell = REFERENCE_SIZE as f32 * field.spacing.max(0.2);
    let range =
        |a: f32, b: f32| ((a - r) / cell).floor() as i32 - 1..=((b + r) / cell).ceil() as i32;
    let mut colonies = Vec::new();
    for j in range(y0, y1) {
        for i in range(x0, x1) {
            let h = |k: u32| noise::hash(i, j, seed.wrapping_add(k * 977));
            if h(0) >= field.density {
                continue;
            }
            let x = (i as f32 + 0.5 + (h(1) - 0.5) * field.jitter) * cell;
            let y = (j as f32 + 0.5 + (h(2) - 0.5) * field.jitter) * cell;
            let delay = h(3) * field.stagger.clamp(0.0, 0.95);
            let gi = ((g - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if gi > 0.0 {
                colonies.push((x, y, gi, (h(4) * 16_777_216.0) as u32 ^ seed));
            }
        }
    }
    if sl.enabled {
        let balls: Vec<_> = colonies
            .iter()
            .flat_map(|&(x, y, g, seed)| slime::puddle(sl, x, y, g, seed))
            .collect();
        slime::paint(sl, &balls, s, Some(cutoff), &mut img);
    }
    bodies(p, s, &colonies, &mut img);
    for &(x, y, g, seed) in &colonies {
        overlays(p, seed, g, s, (x, y), &mut img);
    }
    img
}

/// How far past a square `extent` reference pixels wide the fully grown `colonies` reach:
/// `colony_over` for the colonies themselves, plus however far their joint puddle runs.
fn overflow(
    p: &MoldParams,
    colonies: &[(f32, f32, f32, u32)],
    extent: f32,
    colony_over: f32,
) -> f32 {
    let mut over = colony_over;
    if p.slime.enabled {
        let balls: Vec<_> = colonies
            .iter()
            .flat_map(|&(x, y, g, seed)| slime::puddle(&p.slime, x, y, g, seed))
            .collect();
        if let Some((x0, y0, x1, y1)) = slime::extent(&balls, p.slime.ripple) {
            over = over.max(-x0.min(y0) + EDGE).max(x1.max(y1) - extent + EDGE);
        }
    }
    over.max(0.0)
}

/// Colony centres, growth and seed. Best-candidate sampling spreads them evenly; each one
/// starts growing after a random delay of up to `stagger`.
fn place(patch: &Patch, seed: u32, extent: f32, margin: f32, g: f32) -> Vec<(f32, f32, f32, u32)> {
    let mut rng = Rng(0xC2B2_AE3D_27D4_EB4F ^ (seed as u64).wrapping_mul(104_729));
    let mut out: Vec<(f32, f32, f32, u32)> = Vec::new();
    for i in 0..patch.count {
        let mut best = (extent * 0.5, extent * 0.5, -1.0);
        for _ in 0..12 {
            let (x, y) = (
                rng.range(margin, extent - margin),
                rng.range(margin, extent - margin),
            );
            let d = out
                .iter()
                .map(|o| (o.0 - x).powi(2) + (o.1 - y).powi(2))
                .fold(f32::MAX, f32::min);
            if d > best.2 {
                best = (x, y, d);
            }
        }
        let delay = rng.f() * patch.stagger.clamp(0.0, 0.95);
        let gi = ((g - delay) / (1.0 - delay)).clamp(0.0, 1.0);
        out.push((best.0, best.1, gi, seed.wrapping_add(i * 7919)));
    }
    out.retain(|o| o.2 > 0.0);
    out
}

/// Paint what sits on one colony's body (rosettes, hyphae, spore heads, drops, gloss),
/// centred at `(cx, cy)` reference pixels.
fn overlays(p: &MoldParams, seed: u32, g: f32, s: f32, (cx, cy): (f32, f32), img: &mut Rgba) {
    let ctx = Ctx {
        p,
        seed,
        g,
        s,
        cx,
        cy,
        rc: (p.shape.radius_base + p.shape.radius_growth * g).max(0.5),
    };
    if p.rosettes.enabled {
        rosettes(&ctx, img);
    }
    if p.hyphae.enabled {
        hyphae(&ctx, img);
    }
    if p.sporangia.enabled {
        sporangia(&ctx, img);
    }
    if p.drops.enabled && g > p.drops.min_growth {
        drops(&ctx, img);
    }
    if p.glisten.enabled {
        glisten(&ctx, img);
    }
}

/// Dome lighting: 0.72 at the rim, brighter towards the light (top right).
fn dome(dx: f32, dy: f32, nr: f32) -> f32 {
    let h = (1.0 - nr * nr).max(0.0).sqrt();
    let (nx, ny, nz) = (dx * 0.6, dy * 0.6, h + 0.2);
    let l = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-4);
    let d = (nx * 0.45 - ny * 0.6 + nz * 0.66) / l;
    0.72 + 0.4 * d.max(0.0)
}

fn sorted(stops: &[ColorStop]) -> Vec<ColorStop> {
    let mut v = stops.to_vec();
    v.sort_by(|a, b| a.t.total_cmp(&b.t));
    v
}

/// Polynomial smooth minimum: `min(a, b)` with a fillet of size `k` where they meet.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

/// A colony's body as seen from one pixel.
struct Hit {
    dx: f32,
    dy: f32,
    th: f32,
    wob: f32,
    /// Distance from the centre to the wobbly edge in this direction.
    edge: f32,
    /// Signed distance to that edge, negative inside (reference pixels).
    d: f32,
}

/// Per-colony constants of the body pass.
struct Body<'a> {
    ctx: Ctx<'a>,
    rot: f32,
    furrows: f32,
    /// Square half-size, in reference pixels, outside which the colony cannot touch a pixel.
    reach: f32,
}

impl<'a> Body<'a> {
    fn new(ctx: Ctx<'a>, meld: f32) -> Self {
        let (p, rc) = (ctx.p, ctx.rc);
        let rot = if p.shape.spin {
            ctx.rng(0).range(0.0, TAU)
        } else {
            0.0
        };
        let furrows = p.furrows.count + (ctx.seed % 4) as f32 * 2.0;
        let sh = &p.shape;
        let fringe = if p.fringe.enabled {
            p.fringe.width
        } else {
            0.0
        };
        // Painted pixels lie within the fringe band of the melded edge, and a colony only
        // influences one within `meld + 1` of the nearest edge; see `bodies`.
        let reach = rc * (sh.wobble_base + sh.wobble_amp + sh.lobe_amp) * (1.0 + fringe)
            + 1.25 * meld
            + 3.0;
        Self {
            ctx,
            rot,
            furrows,
            reach,
        }
    }

    fn hit(&self, px: i32, py: i32) -> Option<Hit> {
        let c = &self.ctx;
        let (dx, dy) = (
            (px as f32 + 0.5) / c.s - c.cx,
            (py as f32 + 0.5) / c.s - c.cy,
        );
        if dx.abs() > self.reach || dy.abs() > self.reach {
            return None;
        }
        let sh = &c.p.shape;
        let th = dy.atan2(dx) + self.rot;
        let wob = fbm(
            th.cos() * sh.wobble_freq + 5.0,
            th.sin() * sh.wobble_freq + 5.0,
            c.seed,
            3,
        );
        let lobe = sh.lobe_amp * (th * sh.lobes as f32).cos();
        let edge = (c.rc * (sh.wobble_base + sh.wobble_amp * wob + lobe)).max(0.1);
        let d = (dx * dx + dy * dy).sqrt() - edge;
        Some(Hit {
            dx,
            dy,
            th,
            wob,
            edge,
            d,
        })
    }

    /// Body colour and coverage at normalised radius `nr` (0 centre, 1 edge).
    fn color(&self, gradient: &[ColorStop], px: i32, py: i32, h: &Hit, nr: f32) -> ([f32; 3], f32) {
        let c = &self.ctx;
        let (p, seed, g, s) = (c.p, c.seed, c.g, c.s);
        let (b, l) = (&p.body, &p.lighting);
        let (ux, uy) = (px as f32 / s, py as f32 / s);
        let mut col = sample(gradient, nr);
        let z = &p.zones;
        if nr >= z.start && nr < z.end {
            let zone = (nr * z.freq + h.wob * 2.0).sin() * 0.5 + 0.5;
            col = mix(col, z.rgb, zone * z.amount);
        }
        let d = &p.dust;
        if nr < d.extent {
            let dust = smooth(d.low, d.high, fbm(ux / d.scale, uy / d.scale, seed + 13, 3));
            col = mix(col, d.rgb, dust * d.amount);
        }
        let fluff = fbm(ux / b.fluff_scale, uy / b.fluff_scale, seed + 5, 3);
        col = scale(col, lerp(b.fluff_low, 1.0, fluff.powf(0.8)));
        col = mix(col, b.aged, (1.0 - nr * 1.6).max(0.0) * g * b.age);
        col = mix(
            col,
            b.young,
            (1.0 - g * b.youth).max(0.0) * (1.0 - nr * 0.3),
        );
        let v = &p.variance;
        if v.hue != 0.0 || v.saturation != 1.0 {
            let n = fbm(ux / v.scale, uy / v.scale, seed + 21, 3);
            col = shift_hue(col, (n - 0.5) * 2.0 * v.hue, v.saturation);
        }
        let f = &p.furrows;
        let furrow = ((h.th * self.furrows + h.wob * 4.0).sin() * 0.5 + 0.5).powf(f.sharpness);
        let fine = fbm(ux / l.grain_scale, uy / l.grain_scale, seed + 11, 2);
        let k = lerp(1.0, dome(h.dx / h.edge, h.dy / h.edge, nr), l.dome)
            * (1.0 - f.depth * furrow * (nr * 2.0).min(1.0) * g)
            * (1.0 - l.grain * 0.5 + l.grain * fine);
        col = scale(col, k);
        let sp = &p.specks;
        let hs = hash(px, py, seed + 3);
        if nr < sp.extent && hs > 1.0 - sp.amount {
            col = scale(col, sp.bright);
        } else if nr < sp.extent && hs < sp.amount * 0.7 {
            col = scale(col, sp.dark);
        }
        let cotton = smooth(0.08, 0.4, (1.0 - nr * nr) * (0.45 + 0.9 * fluff));
        (col, lerp(1.0, cotton, b.softness))
    }

    /// Fringe coverage at normalised radius `nr` past the edge.
    fn fringe(&self, px: i32, py: i32, nr: f32) -> f32 {
        let fr = &self.ctx.p.fringe;
        // Output-pixel noise: the fringe stays hair-fine at any size.
        let fuzz = fbm(
            px as f32 / fr.scale,
            py as f32 / fr.scale,
            self.ctx.seed + 7,
            2,
        );
        (1.0 - (nr - 1.0) / fr.width).powf(1.5) * smooth(0.3, 0.7, fuzz) * fr.alpha
    }
}

/// Paint the bodies of all `colonies` as one melded mass: the edge is the smooth union of
/// every colony's edge (gaps up to `shape.meld` close), and colour, texture and fringe blend
/// between the colonies by how close each one's edge is. With one colony this is just its
/// body.
fn bodies(p: &MoldParams, s: f32, colonies: &[(f32, f32, f32, u32)], img: &mut Rgba) {
    if colonies.is_empty() {
        return;
    }
    let gradient = sorted(&p.body.gradient);
    // Fillet size: two edges `meld` apart just touch (the smooth min dips by k / 4 midway).
    let meld = 2.0 * p.shape.meld.max(0.0);
    // Blend weights fall to exactly zero `support` past the nearest edge, so colonies out of
    // range never matter (and field chunks stay seamless).
    let support = meld + 1.0;
    let bodies: Vec<Body<'_>> = colonies
        .iter()
        .map(|&(cx, cy, g, seed)| {
            let ctx = Ctx {
                p,
                seed,
                g,
                s,
                cx,
                cy,
                rc: (p.shape.radius_base + p.shape.radius_growth * g).max(0.5),
            };
            Body::new(ctx, meld)
        })
        .collect();
    let (xs, ys) = bodies
        .iter()
        .map(|b| b.ctx.span(img, b.reach))
        .reduce(|(ax, ay), (bx, by)| {
            (
                ax.start.min(bx.start)..ax.end.max(bx.end),
                ay.start.min(by.start)..ay.end.max(by.end),
            )
        })
        .expect("at least one colony");
    let mut hits: Vec<(&Body<'_>, Hit)> = Vec::new();
    for py in ys {
        for px in xs.clone() {
            hits.clear();
            hits.extend(bodies.iter().filter_map(|b| b.hit(px, py).map(|h| (b, h))));
            if hits.is_empty() {
                continue;
            }
            // Smooth only the two nearest edges: chaining smin over every colony would let
            // the bulge grow with the number of neighbours.
            let (mut dmin, mut d2) = (f32::MAX, f32::MAX);
            for h in &hits {
                if h.1.d < dmin {
                    (dmin, d2) = (h.1.d, dmin);
                } else if h.1.d < d2 {
                    d2 = h.1.d;
                }
            }
            let d = smin(dmin, d2, meld);
            let weight = |h: &Hit| (1.0 - (h.d - dmin) / support).max(0.0).powi(2);
            let wsum: f32 = hits.iter().map(|h| weight(&h.1)).sum();
            let edge = hits.iter().map(|h| weight(&h.1) * h.1.edge).sum::<f32>() / wsum;
            let nr = (1.0 + d / edge).max(0.0);
            if nr < 1.0 {
                let (mut col, mut a) = ([0.0; 3], 0.0);
                for (b, h) in &hits {
                    let w = weight(h) / wsum;
                    if w > 0.0 {
                        let (c, ca) = b.color(&gradient, px, py, h, nr);
                        col = [col[0] + c[0] * w, col[1] + c[1] * w, col[2] + c[2] * w];
                        a += ca * w;
                    }
                }
                img.over(px, py, col, a);
            } else if p.fringe.enabled && nr < 1.0 + p.fringe.width {
                let a: f32 = hits
                    .iter()
                    .map(|(b, h)| weight(h) / wsum * b.fringe(px, py, nr))
                    .sum();
                img.over(px, py, p.fringe.rgb, a);
            }
        }
    }
}

/// Lichen rosettes: each one is born at a random growth stage, then spreads with the colony.
/// Ring colours run back and forth along the gradient (a triangle wave) so bands stay smooth.
fn rosettes(ctx: &Ctx<'_>, img: &mut Rgba) {
    let p = ctx.p;
    let ro = &p.rosettes;
    let (seed, g, s, rc) = (ctx.seed, ctx.g, ctx.s, ctx.rc);
    let gradient = sorted(&ro.gradient);
    let mut rng = ctx.rng(1);
    let max = REFERENCE_SIZE as f32;
    for i in 0..ro.count {
        let a = rng.range(0.0, TAU);
        let d = rng.f().sqrt() * ro.spread * rc;
        let size = rng.range(ro.size_min, ro.size_max.max(ro.size_min));
        let born = rng.range(0.0, 0.6);
        let phase = rng.f();
        let hue = rng.range(-1.0, 1.0) * ro.hue_jitter;
        let local = ((g - born) / (1.0 - born)).clamp(0.0, 1.0);
        let rr = size * rc * local;
        if rr < 0.5 {
            continue;
        }
        // Keep the fully grown rosette inside the colony's reference square.
        let d = d.min((max * 0.5 - 1.0 - size * rc * (1.0 + ro.wobble)).max(0.0));
        let (cx, cy) = (ctx.cx + a.cos() * d, ctx.cy + a.sin() * d);
        let reach = rr * (1.0 + ro.wobble) * 1.1;
        let (xs, ys) = span(img, s, (cx, cy), reach);
        let rseed = seed.wrapping_add(31 + i * 17);
        for py in ys.clone() {
            for px in xs.clone() {
                let (dx, dy) = ((px as f32 + 0.5) / s - cx, (py as f32 + 0.5) / s - cy);
                let th = dy.atan2(dx);
                let w = fbm(th.cos() * 1.5 + 3.0, th.sin() * 1.5 + 3.0, rseed, 2);
                let edge = rr * (1.0 - ro.wobble * 0.5 + ro.wobble * w);
                let n = (dx * dx + dy * dy).sqrt() / edge.max(0.1);
                if n >= 1.05 {
                    continue;
                }
                let (ux, uy) = (px as f32 / s, py as f32 / s);
                let blot = fbm(ux / 2.5, uy / 2.5, rseed + 3, 3);
                let t = n * ro.rings + phase + (blot - 0.5) * ro.blotch;
                let tri = 1.0 - (2.0 * t.fract() - 1.0).abs();
                let mut col = sample(&gradient, tri);
                if hue != 0.0 {
                    col = shift_hue(col, hue, 1.0);
                }
                let fine = fbm(
                    ux / p.lighting.grain_scale,
                    uy / p.lighting.grain_scale,
                    rseed,
                    2,
                );
                let k = (1.0 - ro.rim_dark * smooth(0.6, 1.0, n))
                    * lerp(
                        1.0,
                        dome(dx / edge, dy / edge, n.min(1.0)),
                        p.lighting.dome * 0.6,
                    )
                    * (1.0 - p.lighting.grain * 0.5 + p.lighting.grain * fine);
                let fuzz = fbm(px as f32 / 1.3, py as f32 / 1.3, rseed + 7, 2);
                let a = 1.0 - smooth(0.9, 1.05, n) * (0.4 + 0.6 * (1.0 - fuzz));
                img.over(px, py, scale(col, k), a.max(0.0) * ro.alpha);
            }
        }
    }
}

/// Aerial hyphae: wandering one-pixel strands, past the colony edge too.
fn hyphae(ctx: &Ctx<'_>, img: &mut Rgba) {
    let h = &ctx.p.hyphae;
    let (g, s, cx, cy, rc) = (ctx.g, ctx.s, ctx.cx, ctx.cy, ctx.rc);
    let mut rng = ctx.rng(2);
    // More strands at larger sizes keep the same density of one-pixel hairs.
    let n = ((h.base + h.growth * g) * s.max(1.0)) as u32;
    let sub = s.ceil().max(1.0) as u32;
    const STEP: f32 = 0.7;
    for _ in 0..n {
        let a0 = rng.range(0.0, TAU);
        let d0 = rng.range(0.0, 0.6) * rc;
        let (mut x, mut y) = (cx + a0.cos() * d0, cy + a0.sin() * d0);
        let mut a = a0 + rng.range(-0.8, 0.8);
        let len = rng.range(0.5, 1.0) * rc * h.length;
        let mut t = 0.0;
        while t < len {
            a += rng.range(-h.jitter, h.jitter);
            let col = scale(h.rgb, rng.range(1.0 - h.variance, 1.0));
            for _ in 0..sub {
                x += a.cos() * STEP / sub as f32;
                y += a.sin() * STEP / sub as f32;
                let out = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / rc;
                let alpha = if out > 0.95 { h.outer_alpha } else { h.alpha };
                img.over((x * s).floor() as i32, (y * s).floor() as i32, col, alpha);
            }
            t += STEP;
        }
    }
}

/// Sporangia on slanted stalks, with shadows; ripe ones dark, unripe ones pale.
fn sporangia(ctx: &Ctx<'_>, img: &mut Rgba) {
    let sp = &ctx.p.sporangia;
    let (g, cx, cy, rc) = (ctx.g, ctx.cx, ctx.cy, ctx.rc);
    let mut rng = ctx.rng(3);
    for _ in 0..(sp.count * g * g) as u32 {
        let a = rng.range(0.0, TAU);
        let d = rng.f().sqrt() * 0.9 * rc;
        let (px, py) = (cx + a.cos() * d, cy + a.sin() * d);
        let rr = rng.range(sp.size_min, sp.size_max.max(sp.size_min));
        let ripe = rng.f() < g * sp.ripe_chance;
        for k in 0..3 {
            let k = k as f32;
            ctx.dot(img, px - 0.5 - k * 0.4, py + 1.0 + k, sp.stalk, 0.7);
        }
        ctx.disk(img, px + 1.2, py + 1.5, rr, [50.0, 48.0, 44.0], 0.35);
        let body = if ripe { sp.ripe } else { sp.unripe };
        ctx.disk(img, px, py, rr, body, 1.0);
        ctx.dot(
            img,
            px + rr * 0.35,
            py - rr * 0.45,
            mix(body, WHITE, 0.45),
            0.9,
        );
    }
}

/// Guttation: droplets of exudate with a shadow and a highlight.
fn drops(ctx: &Ctx<'_>, img: &mut Rgba) {
    let dr = &ctx.p.drops;
    let (cx, cy, rc) = (ctx.cx, ctx.cy, ctx.rc);
    let mut rng = ctx.rng(4);
    for _ in 0..(dr.count + ctx.seed % 3) {
        let a = rng.range(0.0, TAU);
        let d = rng.range(0.2, 0.6) * rc;
        let (px, py) = (cx + a.cos() * d, cy + a.sin() * d);
        let rr = rng.range(dr.size_min, dr.size_max.max(dr.size_min));
        ctx.disk(img, px + 0.6, py + 0.8, rr, [40.0, 50.0, 30.0], 0.5);
        ctx.disk(img, px, py, rr, dr.rgb, 1.0);
        ctx.dot(
            img,
            px + rr * 0.4,
            py - rr * 0.5,
            mix(dr.rgb, WHITE, 0.8),
            1.0,
        );
    }
}

/// Wet look: a glossy sheen on the side facing the light and a few beads of clear liquid.
fn glisten(ctx: &Ctx<'_>, img: &mut Rgba) {
    let gl = &ctx.p.glisten;
    let (seed, g, s, cx, cy, rc) = (ctx.seed, ctx.g, ctx.s, ctx.cx, ctx.cy, ctx.rc);
    let (xs, ys) = ctx.span(img, rc + 1.0);
    for py in ys {
        for px in xs.clone() {
            let (dx, dy) = (
                ((px as f32 + 0.5) / s - cx) / rc,
                ((py as f32 + 0.5) / s - cy) / rc,
            );
            let nr = (dx * dx + dy * dy).sqrt();
            if nr < 0.95 && img.alpha(px, py) > 0.9 {
                let h = (1.0 - nr * nr).max(0.0).sqrt();
                let half = (dx * 0.8 * 0.45 - dy * 0.8 * 0.6 + h * 1.66) / 1.79;
                let noise = vnoise(px as f32 / s / 3.0, py as f32 / s / 3.0, seed + 40);
                let sheen = half.max(0.0).powf(gl.power) * gl.sheen * smooth(0.3, 0.7, noise);
                img.over(px, py, [250.0, 252.0, 240.0], sheen);
            }
        }
    }
    let mut rng = ctx.rng(5);
    for _ in 0..(gl.beads_base + gl.beads_growth * g) as u32 {
        let a = rng.range(0.0, TAU);
        let d = rng.f().sqrt() * 0.8 * rc;
        let (px, py, rr) = (cx + a.cos() * d, cy + a.sin() * d, rng.range(0.9, 2.0));
        ctx.disk(img, px + 0.7, py + 0.9, rr, [20.0, 24.0, 18.0], 0.3);
        ctx.disk(img, px, py, rr, [210.0, 220.0, 205.0], 0.35);
        ctx.dot(img, px + rr * 0.35, py - rr * 0.45, WHITE, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_are_seamless() {
        let field = Field::default();
        for (name, f) in crate::presets::ALL {
            let p = f();
            let (size, g) = (24, 0.8);
            let a = paint_chunk(&p, &field, 3, g, (-1, 0), size);
            let b = paint_chunk(&p, &field, 3, g, (0, 0), size);
            let w = a.w;
            // One window two chunks wide, upper half: must match both chunks exactly.
            let wide = paint_field(&p, &field, 3, g, size, Rgba::window(2 * w, -(w as i32), 0));
            for y in 0..w {
                for x in 0..2 * w {
                    let (cx, cy) = (x as i32 - w as i32, y as i32);
                    let chunk = if cx < 0 { &a } else { &b };
                    let expect = chunk.px[(cy - chunk.oy) as usize * w + (cx - chunk.ox) as usize];
                    assert_eq!(
                        wide.px[y * 2 * w + x],
                        expect,
                        "{name}: seam at ({cx}, {cy})"
                    );
                }
            }
            assert!(b.coverage() > 0.05, "{name}: empty chunk");
        }
    }
}
