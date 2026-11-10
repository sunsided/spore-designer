//! [`Editor`]: everything the user can change, the design and how it is being viewed.

use spore_core::Rgb;

use crate::design::Design;

/// The editor's state. The panel edits it, the previews read it.
pub struct Editor {
    pub design: Design,
    /// Current growth stage; fractional while playing.
    pub stage: f32,
    pub playing: bool,
    /// Playback speed, stages per second.
    pub fps: f32,
    /// Preview background (matters for judging translucent slime and fringe).
    pub background: Rgb,
    /// State of the "Randomize everything" random-number generator, so each click differs.
    pub dice: u64,
    /// Field view camera: the canvas pixel shown at the centre of the preview.
    pub camera: [f32; 2],
    /// Field view auto-pan: pixels per second, and direction in degrees (0 = right, 90 = up).
    pub pan_speed: f32,
    pub pan_angle: f32,
    /// How the preview is lit or shown (the big stage, or the whole endless field).
    pub light: LightSettings,
    /// Last mouse position over the preview, screen pixels (the cursor light follows it).
    pub cursor: Option<[f32; 2]>,
}

/// What the preview shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightMode {
    /// As painted, with Sporagon's fixed light from the top right.
    Baked,
    /// The albedo relit by a point light following the mouse.
    Cursor,
    /// The maps themselves.
    Albedo,
    Height,
    Normals,
    Gloss,
}

/// The cursor light and the map view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSettings {
    pub mode: LightMode,
    /// Light height above the ground, reference pixels.
    pub height: f32,
    pub ambient: f32,
    pub specular: f32,
    pub shininess: f32,
}

/// File operations the panel asks for; run after the panel is drawn (see `files.rs`).
pub enum Action {
    Save,
    Load,
    ExportStage,
    ExportSheet,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            design: Design::default(),
            stage: 9.0,
            playing: false,
            fps: 8.0,
            background: [62.0, 66.0, 72.0],
            dice: 0x2545_F491_4F6C_DD1D,
            camera: [0.0, 0.0],
            pan_speed: 40.0,
            pan_angle: 20.0,
            light: LightSettings {
                mode: LightMode::Baked,
                height: 30.0,
                ambient: 0.25,
                specular: 1.0,
                shininess: 60.0,
            },
            cursor: None,
        }
    }

    /// Move the playhead on by `seconds` while playing, looping at the end; keep it within
    /// the design's stages either way (the stage count may just have shrunk).
    pub fn advance_playback(&mut self, seconds: f32) {
        let stages = self.design.stages as f32;
        if self.playing {
            self.stage = (self.stage + seconds * self.fps) % stages;
        }
        self.stage = self.stage.min(stages - 1.0);
    }

    /// The whole stage currently shown, valid for a design with `stages` stages.
    pub fn stage_index(&self, stages: usize) -> usize {
        (self.stage as usize).min(stages - 1)
    }
}
