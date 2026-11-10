//! The parameter panel on the left. It edits the [`Editor`] in place; the previews notice
//! the changed design on their own.
//!
//! [`show`] draws the panel top to bottom: file buttons, presets, seed and size, playback,
//! view settings, light and maps, then every mould value (listed in `fields.rs`, drawn by
//! `panel/egui_form.rs`).

mod egui_form;

use game_toolkit::prelude::egui;
use spore_core::presets;

use crate::design::View;
use crate::editor::{Action, Editor, LightMode};
use crate::fields::{self, Form, RELIEF, SIZES, STAGES};
use egui_form::EguiForm;

/// Draw the whole panel. Returns the file operation the user asked for, if any; `status`
/// is the message from the last one.
pub fn show(ui: &mut egui::Ui, editor: &mut Editor, status: &str) -> Option<Action> {
    ui.heading("Spore Designer");
    let action = file_buttons(ui, status);
    ui.separator();

    presets_and_randomize(ui, editor);
    seed_size_and_stages(ui, editor);
    playback(ui, editor);

    let mut form = EguiForm { ui };
    form.color("background", &mut editor.background);
    view_settings(&mut form, editor);
    light_settings(&mut form, editor);
    form.ui.separator();

    fields::mold(&mut form, &mut editor.design.mold);
    action
}

// ------------------------------------------------------------------------------------------
// Sections

/// Load / save / export buttons, and the last operation's message under them.
fn file_buttons(ui: &mut egui::Ui, status: &str) -> Option<Action> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        if ui.button("Load…").clicked() {
            action = Some(Action::Load);
        }
        if ui.button("Save…").clicked() {
            action = Some(Action::Save);
        }
        if ui.button("Export stage…").clicked() {
            action = Some(Action::ExportStage);
        }
        if ui.button("Export sheet…").clicked() {
            action = Some(Action::ExportSheet);
        }
    });
    if !status.is_empty() {
        ui.small(status);
    }
    action
}

/// One button per preset (replaces the mould, keeps the view), plus "Randomize everything".
fn presets_and_randomize(ui: &mut egui::Ui, editor: &mut Editor) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Preset:");
        for (name, preset) in presets::ALL {
            if ui.button(*name).clicked() {
                editor.design.mold = preset();
            }
        }
    });

    if ui
        .button("Randomize everything")
        .on_hover_text("New random value for every parameter and the seed")
        .clicked()
    {
        fields::randomize(&mut editor.design, &mut editor.dice);
    }
}

fn seed_size_and_stages(ui: &mut egui::Ui, editor: &mut Editor) {
    let design = &mut editor.design;
    ui.horizontal(|ui| {
        ui.label("Seed");
        ui.add(egui::DragValue::new(&mut design.seed));
        if ui.button("Random").on_hover_text("Random seed").clicked() {
            // One step of a linear congruential generator: a new, unrelated-looking seed.
            design.seed = design
                .seed
                .wrapping_mul(747_796_405)
                .wrapping_add(2_891_336_453)
                >> 8;
        }
    });
    ui.add(egui::Slider::new(&mut design.size, SIZES).text("size px"));
    ui.add(egui::Slider::new(&mut design.stages, STAGES).text("stages"));
}

/// Play / pause, the stage slider, and the playback speed.
fn playback(ui: &mut egui::Ui, editor: &mut Editor) {
    ui.horizontal(|ui| {
        let label = if editor.playing { "Pause" } else { "Play" };
        if ui.button(label).clicked() {
            editor.playing = !editor.playing;
        }

        // The playhead is fractional; the slider shows whole stages and only writes back
        // when dragged (a stepped slider would snap the playhead back every frame).
        let mut shown = editor.stage as usize;
        if ui
            .add(egui::Slider::new(&mut shown, 0..=editor.design.stages - 1).text("stage"))
            .changed()
        {
            editor.stage = shown as f32;
        }
    });
    ui.add(egui::Slider::new(&mut editor.fps, 1.0..=30.0).text("fps"));
}

/// The view switch, and the settings of the chosen view.
fn view_settings(form: &mut EguiForm<'_>, editor: &mut Editor) {
    let design = &mut editor.design;
    form.ui.horizontal(|ui| {
        ui.label("View:");
        ui.selectable_value(&mut design.view, View::Colony, "Colony");
        ui.selectable_value(&mut design.view, View::Patch, "Patch");
        ui.selectable_value(&mut design.view, View::Field, "Endless field");
    });

    match design.view {
        View::Colony => {}
        View::Patch => fields::patch(form, &mut design.patch),
        View::Field => {
            fields::field(form, &mut design.field);
            form.float("pan speed (px/s)", &mut editor.pan_speed, 0.0..=400.0);
            form.float("pan direction (deg)", &mut editor.pan_angle, 0.0..=360.0);
            form.ui.small("Drag the preview to move around.");
        }
    }
}

/// What the big stage shows (lit as painted, by the cursor light, or as one of the maps),
/// the cursor light's settings, and the relief strength used for normals and exports.
fn light_settings(form: &mut EguiForm<'_>, editor: &mut Editor) {
    if editor.design.view == View::Field {
        return;
    }
    let light = &mut editor.light;
    form.ui.horizontal_wrapped(|ui| {
        ui.label("Show:");
        for (mode, label) in [
            (LightMode::Baked, "Baked"),
            (LightMode::Cursor, "Cursor light"),
            (LightMode::Albedo, "Albedo"),
            (LightMode::Height, "Height"),
            (LightMode::Normals, "Normals"),
            (LightMode::Gloss, "Specular"),
        ] {
            ui.selectable_value(&mut light.mode, mode, label);
        }
    });

    if light.mode == LightMode::Cursor {
        form.float("light height", &mut light.height, 2.0..=120.0);
        form.float("ambient", &mut light.ambient, 0.0..=1.0);
        form.float("specular", &mut light.specular, 0.0..=3.0);
        form.float("shininess", &mut light.shininess, 1.0..=200.0);
        form.ui
            .small("The light follows the mouse over the preview.");
    }
    form.float(
        "relief (normal strength)",
        &mut editor.design.relief,
        RELIEF,
    );
}
