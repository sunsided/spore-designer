//! Drawing the sheet preview: the big stage (baked, relit or mapped), the cell outline, and
//! the filmstrip.

use game_toolkit::prelude::*;

use super::{SheetPreview, SheetTexture};
use crate::editor::{Editor, LightMode};
use crate::preview::{PreviewArea, clear};

/// Share of the window height taken by the filmstrip.
const FILMSTRIP: f32 = 0.18;

impl SheetPreview {
    /// Draw the current stage as large as fits, the cell outline when the mould spills out
    /// of it, and the filmstrip of all stages below.
    pub fn draw(
        &mut self,
        gfx: &mut Graphics,
        frame: &mut Frame,
        editor: &Editor,
        area: PreviewArea,
    ) {
        let (Some(texture), Some(design)) = (&self.texture, &self.painted) else {
            let mut painter = frame.painter(gfx);
            clear(&mut painter, editor.background);
            return;
        };

        let frame_side = self.frames[0].side;
        let stage = editor.stage_index(design.stages);
        let strip_height = area.height * FILMSTRIP;
        let main_height = area.height - strip_height;

        // The current stage, centred and as large as fits.
        let side = (area.width.min(main_height) * 0.92).max(1.0);
        let position = [
            area.left + (area.width - side) * 0.5,
            (main_height - side) * 0.5,
        ];

        // Relit or map view: made (and uploaded) before drawing starts.
        let relit = (editor.light.mode != LightMode::Baked).then(|| {
            let light_position = light_over_frame(editor.cursor, position, side, frame_side);
            self.relit.update(
                gfx,
                &self.frames,
                stage,
                editor.light,
                design.relief,
                light_position,
            )
        });

        let mut painter = frame.painter(gfx);
        clear(&mut painter, editor.background);
        match relit {
            Some(relit) => painter.sprite(relit, position, [side, side]),
            None => draw_stage(&mut painter, texture, frame_side, stage, position, side),
        }
        if frame_side > design.size {
            let cell = side * design.size as f32 / frame_side as f32;
            draw_cell_outline(&mut painter, position, side, cell);
        }

        let band = (main_height, strip_height);
        draw_filmstrip(
            &mut painter,
            texture,
            frame_side,
            design.stages,
            stage,
            area,
            band,
        );
    }
}

/// Where the cursor light hangs over the frame, frame pixels: under the mouse, or over the
/// top-left quarter until the mouse has been over the preview.
fn light_over_frame(
    cursor: Option<[f32; 2]>,
    position: [f32; 2],
    side: f32,
    frame_side: usize,
) -> [f32; 2] {
    let frame_size = frame_side as f32;
    match cursor {
        Some([x, y]) => [
            (x - position[0]) / side * frame_size,
            (y - position[1]) / side * frame_size,
        ],
        None => [frame_size * 0.25; 2],
    }
}

/// Stage `stage` of the sheet at `position`, `side` screen pixels square.
fn draw_stage(
    painter: &mut Painter<'_>,
    texture: &SheetTexture,
    frame_side: usize,
    stage: usize,
    position: [f32; 2],
    side: f32,
) {
    let frame_size = frame_side as f32;
    painter.sprite_region(
        texture.id,
        sheet_position(texture, frame_side, stage),
        [frame_size, frame_size],
        position,
        [side, side],
    );
}

/// Where stage `stage` sits in the sheet texture (pixels, top-left).
fn sheet_position(texture: &SheetTexture, frame_side: usize, stage: usize) -> [f32; 2] {
    [
        (stage % texture.columns * frame_side) as f32,
        (stage / texture.columns * frame_side) as f32,
    ]
}

/// A faint square of side `cell` centred in the stage drawn at `position`: the cell (or patch
/// area) the mould grew out of.
fn draw_cell_outline(painter: &mut Painter<'_>, position: [f32; 2], side: f32, cell: f32) {
    let (left, top) = (
        position[0] + (side - cell) * 0.5,
        position[1] + (side - cell) * 0.5,
    );
    let line = [1.0, 1.0, 1.0, 0.25];
    painter.rect([left, top], [cell, 1.0], line);
    painter.rect([left, top + cell - 1.0], [cell, 1.0], line);
    painter.rect([left, top], [1.0, cell], line);
    painter.rect([left + cell - 1.0, top], [1.0, cell], line);
}

/// All stages side by side in the horizontal `band` (top, height) under the big one, the
/// current stage highlighted.
fn draw_filmstrip(
    painter: &mut Painter<'_>,
    texture: &SheetTexture,
    frame_side: usize,
    stages: usize,
    current: usize,
    area: PreviewArea,
    (top, height): (f32, f32),
) {
    let frame_size = frame_side as f32;
    let cell = (area.width / stages as f32).min(height * 0.9);
    let strip_width = cell * stages as f32;
    let strip_top = top + (height - cell) * 0.5;
    let strip_left = area.left + (area.width - strip_width) * 0.5;

    painter.rect(
        [strip_left + current as f32 * cell, strip_top],
        [cell, cell],
        [1.0, 1.0, 1.0, 0.08],
    );
    for stage in 0..stages {
        painter.sprite_region(
            texture.id,
            sheet_position(texture, frame_side, stage),
            [frame_size, frame_size],
            [strip_left + stage as f32 * cell, strip_top],
            [cell, cell],
        );
    }
}
