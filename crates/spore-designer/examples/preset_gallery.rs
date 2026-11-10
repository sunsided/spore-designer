//! Render every preset as one fully grown colony into `docs/presets/<name>.png` (transparent
//! background), for the README. Run with `task docs:presets`.

use std::path::Path;

use spore_core::{presets, render};

/// Pixels per cell; the image is a little larger where the colony grows out of its cell.
const SIZE: usize = 256;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/presets");
    std::fs::create_dir_all(&folder)?;

    for (name, make) in presets::ALL {
        let image = render(&make(), 1, 1.0, SIZE);
        let path = folder.join(format!("{}.png", name.to_lowercase()));
        image::save_buffer(
            &path,
            &image.to_rgba8(),
            image.side as u32,
            image.side as u32,
            image::ColorType::Rgba8,
        )?;
        println!("{}", path.display());
    }
    Ok(())
}
