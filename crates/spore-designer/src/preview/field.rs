//! Endless field preview: a camera panning over an infinite canvas of colonies.
//!
//! The canvas is cut into square chunks (`spore_core::render_chunk`), shown at one canvas
//! pixel per screen pixel. Each frame [`FieldPreview::draw`]:
//!
//! 1. works out which chunks the camera sees at the current stage,
//! 2. collects chunks the workers finished and starts the missing ones
//!    (`field/painting.rs`, never blocking),
//! 3. puts painted chunks into reused GPU textures (`field/texture_pool.rs`),
//! 4. with a light mode on, relights the chunks that have their maps (all of them, every
//!    frame: the light and the camera move), in parallel,
//! 5. draws them.
//!
//! A chunk still being painted after a design change keeps showing its previous version.
//!
//! The camera (drift and drag) is `field/camera.rs`; relighting is `field/relight.rs`.

mod camera;
mod painting;
mod relight;
mod relit_textures;
mod texture_pool;

use game_toolkit::prelude::*;
use spore_core::CHUNK_CELLS;

use super::{PreviewArea, clear};
use crate::editor::{Editor, LightMode};
use crate::work::Background;
use painting::ChunkPainter;
use relit_textures::RelitTextures;
use texture_pool::TexturePool;

/// A chunk at one growth stage: column, row, stage.
type ChunkKey = (i32, i32, usize);

pub struct FieldPreview {
    painter: ChunkPainter,
    textures: TexturePool,
    relit: RelitTextures,
    /// A mouse drag started on the preview (not on the panel) and is still held.
    dragging: bool,
}

impl FieldPreview {
    pub fn new() -> Self {
        Self {
            painter: ChunkPainter::new(),
            textures: TexturePool::default(),
            relit: RelitTextures::default(),
            dragging: false,
        }
    }

    pub fn draw(
        &mut self,
        gfx: &mut Graphics,
        frame: &mut Frame,
        editor: &Editor,
        area: PreviewArea,
        background: &mut Background,
    ) {
        let design = &editor.design;
        let chunk_side = design.size * CHUNK_CELLS;
        let chunk_size = chunk_side as f32;
        let stage = editor.stage_index(design.stages);

        // The camera sits at the centre of the area; `origin` is the canvas pixel at the
        // window's top-left corner, `view` the canvas rectangle the area shows.
        let origin = [
            editor.camera[0] - area.width * 0.5 - area.left,
            editor.camera[1] - area.height * 0.5,
        ];
        let view = [origin[0] + area.left, origin[1], area.width, area.height];
        let visible = visible_chunks(view, chunk_size, stage);

        // Paint (in the background) and upload.
        let lighting = editor.light.mode != LightMode::Baked;
        if self.painter.sync(design, lighting) {
            self.textures.design_changed(chunk_side);
        }
        self.painter.collect_finished();
        self.painter.evict(&visible, stage, design.stages);
        let view_center = (
            (view[0] + view[2] * 0.5) / chunk_size,
            (view[1] + view[3] * 0.5) / chunk_size,
        );
        self.painter
            .start_missing(&visible, view_center, background);
        let mut shown = self.textures.show(gfx, &visible, &self.painter);
        if lighting {
            self.relight(gfx, editor, origin, chunk_side, &mut shown);
        }

        // Draw, at whole pixels so chunk edges meet exactly.
        let mut screen = frame.painter(gfx);
        clear(&mut screen, editor.background);
        for (texture, (column, row, _)) in shown {
            let at = [
                (column as f32 * chunk_size - origin[0]).round(),
                (row as f32 * chunk_size - origin[1]).round(),
            ];
            screen.sprite(texture, at, [chunk_size, chunk_size]);
        }
    }
}

/// The chunks (at `stage`) overlapping the canvas rectangle `view` (left, top, width,
/// height), row by row.
fn visible_chunks(view: [f32; 4], chunk_size: f32, stage: usize) -> Vec<ChunkKey> {
    let columns =
        (view[0] / chunk_size).floor() as i32..=((view[0] + view[2]) / chunk_size).floor() as i32;
    let rows =
        (view[1] / chunk_size).floor() as i32..=((view[1] + view[3]) / chunk_size).floor() as i32;
    rows.flat_map(|row| columns.clone().map(move |column| (column, row, stage)))
        .collect()
}
