//! [`Clamp`]: the form that pulls every value into its range (used for loaded files).

use std::ops::RangeInclusive;

use spore_core::{ColorStop, Rgb};

use super::{Form, MAX_STOPS};

/// Clamps every value into its range; values that are not finite (NaN, infinity) take the
/// range start. Gradients are cut to [`MAX_STOPS`] and sorted.
pub(super) struct Clamp;

impl Form for Clamp {
    fn section(&mut self, _title: &str, body: &mut dyn FnMut(&mut dyn Form)) {
        body(self);
    }

    fn float(&mut self, _label: &str, value: &mut f32, range: RangeInclusive<f32>) {
        *value = if value.is_finite() {
            value.clamp(*range.start(), *range.end())
        } else {
            *range.start()
        };
    }

    fn int(&mut self, _label: &str, value: &mut u32, range: RangeInclusive<u32>) {
        *value = (*value).clamp(*range.start(), *range.end());
    }

    fn toggle(&mut self, _label: &str, _value: &mut bool) {}

    fn color(&mut self, _label: &str, value: &mut Rgb) {
        for channel in value {
            self.float("", channel, 0.0..=255.0);
        }
    }

    fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
        stops.truncate(MAX_STOPS);
        for stop in stops.iter_mut() {
            self.float("", &mut stop.t, 0.0..=1.0);
            self.color("", &mut stop.rgb);
        }
        stops.sort_by(|a, b| a.t.total_cmp(&b.t));
    }
}
