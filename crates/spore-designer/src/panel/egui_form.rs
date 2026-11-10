//! [`EguiForm`]: the form that draws each value as an egui widget.

use std::ops::RangeInclusive;

use game_toolkit::prelude::egui;
use spore_core::{ColorStop, Rgb};

use crate::fields::Form;

/// Draws every value it visits into `ui`: sliders, checkboxes, colour buttons, and a small
/// stop editor for gradients. Sections become collapsible headers.
pub(super) struct EguiForm<'a> {
    pub ui: &'a mut egui::Ui,
}

impl Form for EguiForm<'_> {
    fn section(&mut self, title: &str, body: &mut dyn FnMut(&mut dyn Form)) {
        egui::CollapsingHeader::new(title).show(self.ui, |ui| body(&mut EguiForm { ui }));
    }

    fn float(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) {
        self.ui.add(egui::Slider::new(value, range).text(label));
    }

    fn int(&mut self, label: &str, value: &mut u32, range: RangeInclusive<u32>) {
        self.ui.add(egui::Slider::new(value, range).text(label));
    }

    fn toggle(&mut self, label: &str, value: &mut bool) {
        self.ui.checkbox(value, label);
    }

    fn color(&mut self, label: &str, value: &mut Rgb) {
        self.ui.horizontal(|ui| {
            color_button(ui, value);
            ui.label(label);
        });
    }

    /// One row per stop (colour, position, remove), and a button to add one at the end.
    fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
        let ui = &mut *self.ui;
        ui.label("Gradient (centre to edge)");

        let mut remove = None;
        for (index, stop) in stops.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                color_button(ui, &mut stop.rgb);
                ui.add(egui::Slider::new(&mut stop.t, 0.0..=1.0));
                if ui.small_button("x").clicked() {
                    remove = Some(index);
                }
            });
        }
        if let Some(index) = remove {
            stops.remove(index);
        }

        if ui.small_button("+ stop").clicked() {
            let rgb = stops.last().map_or([255.0; 3], |stop| stop.rgb);
            stops.push(ColorStop { t: 1.0, rgb });
        }
    }
}

/// A colour picker button for an RGB `0..255` colour (egui works in `0..1`).
fn color_button(ui: &mut egui::Ui, rgb: &mut Rgb) {
    let mut unit = rgb.map(|channel| channel / 255.0);
    if ui.color_edit_button_rgb(&mut unit).changed() {
        *rgb = unit.map(|channel| channel * 255.0);
    }
}
