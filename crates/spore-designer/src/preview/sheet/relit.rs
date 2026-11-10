//! [`RelitStage`]: the big stage shown other than as painted. It is relit by the cursor
//! light, or shown as one of its maps (albedo, height, normals, specular).
//!
//! It is redone only when something it depends on changed (stage, light, relief, frames),
//! and the relighting is split across threads by rows.

use game_toolkit::prelude::*;
use spore_core::Rgba;
use spore_core::relief::{
    Normal, PointLight, encode_normals, gloss_map, height_map, max_height, normal_map, relight_rows,
};

use crate::editor::{LightMode, LightSettings};

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
    /// [`LightMode::Baked`]), redone only if needed.
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

        let frame = &frames[stage];
        let pixels = match light.mode {
            LightMode::Baked | LightMode::Albedo => frame.albedo_rgba8(),
            LightMode::Cursor => {
                let normals = self.normals(frames, stage, relief);
                relight(frame, normals, &point_light(light, light_position))
            }
            LightMode::Height => {
                let max = max_height(frame);
                grey(
                    height_map(frame, max)
                        .iter()
                        .map(|height| (height >> 8) as u8),
                )
            }
            LightMode::Normals => {
                let normals = self.normals(frames, stage, relief);
                encode_normals(normals)
                    .chunks(3)
                    .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                    .collect()
            }
            LightMode::Gloss => grey(gloss_map(frame).into_iter()),
        };

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

// ------------------------------------------------------------------------------------------

fn point_light(light: LightSettings, position: [f32; 2]) -> PointLight {
    PointLight {
        x: position[0],
        y: position[1],
        height: light.height,
        ambient: light.ambient,
        specular: light.specular,
        shininess: light.shininess,
    }
}

/// Relight a whole frame, rows split across threads.
fn relight(frame: &Rgba, normals: &[Normal], light: &PointLight) -> Vec<u8> {
    let side = frame.side;
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_per_thread = side.div_ceil(threads).max(1);
    let mut out = vec![0u8; side * side * 4];

    std::thread::scope(|scope| {
        for (index, rows) in out.chunks_mut(rows_per_thread * side * 4).enumerate() {
            let first = index * rows_per_thread;
            let last = (first + rows_per_thread).min(side);
            scope.spawn(move || relight_rows(frame, normals, light, first..last, rows));
        }
    });
    out
}

/// Opaque grey RGBA8 from grey values.
fn grey(values: impl Iterator<Item = u8>) -> Vec<u8> {
    values
        .flat_map(|value| [value, value, value, 255])
        .collect()
}
