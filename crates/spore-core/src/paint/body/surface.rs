//! What one colony's body looks like at one pixel: colour, coverage, and the fringe past
//! its edge.
//!
//! [`ColonyBody::surface`] is a pipeline; each step adjusts the colour from the one before:
//! gradient, growth rings, dust, fluff, age and youth, colour drift, lighting (dome, furrows,
//! grain), specks. The parameters for each step are in `params/surface.rs`.
//!
//! The lit colour includes the dome shading from the fixed light; the albedo keeps only
//! furrows and grain (they darken crevices whatever the light). The same dome, furrows, fluff
//! and grain also shape the height map.

use super::ColonyBody;
use super::outline::EdgeSample;
use crate::math::{lerp, mix, scale, shift_hue, smooth};
use crate::noise::{fbm, hash};
use crate::paint::lighting::dome;
use crate::params::{ColorStop, Rgb, sample};

/// Dome height at the centre of a colony, as a fraction of its radius.
const DOME_RELIEF: f32 = 0.25;
/// Height of fluff and grain texture, reference pixels per unit of noise.
const FLUFF_RELIEF: f32 = 1.2;
const GRAIN_RELIEF: f32 = 0.6;
/// Fringe hairs stand this high where the fringe is dense, reference pixels.
pub(super) const FRINGE_RELIEF: f32 = 0.4;

/// Everything a body paints at one pixel.
pub(super) struct BodySample {
    pub lit: Rgb,
    pub albedo: Rgb,
    pub alpha: f32,
    /// Surface height, reference pixels.
    pub height: f32,
    /// Shininess `0..1`.
    pub gloss: f32,
}

impl ColonyBody<'_> {
    /// The body at canvas pixel `(x, y)`, given the melded mass's normalised radius there
    /// (0 centre, 1 edge).
    pub(super) fn surface(
        &self,
        gradient: &[ColorStop],
        x: i32,
        y: i32,
        edge: &EdgeSample,
        normalized_radius: f32,
    ) -> BodySample {
        let brush = &self.brush;
        let (design, seed, growth) = (brush.design, brush.seed, brush.growth);
        let body = &design.body;
        let lighting = &design.lighting;
        let nr = normalized_radius;

        // Noise coordinates in reference pixels: textures keep their size at any output size.
        let (ux, uy) = (x as f32 / brush.scale, y as f32 / brush.scale);

        // Base colour, and concentric growth rings.
        let mut color = sample(gradient, nr);
        let zones = &design.zones;
        if nr >= zones.start && nr < zones.end {
            let ring = (nr * zones.freq + edge.wobble * 2.0).sin() * 0.5 + 0.5;
            color = mix(color, zones.rgb, ring * zones.amount);
        }

        // Powdery dust patches.
        let dust = &design.dust;
        if nr < dust.extent {
            let patch = smooth(
                dust.low,
                dust.high,
                fbm(ux / dust.scale, uy / dust.scale, seed + 13, 3),
            );
            color = mix(color, dust.rgb, patch * dust.amount);
        }

        // Fluff shading, old grey centre, young white mycelium.
        let fluff = fbm(ux / body.fluff_scale, uy / body.fluff_scale, seed + 5, 3);
        color = scale(color, lerp(body.fluff_low, 1.0, fluff.powf(0.8)));
        color = mix(
            color,
            body.aged,
            (1.0 - nr * 1.6).max(0.0) * growth * body.age,
        );
        color = mix(
            color,
            body.young,
            (1.0 - growth * body.youth).max(0.0) * (1.0 - nr * 0.3),
        );

        // Slow hue and saturation drift.
        let variance = &design.variance;
        if variance.hue != 0.0 || variance.saturation != 1.0 {
            let drift = fbm(ux / variance.scale, uy / variance.scale, seed + 21, 3);
            color = shift_hue(
                color,
                (drift - 0.5) * 2.0 * variance.hue,
                variance.saturation,
            );
        }

        // Lighting: dome shading, darker furrows (deeper on grown colonies), fine grain.
        let furrows = &design.furrows;
        let furrow = ((edge.angle * self.furrow_count + edge.wobble * 4.0).sin() * 0.5 + 0.5)
            .powf(furrows.sharpness);
        let grain = fbm(
            ux / lighting.grain_scale,
            uy / lighting.grain_scale,
            seed + 11,
            2,
        );
        let furrow_shade = 1.0 - furrows.depth * furrow * (nr * 2.0).min(1.0) * growth;
        let grain_shade = 1.0 - lighting.grain * 0.5 + lighting.grain * grain;
        let dome_shade = lerp(
            1.0,
            dome(
                edge.dx / edge.edge_distance,
                edge.dy / edge.edge_distance,
                nr,
            ),
            lighting.dome,
        );
        let mut albedo = scale(color, furrow_shade * grain_shade);
        color = scale(color, dome_shade * furrow_shade * grain_shade);

        // Specks: single pixels brightened or darkened, one canvas pixel big at any size.
        let specks = &design.specks;
        let speck = hash(x, y, seed + 3);
        let speck_factor = if nr < specks.extent && speck > 1.0 - specks.amount {
            Some(specks.bright)
        } else if nr < specks.extent && speck < specks.amount * 0.7 {
            Some(specks.dark)
        } else {
            None
        };
        if let Some(factor) = speck_factor {
            color = scale(color, factor);
            albedo = scale(albedo, factor);
        }

        // Coverage: solid velvet, or cotton thinning towards the edge where the fluff is thin.
        let cotton = smooth(0.08, 0.4, (1.0 - nr * nr) * (0.45 + 0.9 * fluff));
        let alpha = lerp(1.0, cotton, body.softness);

        // Height: a dome sunk along the furrows, roughened by fluff (on cotton) and grain,
        // tapering where cotton coverage thins.
        let dome_height = brush.radius * DOME_RELIEF * (1.0 - nr * nr).max(0.0).sqrt();
        let texture_height = body.softness * (fluff - 0.5) * FLUFF_RELIEF
            + lighting.grain * (grain - 0.5) * GRAIN_RELIEF;
        let height = ((dome_height * furrow_shade + texture_height) * alpha).max(0.0);

        // Gloss: velvet is dull, cotton duller; a glistening colony is wet.
        let glisten = &design.glisten;
        let wetness = if glisten.enabled {
            0.45 * glisten.sheen
        } else {
            0.0
        };

        BodySample {
            lit: color,
            albedo,
            alpha,
            height,
            gloss: lerp(0.12, 0.04, body.softness) + wetness,
        }
    }

    /// Fringe coverage at canvas pixel `(x, y)`, at normalised radius `normalized_radius`
    /// past the edge (`1..1 + width`).
    pub(super) fn fringe(&self, x: i32, y: i32, normalized_radius: f32) -> f32 {
        let fringe = &self.brush.design.fringe;
        // Output-pixel noise: the fringe stays hair-fine at any size.
        let fuzz = fbm(
            x as f32 / fringe.scale,
            y as f32 / fringe.scale,
            self.brush.seed + 7,
            2,
        );
        (1.0 - (normalized_radius - 1.0) / fringe.width).powf(1.5)
            * smooth(0.3, 0.7, fuzz)
            * fringe.alpha
    }
}
