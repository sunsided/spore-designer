//! [`RelitStage`]: the big stage shown other than as painted. It is relit by the cursor
//! light, or shown as one of its maps (albedo, height, normals, specular).
//!
//! It is redone only when something it depends on changed (stage, light, relief, frames);
//! the picture itself comes from `preview/lighting.rs`.

use game_toolkit::prelude::*;
use spore_core::Rgba;
use spore_core::relief::{Normal, max_height, normal_map};

use crate::editor::LightSettings;
use crate::preview::lighting::shade;

/// Everything the shown picture depends on (besides the frames themselves).
#[derive(Clone, Copy, PartialEq)]
struct Inputs {
    stage: usize,
    light: LightSettings,
    relief: f32,
    /// Light position over the frame, frame pixels.
    light_position: [f32; 2],
}

#[derive(Default)]
pub(super) struct RelitStage {
    /// The texture and its side length, pixels.
    texture: Option<(TextureId, usize)>,
    /// What the texture shows now.
    shown: Option<Inputs>,
    /// Normal maps per stage, computed when first needed, for `normals_relief`.
    normals: Vec<Option<Vec<Normal>>>,
    normals_relief: f32,
}

impl RelitStage {
    /// The frames were repainted: everything cached is stale.
    pub fn frames_changed(&mut self, stages: usize) {
        self.shown = None;
        self.normals = vec![None; stages];
    }

    /// The texture showing `frames[stage]` the way `light.mode` asks (anything but
    /// [`LightMode::Baked`](crate::editor::LightMode::Baked)), redone only if needed.
    pub fn update(
        &mut self,
        gfx: &mut Graphics,
        frames: &[Rgba],
        stage: usize,
        light: LightSettings,
        relief: f32,
        light_position: [f32; 2],
    ) -> TextureId {
        let inputs = Inputs {
            stage,
            light,
            relief,
            light_position,
        };
        if let (Some((texture, _)), Some(shown)) = (self.texture, self.shown)
            && shown == inputs
        {
            return texture;
        }

        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let frame = &frames[stage];
        let normals = self.normals(frames, stage, relief);
        let pixels = shade(
            frame,
            normals,
            light,
            light_position,
            max_height(frame),
            threads,
        );

        self.shown = Some(inputs);
        self.upload(gfx, frame.side, &pixels)
    }

    /// The normal map of `stage`, computed once per stage and relief strength.
    fn normals(&mut self, frames: &[Rgba], stage: usize, relief: f32) -> &[Normal] {
        if relief != self.normals_relief || self.normals.len() != frames.len() {
            self.normals = vec![None; frames.len()];
            self.normals_relief = relief;
        }
        self.normals[stage].get_or_insert_with(|| normal_map(&frames[stage], relief))
    }

    fn upload(&mut self, gfx: &mut Graphics, side: usize, pixels: &[u8]) -> TextureId {
        match self.texture {
            Some((texture, texture_side)) if texture_side == side => {
                gfx.update_texture_rgba(texture, pixels);
                texture
            }
            // shortcut: the old texture is not freed (the toolkit has no release call); only
            // size changes leak, fine for an editor session.
            _ => {
                let side_u32 = side as u32;
                let texture = gfx.create_texture_rgba(side_u32, side_u32, pixels, Some("relit"));
                self.texture = Some((texture, side));
                texture
            }
        }
    }
}
