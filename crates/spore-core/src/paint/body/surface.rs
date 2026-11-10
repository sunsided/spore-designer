//! What one colony's body looks like at one pixel: colour, coverage, and the fringe past
//! its edge.
//!
//! [`ColonyBody::surface`] is a pipeline; each step adjusts the colour from the one before:
//! gradient, growth rings, dust, fluff, age and youth, colour drift, lighting (dome, furrows,
//! grain), specks. The parameters for each step are in `params/surface.rs`.

use super::ColonyBody;
use super::outline::EdgeSample;
use crate::math::{lerp, mix, scale, shift_hue, smooth};
use crate::noise::{fbm, hash};
use crate::paint::lighting::dome;
use crate::params::{ColorStop, Rgb, sample};

impl ColonyBody<'_> {
    /// Body colour and coverage at canvas pixel `(x, y)`, given the melded mass's normalised
    /// radius there (0 centre, 1 edge).
    pub(super) fn surface(
        &self,
        gradient: &[ColorStop],
        x: i32,
        y: i32,
        edge: &EdgeSample,
        normalized_radius: f32,
    ) -> (Rgb, f32) {
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
        let brightness = lerp(
            1.0,
            dome(
                edge.dx / edge.edge_distance,
                edge.dy / edge.edge_distance,
                nr,
            ),
            lighting.dome,
        ) * (1.0 - furrows.depth * furrow * (nr * 2.0).min(1.0) * growth)
            * (1.0 - lighting.grain * 0.5 + lighting.grain * grain);
        color = scale(color, brightness);

        // Specks: single pixels brightened or darkened, one canvas pixel big at any size.
        let specks = &design.specks;
        let speck = hash(x, y, seed + 3);
        if nr < specks.extent && speck > 1.0 - specks.amount {
            color = scale(color, specks.bright);
        } else if nr < specks.extent && speck < specks.amount * 0.7 {
            color = scale(color, specks.dark);
        }

        // Coverage: solid velvet, or cotton thinning towards the edge where the fluff is thin.
        let cotton = smooth(0.08, 0.4, (1.0 - nr * nr) * (0.45 + 0.9 * fluff));
        (color, lerp(1.0, cotton, body.softness))
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
