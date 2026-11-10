//! Every editable value of a design, with its allowed range, listed exactly once.
//!
//! The lists ([`mold`], [`patch`], [`field`], in `fields/lists.rs`) walk the values through
//! the [`Form`] trait.
//! Three forms use them, so sliders, randomizing and loading can never disagree on a range:
//!
//! - `panel/egui_form.rs` draws a slider, checkbox or colour button per value;
//! - `fields/randomizer.rs` sets each value to a random one in its range ([`randomize`]);
//! - `fields/clamp.rs` pulls each value into its range ([`sanitize`], for loaded files).

mod clamp;
mod lists;
mod randomizer;

use std::ops::RangeInclusive;

use spore_core::noise::Rng;
use spore_core::{ColorStop, Rgb};

use crate::design::Design;
use clamp::Clamp;
use randomizer::Randomizer;

pub use lists::{field, mold, patch};

/// Allowed cell sizes in pixels, and stage counts.
pub const SIZES: RangeInclusive<usize> = 32..=512;
pub const STAGES: RangeInclusive<usize> = 1..=24;

/// Allowed normal-map strengths.
pub const RELIEF: RangeInclusive<f32> = 0.0..=4.0;

/// Most gradient stops a design may bring along.
const MAX_STOPS: usize = 16;

/// Something that visits every editable value of a design.
pub trait Form {
    /// A group of values under a title; call `body` with the form to visit them.
    fn section(&mut self, title: &str, body: &mut dyn FnMut(&mut dyn Form));
    fn float(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>);
    fn int(&mut self, label: &str, value: &mut u32, range: RangeInclusive<u32>);
    fn toggle(&mut self, label: &str, value: &mut bool);
    /// An RGB colour, channels `0..255`.
    fn color(&mut self, label: &str, value: &mut Rgb);
    fn gradient(&mut self, stops: &mut Vec<ColorStop>);
}

/// Bring every value of a design from outside (a loaded file) into its slider's range, so
/// absurd values cannot exhaust memory or hang the painter.
pub fn sanitize(design: &mut Design) {
    design.size = design.size.clamp(*SIZES.start(), *SIZES.end());
    design.stages = design.stages.clamp(*STAGES.start(), *STAGES.end());
    mold(&mut Clamp, &mut design.mold);
    patch(&mut Clamp, &mut design.patch);
    field(&mut Clamp, &mut design.field);
    Clamp.float("relief", &mut design.relief, RELIEF);
}

/// Give every mould value and the seed a uniformly random value in its range. `dice` is the
/// random-number generator's state; it moves on so the next call differs.
pub fn randomize(design: &mut Design, dice: &mut u64) {
    let mut randomizer = Randomizer(Rng(*dice));
    mold(&mut randomizer, &mut design.mold);
    design.seed = randomizer.0.next_u32() >> 8;
    *dice = randomizer.0.0;
}

#[cfg(test)]
mod tests;
