use image::ColorType;
use spore_core::{Field, Patch, presets, sheet};

use super::export::{export_with_maps, sibling};
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

    let directory = std::env::temp_dir().join(format!("spore-export-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("sheet.png");
    export_with_maps(&path, &frames, design.relief).unwrap();

    let lit = image::open(&path).unwrap().into_rgba8();
    assert_eq!(lit.dimensions(), (side as u32 * 3, side as u32));
    assert_eq!(lit.as_raw(), &sheet(&frames, 3));
    for (suffix, color) in [
        ("albedo", ColorType::Rgba8),
        ("height", ColorType::L16),
        ("normal", ColorType::Rgb8),
        ("specular", ColorType::L8),
    ] {
        let map = image::open(sibling(&path, suffix)).unwrap();
        assert_eq!(
            (map.width(), map.height(), map.color()),
            (lit.width(), lit.height(), color)
        );
    }
    std::fs::remove_dir_all(&directory).ok();
}
