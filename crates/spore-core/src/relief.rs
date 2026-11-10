//! Maps for real-time lighting, made from the layers every painting pass fills in (see
//! [`Rgba`]):
//!
//! - **displacement / height map** ([`height_map`]): surface height as 16-bit grey;
//! - **normal map** ([`normal_map`], [`encode_normals`]): which way the surface faces, in the
//!   common OpenGL convention (tangent space, X right, Y up, Z out of the image);
//! - **specular map** ([`gloss_map`]): how shiny each pixel is;
//! - **albedo** ([`Rgba::albedo_rgba8`]): the colour without baked lighting.
//!
//! Together they let a game (or the designer's cursor light, `relief/light.rs`) light a
//! mould from any direction.

mod light;
mod maps;
mod normals;

pub use light::{PointLight, relight_rows};
pub use maps::{gloss_map, height_map, max_height};
pub use normals::{encode_normals, normal_map};

/// A unit surface normal `[x, y, z]`, OpenGL convention (Y up).
pub type Normal = [f32; 3];

#[cfg(test)]
mod tests;
