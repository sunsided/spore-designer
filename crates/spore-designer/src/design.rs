//! [`Design`]: what a saved `.ron` file holds, and how to paint all of its growth stages.

use serde::{Deserialize, Serialize};
use spore_core::{Field, MoldParams, Patch, PatchLayout, Rgba, presets, render, stage_growth};

/// What the preview shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    /// One colony.
    #[default]
    Colony,
    /// A few colonies growing together on one square.
    Patch,
    /// An endless field of colonies to pan across.
    Field,
}

/// A mould plus how it is rendered. This is the save-file format: field names must stay.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Design {
    pub seed: u32,
    /// Pixels per cell (and per patch area in the patch view).
    pub size: usize,
    /// Growth stages from first sprout to full growth.
    pub stages: usize,
    pub mold: MoldParams,
    #[serde(default)]
    pub view: View,
    #[serde(default)]
    pub patch: Patch,
    #[serde(default)]
    pub field: Field,
    /// Normal-map strength: 1 = true slopes, more exaggerates the relief.
    #[serde(default = "unit_relief")]
    pub relief: f32,
}

fn unit_relief() -> f32 {
    1.0
}

impl Default for Design {
    fn default() -> Self {
        Self {
            seed: 1,
            size: 128,
            stages: 10,
            mold: presets::penicillium(),
            view: View::Colony,
            patch: Patch::default(),
            field: Field::default(),
            relief: 1.0,
        }
    }
}

impl Design {
    /// Every growth stage, painted in parallel (one thread per stage): a patch in the patch
    /// view, else a lone colony. The field view paints its own chunks instead (see
    /// `preview/field.rs`).
    pub fn frames(&self) -> Vec<Rgba> {
        // A patch's layout is the same for every stage; work it out once.
        let layout = (self.view == View::Patch)
            .then(|| PatchLayout::new(&self.mold, &self.patch, self.seed, self.size));
        let layout = layout.as_ref();

        std::thread::scope(|scope| {
            let jobs: Vec<_> = (0..self.stages)
                .map(|stage| {
                    scope.spawn(move || {
                        let growth = stage_growth(stage, self.stages);
                        match layout {
                            Some(layout) => layout.render(&self.mold, growth),
                            None => render(&self.mold, self.seed, growth, self.size),
                        }
                    })
                })
                .collect();
            jobs.into_iter()
                .map(|job| job.join().expect("painter panicked"))
                .collect()
        })
    }
}
