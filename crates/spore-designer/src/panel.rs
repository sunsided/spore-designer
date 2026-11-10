//! The parameter panel on the left. It edits the [`Editor`] in place; the previews notice
//! the changed design on their own.
//!
//! [`show`] draws the panel top to bottom: file buttons, presets, seed and size, playback,
//! view settings, light and maps, then every mould value (listed in `fields.rs`, drawn by
//! `panel/egui_form.rs`).

mod egui_form;
mod sections;

use game_toolkit::prelude::egui;

use crate::editor::{Action, Editor};
use crate::fields::{self, Form};
use egui_form::EguiForm;
use sections::{
    file_buttons, light_settings, playback, presets_and_randomize, seed_size_and_stages,
    view_settings,
};

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
