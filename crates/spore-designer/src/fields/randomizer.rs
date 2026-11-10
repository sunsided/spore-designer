//! [`Randomizer`]: the form behind "Randomize everything".

use std::ops::RangeInclusive;

use spore_core::noise::Rng;
use spore_core::{ColorStop, Rgb};

use super::Form;

/// Sets every value to a uniformly random one in its range, using the wrapped generator.
pub(super) struct Randomizer(pub Rng);

impl Form for Randomizer {
    fn section(&mut self, _title: &str, body: &mut dyn FnMut(&mut dyn Form)) {
        body(self);
    }

    fn float(&mut self, _label: &str, value: &mut f32, range: RangeInclusive<f32>) {
        *value = self.0.range(*range.start(), *range.end());
    }

    fn int(&mut self, _label: &str, value: &mut u32, range: RangeInclusive<u32>) {
        let span = range.end() - range.start() + 1;
        *value = range.start() + self.0.next_u32() % span;
    }

    fn toggle(&mut self, _label: &str, value: &mut bool) {
        *value = self.0.unit() < 0.5;
    }

    fn color(&mut self, _label: &str, value: &mut Rgb) {
        *value = [0; 3].map(|_| self.0.range(0.0, 255.0));
    }

    /// Same number of stops, spread from 0 to 1 at random positions, random colours.
    fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
        let count = stops.len();
        for (index, stop) in stops.iter_mut().enumerate() {
            stop.t = match index {
                0 => 0.0,
                _ if index == count - 1 => 1.0,
                _ => self.0.unit(),
            };
            self.color("", &mut stop.rgb);
        }
        stops.sort_by(|a, b| a.t.total_cmp(&b.t));
    }
}
