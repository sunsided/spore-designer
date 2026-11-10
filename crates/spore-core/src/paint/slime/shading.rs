//! What the slime film looks like at one pixel.
//!
//! The field value is turned into a *height* (thicker film where the field is stronger), the
//! height's slope gives a surface normal, and the normal is lit like the colonies: diffuse
//! light from the top right plus a sharp specular glint.
//!
//! Besides the lit look, the film reports its unlit look (albedo), its thickness for the
//! height map, and its wetness for the specular map.

use super::Ball;
use super::film::field;
use crate::math::{mix, scale, smooth};
use crate::noise::hash;
use crate::params::{Rgb, Slime};

/// Film thickness at its thickest, reference pixels.
const THICKNESS: f32 = 1.5;

/// Everything the slime paints at one pixel.
pub(super) struct SlimeSample {
    pub lit: Rgb,
    pub lit_alpha: f32,
    pub albedo: Rgb,
    pub albedo_alpha: f32,
    /// Film thickness, reference pixels.
    pub height: f32,
    /// Shininess `0..1`.
    pub gloss: f32,
}

/// The film at canvas pixel `canvas_pixel`, which sits at reference position `reference`;
/// `None` where there is no film.
pub(super) fn shade(
    slime: &Slime,
    balls: &[Ball],
    cutoff: Option<f32>,
    canvas_pixel: (i32, i32),
    reference: (f32, f32),
) -> Option<SlimeSample> {
    let (x, y) = reference;
    let field_at = |x, y| field(balls, slime.ripple, cutoff, x, y);
    let value = field_at(x, y);
    if value < 1.0 {
        return None;
    }

    // Height of the film (0..1), and its slope towards +x and +y (sampled 0.8 reference px
    // away).
    let height_of = |value: f32| smooth(1.0, 6.0, value).sqrt();
    let height = height_of(value);
    let slope_x = height_of(field_at(x + 0.8, y)) - height;
    let slope_y = height_of(field_at(x, y + 0.8)) - height;

    // Surface normal: steep slopes tilt it strongly (x9 exaggerates the thin film).
    let (normal_x, normal_y, normal_z) = (-slope_x * 9.0, -slope_y * 9.0, 1.0);
    let length = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();
    let (normal_x, normal_y, normal_z) = (normal_x / length, normal_y / length, normal_z / length);

    // Light from the top right; viewer straight above.
    let diffuse = (normal_x * 0.45 - normal_y * 0.6 + normal_z * 0.66).max(0.0);
    let half = (normal_x * 0.45 - normal_y * 0.6 + normal_z * 1.66) / 1.82;
    let specular = half.max(0.0).powf(slime.gloss);

    // The colour under that light, and (for the albedo) as if the film were flat: a flat
    // film faces the light with diffuse 0.66 and has no glint.
    let film_color = |diffuse: f32| {
        scale(
            mix(slime.deep, slime.light, height * 0.4 + diffuse * 0.5),
            0.6 + 0.55 * diffuse,
        )
    };
    let mut lit = film_color(diffuse);
    let mut albedo = film_color(0.66);

    // Darker meniscus at the edge, a thin bright refraction line inside it.
    let meniscus = if height < 0.12 {
        1.0 - slime.meniscus
    } else if height < 0.2 {
        1.0 + slime.meniscus * 0.5
    } else {
        1.0
    };
    if meniscus != 1.0 {
        lit = scale(lit, meniscus);
        albedo = scale(albedo, meniscus);
    }

    // Bubbles (or carried spores) in the thicker film.
    let (canvas_x, canvas_y) = canvas_pixel;
    if height > 0.25 && hash(canvas_x, canvas_y, 900) < slime.bubbles {
        lit = slime.bubble;
        albedo = slime.bubble;
    }

    lit = mix(lit, [255.0, 255.0, 246.0], (specular * 2.5).min(1.0));
    let alpha = slime.alpha + slime.depth_alpha * smooth(0.0, 0.6, height);

    Some(SlimeSample {
        lit,
        lit_alpha: alpha.max(specular),
        albedo,
        albedo_alpha: alpha,
        height: height * THICKNESS,
        // Sharper glints (higher exponent) read as wetter.
        gloss: 0.35 + 0.6 * (slime.gloss / (slime.gloss + 30.0)),
    })
}
