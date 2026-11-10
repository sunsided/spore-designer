//! Colony and patch preview: every growth stage of the design, painted on a worker thread,
//! uploaded as one sprite-sheet texture, and drawn as one big stage plus a filmstrip.

use std::sync::mpsc::{Receiver, Sender, channel};

use game_toolkit::prelude::*;
use spore_core::{Rgba, sheet};

use super::{PreviewArea, clear};
use crate::design::Design;
use crate::editor::Editor;

/// Share of the window height taken by the filmstrip.
const FILMSTRIP: f32 = 0.18;

/// Painted stages, and the design they show.
type PaintedSheet = (Design, Vec<Rgba>);

/// All stages as one GPU texture, wrapped into rows of `columns` stages.
struct SheetTexture {
    id: TextureId,
    size: [u32; 2],
    columns: usize,
}

pub struct SheetPreview {
    /// The design `frames` show, and the frames themselves.
    painted: Option<Design>,
    frames: Vec<Rgba>,
    /// The design a worker is painting right now, if any.
    painting: Option<Design>,
    /// Workers send finished sheets here.
    results: (Sender<PaintedSheet>, Receiver<PaintedSheet>),
    texture: Option<SheetTexture>,
}

impl SheetPreview {
    pub fn new() -> Self {
        Self {
            painted: None,
            frames: Vec::new(),
            painting: None,
            results: channel(),
            texture: None,
        }
    }

    /// The stages currently shown (what exports save).
    pub fn frames(&self) -> &[Rgba] {
        &self.frames
    }

    // --------------------------------------------------------------------------------------
    // Keeping up with the design

    /// Keep the sheet in step with `design` without blocking the frame: take the newest
    /// finished stages, and while the design differs from them, paint it on a worker (one at
    /// a time; when it finishes, the design as it is by then gets painted next).
    pub fn update(&mut self, gfx: &mut Graphics, design: &Design) {
        while let Ok((painted, frames)) = self.results.1.try_recv() {
            self.painting = None;
            self.upload(gfx, painted, frames);
        }

        if self.painting.is_none() && self.painted.as_ref() != Some(design) {
            let (design, results) = (design.clone(), self.results.0.clone());
            self.painting = Some(design.clone());
            std::thread::spawn(move || {
                let frames = design.frames();
                // The app may be closing; nothing to do then.
                let _ = results.send((design, frames));
            });
        }
    }

    /// Upload painted stages as one texture, wrapped into rows so it stays within the GPU's
    /// largest texture.
    fn upload(&mut self, gfx: &mut Graphics, design: Design, frames: Vec<Rgba>) {
        // Frames are padded past the cell when the design grows out of it.
        let frame_side = frames[0].side;
        let max_side = gfx.device().limits().max_texture_dimension_2d as usize;
        let columns = (max_side / frame_side).clamp(1, frames.len());
        let rows = frames.len().div_ceil(columns);
        let pixels = sheet(&frames, columns);
        let size = [(frame_side * columns) as u32, (frame_side * rows) as u32];

        match &self.texture {
            Some(texture) if texture.size == size => {
                gfx.update_texture_rgba(texture.id, &pixels);
                self.texture = Some(SheetTexture {
                    id: texture.id,
                    size,
                    columns,
                });
            }
            // shortcut: the old texture is not freed (the toolkit has no release call); only
            // size/stage-count changes leak, fine for an editor session.
            _ => {
                let id = gfx.create_texture_rgba(size[0], size[1], &pixels, Some("sheet"));
                self.texture = Some(SheetTexture { id, size, columns });
            }
        }
        self.frames = frames;
        self.painted = Some(design);
    }

    // --------------------------------------------------------------------------------------
    // Drawing

    /// Draw the current stage as large as fits, the cell outline when the mould spills out
    /// of it, and the filmstrip of all stages below.
    pub fn draw(&self, gfx: &mut Graphics, frame: &mut Frame, editor: &Editor, area: PreviewArea) {
        let mut painter = frame.painter(gfx);
        clear(&mut painter, editor.background);
        let (Some(texture), Some(design)) = (&self.texture, &self.painted) else {
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
        draw_stage(&mut painter, texture, frame_side, stage, position, side);
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
