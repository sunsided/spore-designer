//! The preview to the right of the panel. Both kinds paint off the UI thread and show the
//! newest finished pictures meanwhile, so the window never freezes:
//!
//! - `preview/sheet.rs`: colony and patch views; every growth stage as one texture, the
//!   current stage large and all of them as a filmstrip below.
//! - `preview/field.rs`: the endless field; chunks painted around a panning camera.

mod field;
mod sheet;

use game_toolkit::prelude::*;
use spore_core::Rgb;

pub use field::FieldPreview;
pub use sheet::SheetPreview;

/// Where the preview may draw: right of the panel, the full window height (screen pixels).
#[derive(Clone, Copy)]
pub struct PreviewArea {
    pub left: f32,
    pub width: f32,
    pub height: f32,
}

/// Fill the whole frame with the background colour (`0..255` channels).
fn clear(painter: &mut Painter<'_>, background: Rgb) {
    let [r, g, b] = background.map(|channel| channel / 255.0);
    painter.clear([r, g, b, 1.0]);
}
