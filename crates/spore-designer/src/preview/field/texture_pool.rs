//! [`TexturePool`]: GPU textures for field chunks, reused instead of created per chunk.
//!
//! The toolkit cannot free a texture, so panning forever must not create one per chunk.
//! The pool only grows to what is on screen at once; textures move between chunks.

use game_toolkit::prelude::*;

use super::ChunkKey;
use super::painting::ChunkPainter;

/// A texture and the (generation, chunk) it currently shows, if any.
type Slot = (TextureId, Option<(u64, ChunkKey)>);

#[derive(Default)]
pub(super) struct TexturePool {
    slots: Vec<Slot>,
    /// Side of every texture in the pool, pixels.
    chunk_side: usize,
}

impl TexturePool {
    /// The design changed. Textures keep their old pictures (shown until replacements are
    /// painted), unless the chunk size changed and they no longer fit.
    pub fn design_changed(&mut self, chunk_side: usize) {
        if chunk_side != self.chunk_side {
            // shortcut: textures of the old chunk size are not freed (no toolkit call);
            // only changing the size leaks, fine for an editor session.
            self.slots.clear();
            self.chunk_side = chunk_side;
        }
    }

    /// A texture for each visible chunk that has a picture: its current one if uploaded,
    /// else uploaded now into a free slot, else (still painting) the previous design's
    /// version if a slot still holds it. Returns `(texture, chunk)` pairs to draw.
    pub fn show(
        &mut self,
        gfx: &mut Graphics,
        visible: &[ChunkKey],
        painter: &ChunkPainter,
    ) -> Vec<(TextureId, ChunkKey)> {
        let generation = painter.generation();
        // Slots already drawn this frame must not be overwritten by a later chunk.
        let mut used = vec![false; self.slots.len()];
        let mut shown = Vec::new();

        for &chunk in visible {
            let current = Some((generation, chunk));
            if let Some(index) = self.slots.iter().position(|slot| slot.1 == current) {
                used[index] = true;
                shown.push((self.slots[index].0, chunk));
                continue;
            }

            let Some(pixels) = painter.painted(&chunk).map(|painted| &painted.lit) else {
                if let Some(index) = self.previous_version(chunk) {
                    used[index] = true;
                    shown.push((self.slots[index].0, chunk));
                }
                continue;
            };

            let texture = match self.free_slot(&used, visible, generation) {
                Some(index) => {
                    gfx.update_texture_rgba(self.slots[index].0, pixels);
                    self.slots[index].1 = current;
                    used[index] = true;
                    self.slots[index].0
                }
                None => {
                    let side = self.chunk_side as u32;
                    let id = gfx.create_texture_rgba(side, side, pixels, Some("chunk"));
                    self.slots.push((id, current));
                    used.push(true);
                    id
                }
            };
            shown.push((texture, chunk));
        }
        shown
    }

    /// A slot showing `chunk` from any generation.
    fn previous_version(&self, chunk: ChunkKey) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| slot.1.is_some_and(|(_, shown)| shown == chunk))
    }

    /// A slot not drawn this frame that is empty, shows something off-screen, or shows an
    /// outdated generation.
    fn free_slot(&self, used: &[bool], visible: &[ChunkKey], generation: u64) -> Option<usize> {
        self.slots.iter().enumerate().position(|(index, slot)| {
            !used[index]
                && slot.1.is_none_or(|(slot_generation, shown)| {
                    !visible.contains(&shown) || slot_generation != generation
                })
        })
    }
}
