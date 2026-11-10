//! Where colonies sit when there is more than one, and how far each has grown.
//!
//! Both layouts give every colony a random *start delay*: at overall growth `g` a colony
//! that starts at `delay` has grown `(g - delay) / (1 - delay)`, so late ones catch up by the
//! end. Colonies that have not started yet are left out.

//!
//! - `placement/patch.rs`: a patch, spread evenly over a square.
//! - `placement/field.rs`: the endless field, one colony per grid cell.

mod field;
mod patch;

pub(super) use field::field_colonies;
pub(super) use patch::place_in_patch;

/// A colony's own growth at overall growth `overall`, given its start delay.
fn delayed_growth(overall: f32, delay: f32) -> f32 {
    ((overall - delay) / (1.0 - delay)).clamp(0.0, 1.0)
}
