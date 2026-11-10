//! Painting: turning a design into pixels.
//!
//! # The scene
//!
//! Every view paints the same *scene*, in the same order ([`paint_scene`]):
//!
//! 1. **Slime** (`paint/slime.rs`): the puddles under all colonies, as one metaball film, so
//!    neighbouring puddles flow together.
//! 2. **Bodies** (`paint/body.rs`): all colony bodies at once, melded into one mass where
//!    they come close; the fuzzy fringe is part of this pass.
//! 3. **Overlays** (`paint/overlays.rs`): per colony, what sits on top: lichen rosettes,
//!    hyphae, spore heads, drops and gloss.
//!
//! # The views
//!
//! The views differ only in where colonies sit and how large the image is:
//!
//! - `paint/single.rs` ([`paint_colony`]): one colony centred in its cell, padded so growth
//!   may spill out.
//! - `paint/patch.rs` ([`PatchLayout`]): several colonies scattered over a square area.
//! - `paint/field.rs` ([`paint_chunk`]): one tile of an endless field; tiles meet without
//!   seams.
//!
//! Where colonies go is `paint/placement.rs`; how far they can reach (and so how much
//! padding an image needs) is `paint/reach.rs`.
//!
//! # Coordinates
//!
//! Geometry lives in *reference pixels*: pixels of the 64-pixel sprite the parameters were
//! tuned on ([`REFERENCE_SIZE`]). `scale` is output pixels per reference pixel. Images are
//! windows onto an unbounded *canvas* of output pixels (see [`Rgba`]); a canvas pixel `p` is
//! at reference position `(p + 0.5) / scale`. Noise is sampled in reference units so a
//! design looks the same at any size; per-pixel specks use canvas pixels directly.

mod body;
mod brush;
mod field;
mod lighting;
mod overlays;
mod patch;
mod placement;
mod reach;
mod single;
mod slime;

use crate::Rgba;
use crate::params::MoldParams;

pub(crate) use field::paint_chunk;
pub use patch::PatchLayout;
pub(crate) use single::paint_colony;

/// One colony in a scene: where it sits, how far it has grown, and the seed that makes it
/// look different from its neighbours.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Colony {
    /// Centre in reference pixels.
    pub x: f32,
    pub y: f32,
    /// `0..1`; a colony that has not started yet is left out of the scene instead.
    pub growth: f32,
    pub seed: u32,
}

/// Paint `colonies` into `canvas` at `scale` output pixels per reference pixel, in the order
/// described at the top of this file.
///
/// `slime_cutoff` limits how far each slime ball reaches. The endless field needs that so a
/// pixel only depends on nearby colonies; single colonies and patches pass `None` and get
/// the exact, unlimited metaball field.
fn paint_scene(
    design: &MoldParams,
    scale: f32,
    colonies: &[Colony],
    slime_cutoff: Option<f32>,
    canvas: &mut Rgba,
) {
    canvas.scale = scale;
    if design.slime.enabled {
        let balls = slime::puddles(&design.slime, colonies);
        slime::paint(&design.slime, &balls, scale, slime_cutoff, canvas);
    }

    body::paint_bodies(design, scale, colonies, canvas);

    for colony in colonies {
        overlays::paint(design, colony, scale, canvas);
    }
}

#[cfg(test)]
mod tests;
