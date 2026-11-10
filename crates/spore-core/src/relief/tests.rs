use super::*;
use crate::{presets, render};

#[test]
fn domes_tilt_outwards_and_flat_ground_faces_up() {
    let image = render(&presets::penicillium(), 1, 1.0, 64);
    let normals = normal_map(&image, 1.0);
    let side = image.side;
    let center = side / 2;

    // Corner: no mould, flat ground.
    assert_eq!(encode_normals(&normals[..1]), vec![128, 128, 255]);

    // Left of the centre the dome faces left (-X), right of it right (+X); above the
    // centre it faces up (+Y in the normal map).
    let at = |x: usize, y: usize| normals[y * side + x];
    assert!(at(center - 10, center)[0] < -0.05);
    assert!(at(center + 10, center)[0] > 0.05);
    assert!(at(center, center - 10)[1] > 0.05);

    assert!(max_height(&image) > 3.0, "a grown colony stands out");
    assert!(gloss_map(&image).iter().any(|&gloss| gloss > 30));
}
