//! The field camera: auto-pan drift and mouse dragging.

use game_toolkit::prelude::*;

use super::FieldPreview;
use crate::editor::Editor;

impl FieldPreview {
    /// Move the camera: drift by the auto-pan, and follow a mouse drag that started right of
    /// the panel (`panel_width` screen pixels wide).
    pub fn pan(&mut self, editor: &mut Editor, input: &Input, seconds: f32, panel_width: f32) {
        let heading = editor.pan_angle.to_radians();
        editor.camera[0] += heading.cos() * editor.pan_speed * seconds;
        editor.camera[1] -= heading.sin() * editor.pan_speed * seconds;

        if input.mouse_pressed(MouseButton::Left) {
            self.dragging = input.mouse_pos_window().0 > panel_width;
        }
        if !input.mouse_held(MouseButton::Left) {
            self.dragging = false;
        }
        if self.dragging {
            let (dx, dy) = input.mouse_delta();
            editor.camera = [editor.camera[0] - dx, editor.camera[1] - dy];
        }
    }
}
