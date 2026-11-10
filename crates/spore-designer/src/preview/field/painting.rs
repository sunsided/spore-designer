//! [`ChunkPainter`]: paints field chunks on worker threads and caches the results.
//!
//! Every design change starts a new *generation*. Workers still busy with an older one
//! finish anyway (threads cannot be cancelled), but their results are dropped.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use spore_core::{render_chunk, stage_growth};

use super::ChunkKey;
use crate::design::Design;

/// Painted chunks kept in memory before off-screen ones are dropped.
const CACHE_BYTES: usize = 192 << 20;

/// A finished chunk: design generation, chunk, RGBA8 pixels.
type FinishedChunk = (u64, ChunkKey, Vec<u8>);

pub(super) struct ChunkPainter {
    /// The design being shown, shared with the workers.
    design: Option<Arc<Design>>,
    generation: u64,
    /// Painted chunks of the current generation, and their total size.
    cache: HashMap<ChunkKey, Vec<u8>>,
    cached_bytes: usize,
    /// Chunks of the current generation being painted.
    pending: HashSet<ChunkKey>,
    /// Workers running, of any generation, and how many may run at once.
    busy: usize,
    max_workers: usize,
    /// Workers send finished chunks here.
    results: (Sender<FinishedChunk>, Receiver<FinishedChunk>),
}

impl ChunkPainter {
    pub fn new() -> Self {
        Self {
            design: None,
            generation: 0,
            cache: HashMap::new(),
            cached_bytes: 0,
            pending: HashSet::new(),
            busy: 0,
            max_workers: std::thread::available_parallelism().map_or(4, |n| n.get()),
            results: channel(),
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The painted pixels of `chunk`, if finished.
    pub fn painted(&self, chunk: &ChunkKey) -> Option<&Vec<u8>> {
        self.cache.get(chunk)
    }

    // --------------------------------------------------------------------------------------

    /// Follow `design`: when it changed, start a new generation with an empty cache.
    /// Returns whether it changed.
    pub fn sync(&mut self, design: &Design) -> bool {
        if self.design.as_deref() == Some(design) {
            return false;
        }
        self.generation += 1;
        self.cache.clear();
        self.cached_bytes = 0;
        self.pending.clear();
        self.design = Some(Arc::new(design.clone()));
        true
    }

    /// Take every chunk the workers finished; ones from an older generation are dropped.
    pub fn collect_finished(&mut self) {
        while let Ok((generation, chunk, pixels)) = self.results.1.try_recv() {
            self.busy -= 1;
            if generation == self.generation {
                self.pending.remove(&chunk);
                self.cached_bytes += pixels.len();
                self.cache.insert(chunk, pixels);
            }
        }
    }

    /// Keep the cache within its budget. Drops off-screen chunks first, then the stages
    /// needed furthest in the future (playback runs forward), until it fits; chunks on
    /// screen (`visible`) always stay.
    pub fn evict(&mut self, visible: &[ChunkKey], stage: usize, stages: usize) {
        if self.cached_bytes <= CACHE_BYTES {
            return;
        }

        let wanted: HashSet<ChunkKey> = visible.iter().copied().collect();
        let on_screen: HashSet<(i32, i32)> = visible.iter().map(|k| (k.0, k.1)).collect();
        let stages_ahead = |chunk: &ChunkKey| (chunk.2 + stages - stage) % stages.max(1);

        let mut candidates: Vec<ChunkKey> = self.cache.keys().copied().collect();
        candidates.sort_by_key(|chunk| {
            (
                on_screen.contains(&(chunk.0, chunk.1)),
                std::cmp::Reverse(stages_ahead(chunk)),
            )
        });

        for chunk in candidates {
            if self.cached_bytes <= CACHE_BYTES {
                break;
            }
            if !wanted.contains(&chunk)
                && let Some(pixels) = self.cache.remove(&chunk)
            {
                self.cached_bytes -= pixels.len();
            }
        }
    }

    /// Start workers for visible chunks that are neither painted nor in progress, nearest
    /// to `view_center` (in chunks) first, as long as workers are free.
    pub fn start_missing(&mut self, visible: &[ChunkKey], view_center: (f32, f32)) {
        let Some(design) = self.design.clone() else {
            return;
        };

        let mut missing: Vec<ChunkKey> = visible
            .iter()
            .copied()
            .filter(|chunk| !self.cache.contains_key(chunk) && !self.pending.contains(chunk))
            .collect();
        let distance = |chunk: &ChunkKey| {
            (chunk.0 as f32 + 0.5 - view_center.0).hypot(chunk.1 as f32 + 0.5 - view_center.1)
        };
        missing.sort_by(|a, b| distance(a).total_cmp(&distance(b)));

        let free_workers = self.max_workers.saturating_sub(self.busy);
        for chunk in missing.into_iter().take(free_workers) {
            let (results, design, generation) =
                (self.results.0.clone(), design.clone(), self.generation);
            std::thread::spawn(move || {
                let growth = stage_growth(chunk.2, design.stages);
                let image = render_chunk(
                    &design.mold,
                    &design.field,
                    design.seed,
                    growth,
                    (chunk.0, chunk.1),
                    design.size,
                );
                // The preview may be gone by now; nothing to do then.
                let _ = results.send((generation, chunk, image.to_rgba8()));
            });
            self.busy += 1;
            self.pending.insert(chunk);
        }
    }
}
