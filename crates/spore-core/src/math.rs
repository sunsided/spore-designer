//! Small numeric and colour helpers shared by every painting pass. Colours are RGB triples
//! with channels in `0..255`.

// ------------------------------------------------------------------------------------------
// Scalars

/// Linear blend: `a` at `t = 0`, `b` at `t = 1`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smoothstep: 0 below `edge0`, 1 above `edge1`, an S-curve in between.
pub fn smooth(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ------------------------------------------------------------------------------------------
// Colours

/// Blend two colours; `t` is clamped to `0..1`.
pub fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

/// Brighten (`factor > 1`) or darken (`factor < 1`) a colour.
pub fn scale(color: [f32; 3], factor: f32) -> [f32; 3] {
    [color[0] * factor, color[1] * factor, color[2] * factor]
}

/// Rotate the hue of a colour by `turns` (1 = full circle) and multiply its saturation by
/// `saturation`. Works through HSV: convert, adjust, convert back.
pub fn shift_hue(color: [f32; 3], turns: f32, saturation: f32) -> [f32; 3] {
    // RGB -> HSV.
    let [r, g, b] = color.map(|v| (v / 255.0).clamp(0.0, 1.0));
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;

    let mut hue = if chroma <= 1e-6 {
        0.0
    } else if max == r {
        ((g - b) / chroma).rem_euclid(6.0)
    } else if max == g {
        (b - r) / chroma + 2.0
    } else {
        (r - g) / chroma + 4.0
    } / 6.0;
    let sat = if max <= 1e-6 {
        0.0
    } else {
        (chroma / max * saturation).clamp(0.0, 1.0)
    };
    hue = (hue + turns).rem_euclid(1.0);

    // HSV -> RGB.
    let value = max;
    let sector = (hue * 6.0).floor();
    let within = hue * 6.0 - sector;
    let (p, q, t) = (
        value * (1.0 - sat),
        value * (1.0 - sat * within),
        value * (1.0 - sat * (1.0 - within)),
    );
    let rgb = match sector as i32 % 6 {
        0 => [value, t, p],
        1 => [q, value, p],
        2 => [p, value, t],
        3 => [p, q, value],
        4 => [t, p, value],
        _ => [value, p, q],
    };
    rgb.map(|channel| channel * 255.0)
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_shift_roundtrips() {
        let color = [200.0, 80.0, 40.0];
        let back = shift_hue(shift_hue(color, 0.3, 1.0), -0.3, 1.0);
        for i in 0..3 {
            assert!((color[i] - back[i]).abs() < 0.5, "{color:?} vs {back:?}");
        }

        let grey = shift_hue([128.0; 3], 0.5, 2.0);
        assert!(grey.iter().all(|v| (v - 128.0).abs() < 0.5));
    }
}
