//! The overlay pass: what sits on one colony's body, painted after all bodies. Each layer is
//! its own file in `overlays/` and draws with its own random stream (see
//! [`ColonyBrush::rng`]), so toggling one never moves another.

mod drops;
mod glisten;
mod hyphae;
mod rosettes;
mod sporangia;

use super::Colony;
use super::brush::ColonyBrush;
use crate::Rgba;
use crate::params::MoldParams;

/// Highlight colour for droplets and spore heads.
const WHITE: [f32; 3] = [255.0, 255.0, 250.0];

/// Paint every enabled overlay of `colony`, bottom to top.
pub(super) fn paint(design: &MoldParams, colony: &Colony, scale: f32, canvas: &mut Rgba) {
    let brush = ColonyBrush::new(design, colony, scale);

    if design.rosettes.enabled {
        rosettes::paint(&brush, canvas);
    }
    if design.hyphae.enabled {
        hyphae::paint(&brush, canvas);
    }
    if design.sporangia.enabled {
        sporangia::paint(&brush, canvas);
    }
    if design.drops.enabled && colony.growth > design.drops.min_growth {
        drops::paint(&brush, canvas);
    }
    if design.glisten.enabled {
        glisten::paint(&brush, canvas);
    }
}
