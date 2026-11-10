//! Where colonies sit when there is more than one: a [`Patch`] on a square, or an endless
//! [`Field`]. Used by `paint/placement.rs`.

use serde::{Deserialize, Serialize};

/// `count` colonies on a square `area` colony widths wide, each starting to grow after a
/// random delay of up to `stagger` (as a fraction of the whole growth).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub count: u32,
    pub area: f32,
    pub stagger: f32,
}

impl Default for Patch {
    fn default() -> Self {
        Self {
            count: 7,
            area: 2.5,
            stagger: 0.5,
        }
    }
}

/// An endless field of colonies: one per cell of a grid `spacing` colony widths wide (with
/// probability `density`), pushed off-centre by up to `jitter` of a cell, each starting to
/// grow after a random delay of up to `stagger`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub spacing: f32,
    pub jitter: f32,
    pub density: f32,
    pub stagger: f32,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            spacing: 1.1,
            jitter: 0.8,
            density: 0.85,
            stagger: 0.6,
        }
    }
}
