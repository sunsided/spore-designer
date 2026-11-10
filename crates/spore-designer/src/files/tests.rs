use image::ColorType;
use spore_core::{Field, Patch, presets, sheet};

use super::export::encode_with_maps;
use crate::design::{Design, View};

#[test]
fn design_file_and_png_roundtrip() {
    let design = Design {
        seed: 42,
        size: 32,
        stages: 3,
        mold: presets::annihilation(),
        view: View::Patch,
        patch: Patch::default(),
        field: Field::default(),
        relief: 1.5,
    };
    let text = ron::ser::to_string_pretty(&design, ron::ser::PrettyConfig::default()).unwrap();
    assert!(ron::from_str::<Design>(&text).unwrap() == design);

    let frames = design.frames();
    let side = frames[0].side;
    assert!(side >= 32, "frames are at least one cell");

    let (files, _) = encode_with_maps(&frames, design.relief).unwrap();
    let decode = |suffix: &str| {
        let (_, png) = files.iter().find(|(name, _)| *name == suffix).unwrap();
        image::load_from_memory(png).unwrap()
    };

    let lit = decode("").into_rgba8();
    assert_eq!(lit.dimensions(), (side as u32 * 3, side as u32));
    assert_eq!(lit.as_raw(), &sheet(&frames, 3));
    for (suffix, color) in [
        ("-albedo", ColorType::Rgba8),
        ("-height", ColorType::L16),
        ("-normal", ColorType::Rgb8),
        ("-specular", ColorType::L8),
    ] {
        let map = decode(suffix);
        assert_eq!(
            (map.width(), map.height(), map.color()),
            (lit.width(), lit.height(), color)
        );
    }

    // 16-bit heights survive the trip: the tallest point is white.
    let heights = decode("-height").into_luma16();
    assert_eq!(
        heights.pixels().map(|pixel| pixel.0[0]).max(),
        Some(u16::MAX)
    );
}
