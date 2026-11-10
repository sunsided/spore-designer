//! The light in every scene comes from the top right, like the fridge bulb in Sporagon.

/// Brightness of a dome-shaped surface: 0.72 at the rim, up to 1.12 where the dome faces the
/// light.
///
/// `dx, dy` is the position relative to the dome's centre divided by its edge distance, and
/// `normalized_radius` is the distance from the centre in the same units (0 centre, 1 rim).
pub(super) fn dome(dx: f32, dy: f32, normalized_radius: f32) -> f32 {
    // Height of a hemisphere at this radius; the surface normal tilts outward with dx, dy.
    let height = (1.0 - normalized_radius * normalized_radius)
        .max(0.0)
        .sqrt();
    let (normal_x, normal_y, normal_z) = (dx * 0.6, dy * 0.6, height + 0.2);
    let length = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z)
        .sqrt()
        .max(1e-4);

    // Diffuse lighting: how directly the normal faces the light (0.45, -0.6, 0.66).
    let facing = (normal_x * 0.45 - normal_y * 0.6 + normal_z * 0.66) / length;
    0.72 + 0.4 * facing.max(0.0)
}
