//! The endless field preview: chunks painted on demand around the camera, cached per growth
//! stage, and shown through a small pool of reused textures (the toolkit cannot free one, so
//! the pool only grows to what fits on screen).

use std::collections::{HashMap, HashSet};

use game_toolkit::prelude::*;
use spore_core::{CHUNK_CELLS, render_chunk, stage_growth};

use crate::Design;

/// Chunk column, row and growth stage.
type Key = (i32, i32, usize);

/// Painted chunks kept in memory before off-screen ones are dropped.
const CACHE_BYTES: usize = 192 << 20;

#[derive(Default)]
pub struct FieldView {
    /// The design the cache was painted from.
    painted: Option<Design>,
    cache: HashMap<Key, Vec<u8>>,
    bytes: usize,
    /// Textures and the chunk each one currently shows.
    pool: Vec<(TextureId, Option<Key>)>,
    /// Side of a chunk in pixels, which is the size of every pool texture.
    chunk_px: usize,
}

impl FieldView {
    /// Make sure the chunks covering `view` (canvas pixels: left, top, width, height) at
    /// `stage` are painted and uploaded. Returns each visible texture with its canvas
    /// position. Paints at most a few chunks per call so panning stays responsive; the rest
    /// fill in over the next frames.
    pub fn prepare(
        &mut self,
        gfx: &mut Graphics,
        d: &Design,
        stage: usize,
        view: [f32; 4],
    ) -> Vec<(TextureId, [f32; 2])> {
        let chunk_px = d.size * CHUNK_CELLS;
        if self.painted.as_ref() != Some(d) {
            self.cache.clear();
            self.bytes = 0;
            if chunk_px != self.chunk_px {
                // shortcut: textures of the old chunk size are not freed (no toolkit call);
                // only changing the size leaks, fine for an editor session.
                self.pool.clear();
                self.chunk_px = chunk_px;
            }
            for slot in &mut self.pool {
                slot.1 = None;
            }
            self.painted = Some(d.clone());
        }

        let c = chunk_px as f32;
        let cols = (view[0] / c).floor() as i32..=((view[0] + view[2]) / c).floor() as i32;
        let rows = (view[1] / c).floor() as i32..=((view[1] + view[3]) / c).floor() as i32;
        let visible: Vec<Key> = rows
            .flat_map(|j| cols.clone().map(move |i| (i, j, stage)))
            .collect();

        // Paint missing chunks, nearest to the view centre first, a batch per frame.
        let centre = ((view[0] + view[2] * 0.5) / c, (view[1] + view[3] * 0.5) / c);
        let mut missing: Vec<Key> = visible
            .iter()
            .copied()
            .filter(|k| !self.cache.contains_key(k))
            .collect();
        missing.sort_by(|a, b| {
            let dist = |k: &Key| (k.0 as f32 + 0.5 - centre.0).hypot(k.1 as f32 + 0.5 - centre.1);
            dist(a).total_cmp(&dist(b))
        });
        let batch = std::thread::available_parallelism().map_or(4, |n| n.get());
        missing.truncate(batch);
        let g = stage_growth(stage, d.stages);
        let painted: Vec<(Key, Vec<u8>)> = std::thread::scope(|s| {
            let jobs: Vec<_> = missing
                .iter()
                .map(|&k| {
                    s.spawn(move || {
                        let img = render_chunk(&d.mold, &d.field, d.seed, g, (k.0, k.1), d.size);
                        (k, img.to_rgba8())
                    })
                })
                .collect();
            jobs.into_iter()
                .map(|j| j.join().expect("chunk painter panicked"))
                .collect()
        });
        let wanted: HashSet<Key> = visible.iter().copied().collect();
        for (k, px) in painted {
            self.bytes += px.len();
            self.cache.insert(k, px);
        }
        if self.bytes > CACHE_BYTES {
            self.cache.retain(|k, _| wanted.contains(k));
            self.bytes = self.cache.values().map(Vec::len).sum();
        }

        // Upload: reuse a texture already showing the chunk, else one showing something
        // off-screen, else grow the pool.
        let mut out = Vec::new();
        for k in visible {
            let Some(px) = self.cache.get(&k) else {
                continue;
            };
            let tex = if let Some(slot) = self.pool.iter().find(|s| s.1 == Some(k)) {
                slot.0
            } else if let Some(slot) = self
                .pool
                .iter_mut()
                .find(|s| s.1.is_none_or(|old| !wanted.contains(&old)))
            {
                gfx.update_texture_rgba(slot.0, px);
                slot.1 = Some(k);
                slot.0
            } else {
                let side = chunk_px as u32;
                let id = gfx.create_texture_rgba(side, side, px, Some("chunk"));
                self.pool.push((id, Some(k)));
                id
            };
            out.push((tex, [k.0 as f32 * c, k.1 as f32 * c]));
        }
        out
    }
}
