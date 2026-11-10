//! The endless field preview: chunks painted on worker threads around the camera, cached per
//! growth stage, and shown through a small pool of reused textures (the toolkit cannot free
//! one, so the pool only grows to what fits on screen).
//!
//! Painting never blocks the frame. After a design change the previous chunks stay on screen
//! until their replacements arrive.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use game_toolkit::prelude::*;
use spore_core::{CHUNK_CELLS, render_chunk, stage_growth};

use crate::Design;

/// Chunk column, row and growth stage.
type Key = (i32, i32, usize);

/// Painted chunks kept in memory before off-screen ones are dropped.
const CACHE_BYTES: usize = 192 << 20;

/// A finished chunk: design generation, chunk, RGBA8 pixels.
type Done = (u64, Key, Vec<u8>);

pub struct FieldView {
    /// The design the cache belongs to, and a counter bumped whenever it changes.
    painted: Option<Arc<Design>>,
    generation: u64,
    cache: HashMap<Key, Vec<u8>>,
    bytes: usize,
    /// Chunks of the current generation being painted.
    pending: HashSet<Key>,
    /// Workers running, of any generation.
    busy: usize,
    workers: usize,
    tx: Sender<Done>,
    rx: Receiver<Done>,
    /// Textures and the (generation, chunk) each one currently shows.
    pool: Vec<(TextureId, Option<(u64, Key)>)>,
    /// Side of a chunk in pixels, which is the size of every pool texture.
    chunk_px: usize,
}

impl Default for FieldView {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            painted: None,
            generation: 0,
            cache: HashMap::new(),
            bytes: 0,
            pending: HashSet::new(),
            busy: 0,
            workers: std::thread::available_parallelism().map_or(4, |n| n.get()),
            tx,
            rx,
            pool: Vec::new(),
            chunk_px: 0,
        }
    }
}

impl FieldView {
    /// Queue painting of the chunks covering `view` (canvas pixels: left, top, width, height)
    /// at `stage`, upload whatever has finished, and return each texture to draw with its
    /// canvas position. Chunks still painting show their previous version, if any.
    pub fn prepare(
        &mut self,
        gfx: &mut Graphics,
        d: &Design,
        stage: usize,
        view: [f32; 4],
    ) -> Vec<(TextureId, [f32; 2])> {
        let chunk_px = d.size * CHUNK_CELLS;
        if self.painted.as_deref() != Some(d) {
            self.generation += 1;
            self.cache.clear();
            self.bytes = 0;
            self.pending.clear();
            if chunk_px != self.chunk_px {
                // shortcut: textures of the old chunk size are not freed (no toolkit call);
                // only changing the size leaks, fine for an editor session.
                self.pool.clear();
                self.chunk_px = chunk_px;
            }
            self.painted = Some(Arc::new(d.clone()));
        }
        let design = self.painted.clone().expect("set above");

        let c = chunk_px as f32;
        let cols = (view[0] / c).floor() as i32..=((view[0] + view[2]) / c).floor() as i32;
        let rows = (view[1] / c).floor() as i32..=((view[1] + view[3]) / c).floor() as i32;
        let visible: Vec<Key> = rows
            .flat_map(|j| cols.clone().map(move |i| (i, j, stage)))
            .collect();
        let wanted: HashSet<Key> = visible.iter().copied().collect();

        // Collect finished chunks; ones from an older design are dropped.
        while let Ok((generation, k, px)) = self.rx.try_recv() {
            self.busy -= 1;
            if generation == self.generation {
                self.pending.remove(&k);
                self.bytes += px.len();
                self.cache.insert(k, px);
            }
        }
        if self.bytes > CACHE_BYTES {
            self.cache.retain(|k, _| wanted.contains(k));
            self.bytes = self.cache.values().map(Vec::len).sum();
        }

        // Start missing chunks, nearest to the view centre first.
        let centre = ((view[0] + view[2] * 0.5) / c, (view[1] + view[3] * 0.5) / c);
        let mut missing: Vec<Key> = visible
            .iter()
            .copied()
            .filter(|k| !self.cache.contains_key(k) && !self.pending.contains(k))
            .collect();
        missing.sort_by(|a, b| {
            let dist = |k: &Key| (k.0 as f32 + 0.5 - centre.0).hypot(k.1 as f32 + 0.5 - centre.1);
            dist(a).total_cmp(&dist(b))
        });
        for k in missing
            .into_iter()
            .take(self.workers.saturating_sub(self.busy))
        {
            let (tx, d, generation) = (self.tx.clone(), design.clone(), self.generation);
            std::thread::spawn(move || {
                let g = stage_growth(k.2, d.stages);
                let img = render_chunk(&d.mold, &d.field, d.seed, g, (k.0, k.1), d.size);
                // The view may be gone by now; nothing to do then.
                let _ = tx.send((generation, k, img.to_rgba8()));
            });
            self.busy += 1;
            self.pending.insert(k);
        }

        // Upload: reuse the texture already showing the chunk, else one showing something
        // off-screen (or outdated, and not on screen this frame), else grow the pool.
        let generation = self.generation;
        let mut used = vec![false; self.pool.len()];
        let mut out = Vec::new();
        for k in visible {
            let current = Some((generation, k));
            if let Some(i) = self.pool.iter().position(|s| s.1 == current) {
                used[i] = true;
                out.push((self.pool[i].0, k));
                continue;
            }
            let Some(px) = self.cache.get(&k) else {
                // Not painted yet: show the previous design's version if one is left.
                if let Some(i) = self.pool.iter().position(|s| s.1.is_some_and(|o| o.1 == k)) {
                    used[i] = true;
                    out.push((self.pool[i].0, k));
                }
                continue;
            };
            let free = self.pool.iter().enumerate().position(|(i, s)| {
                !used[i]
                    && s.1
                        .is_none_or(|(g, old)| !wanted.contains(&old) || g != generation)
            });
            let tex = if let Some(i) = free {
                gfx.update_texture_rgba(self.pool[i].0, px);
                self.pool[i].1 = current;
                used[i] = true;
                self.pool[i].0
            } else {
                let side = chunk_px as u32;
                let id = gfx.create_texture_rgba(side, side, px, Some("chunk"));
                self.pool.push((id, current));
                used.push(true);
                id
            };
            out.push((tex, k));
        }
        out.into_iter()
            .map(|(tex, k)| (tex, [k.0 as f32 * c, k.1 as f32 * c]))
            .collect()
    }
}
