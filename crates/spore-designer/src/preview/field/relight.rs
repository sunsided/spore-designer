//! Lighting the field: every visible chunk with lighting maps, relit (or mapped) each frame.

use game_toolkit::prelude::*;

use super::painting::ChunkMaps;
use super::{ChunkKey, FieldPreview};
use crate::design::Design;
use crate::editor::Editor;
use crate::preview::lighting::shade;

impl FieldPreview {
    /// Replace the baked texture of every shown chunk that has its lighting maps by one lit
    /// (or mapped) the way the light mode asks. The cursor's canvas position is the light's.
    pub(super) fn relight(
        &mut self,
        gfx: &mut Graphics,
        editor: &Editor,
        origin: [f32; 2],
        chunk_side: usize,
        shown: &mut [(TextureId, ChunkKey)],
    ) {
        let light_canvas = match editor.cursor {
            Some([x, y]) => [origin[0] + x, origin[1] + y],
            None => editor.camera,
        };
        let height_scale = height_scale(&editor.design);

        // Indices into `shown` of the chunks that can be relit, and their maps.
        let lit: Vec<(usize, &ChunkMaps)> = shown
            .iter()
            .enumerate()
            .filter_map(|(index, (_, chunk))| {
                let maps = self.painter.painted(chunk)?.maps.as_ref()?;
                Some((index, maps))
            })
            .collect();

        // One thread per chunk; each chunk is small.
        let pictures: Vec<Vec<u8>> = std::thread::scope(|scope| {
            let jobs: Vec<_> = lit
                .iter()
                .map(|(_, maps)| {
                    let light_position = [
                        light_canvas[0] - maps.image.left as f32,
                        light_canvas[1] - maps.image.top as f32,
                    ];
                    scope.spawn(move || {
                        shade(
                            &maps.image,
                            &maps.normals,
                            editor.light,
                            light_position,
                            height_scale,
                            1,
                        )
                    })
                })
                .collect();
            jobs.into_iter()
                .map(|job| job.join().expect("relighting panicked"))
                .collect()
        });

        let textures = self.relit.upload(gfx, chunk_side, &pictures);
        for ((index, _), texture) in lit.iter().zip(textures) {
            shown[*index].0 = texture;
        }
    }
}

/// White in the field's height view, reference pixels. Every chunk uses the same scale so
/// they match (unlike a single stage, which is scaled to its own tallest point): about the
/// height of a grown colony's dome plus what sits on it.
fn height_scale(design: &Design) -> f32 {
    let shape = &design.mold.shape;
    ((shape.radius_base + shape.radius_growth) * 0.5).max(1.0)
}
