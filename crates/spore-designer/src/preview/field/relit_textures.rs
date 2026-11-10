//! [`RelitTextures`]: textures for relit chunks, reused frame after frame.

use game_toolkit::prelude::*;

/// Textures for relit chunks: refilled every frame (the light and camera move), so they
/// are simply handed out in order; the pool only grows to the most chunks on screen at once.
#[derive(Default)]
pub(super) struct RelitTextures {
    textures: Vec<TextureId>,
    chunk_side: usize,
}

impl RelitTextures {
    /// Upload each picture (`chunk_side` square RGBA8) into its own texture.
    pub fn upload(
        &mut self,
        gfx: &mut Graphics,
        chunk_side: usize,
        pictures: &[Vec<u8>],
    ) -> Vec<TextureId> {
        if chunk_side != self.chunk_side {
            // shortcut: textures of the old chunk size are not freed (no toolkit call);
            // only changing the size leaks, fine for an editor session.
            self.textures.clear();
            self.chunk_side = chunk_side;
        }

        pictures
            .iter()
            .enumerate()
            .map(|(index, pixels)| match self.textures.get(index) {
                Some(&texture) => {
                    gfx.update_texture_rgba(texture, pixels);
                    texture
                }
                None => {
                    let side = chunk_side as u32;
                    let texture = gfx.create_texture_rgba(side, side, pixels, Some("relit chunk"));
                    self.textures.push(texture);
                    texture
                }
            })
            .collect()
    }
}
