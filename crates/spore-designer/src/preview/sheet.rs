//! Colony and patch preview: every growth stage of the design, painted on a worker thread,
//! uploaded as one sprite-sheet texture, and drawn as one big stage plus a filmstrip.
//!
//! The big stage can also be shown relit by the cursor light or as one of its maps; that
//! picture is made by `sheet/relit.rs`. Drawing is `sheet/drawing.rs`.

mod drawing;
mod relit;

use std::sync::mpsc::{Receiver, Sender, channel};

use game_toolkit::prelude::*;
use spore_core::{Rgba, sheet};

use crate::design::Design;
use crate::work::Background;
use relit::RelitStage;

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
    /// The big stage when not shown as painted.
    relit: RelitStage,
}

impl SheetPreview {
    pub fn new() -> Self {
        Self {
            painted: None,
            frames: Vec::new(),
            painting: None,
            results: channel(),
            texture: None,
            relit: RelitStage::default(),
        }
    }

    /// The stages currently shown (what exports save).
    pub fn frames(&self) -> &[Rgba] {
        &self.frames
    }

    /// Keep the sheet in step with `design` without blocking the frame: take the newest
    /// finished stages, and while the design differs from them, paint it on a worker (one at
    /// a time; when it finishes, the design as it is by then gets painted next).
    pub fn update(&mut self, gfx: &mut Graphics, design: &Design, background: &mut Background) {
        while let Ok((painted, frames)) = self.results.1.try_recv() {
            self.painting = None;
            self.upload(gfx, painted, frames);
        }

        if self.painting.is_none() && self.painted.as_ref() != Some(design) {
            let (design, results) = (design.clone(), self.results.0.clone());
            self.painting = Some(design.clone());
            background.spawn(move || {
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
        self.relit.frames_changed(frames.len());
        self.frames = frames;
        self.painted = Some(design);
    }
}
