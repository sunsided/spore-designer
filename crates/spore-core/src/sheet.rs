//! Sprite sheets: several equally sized frames laid out on one grid image.

use crate::Rgba;

/// Lay equally sized frames out as straight-alpha RGBA8, `columns` per row. Rows fill from the
/// top left; cells after the last frame stay transparent. The image is `columns * side` wide.
pub fn sheet(frames: &[Rgba], columns: usize) -> Vec<u8> {
    let Some(first) = frames.first() else {
        return Vec::new();
    };
    let images: Vec<Vec<u8>> = frames.iter().map(Rgba::to_rgba8).collect();
    grid(&images, first.side, 4, columns)
}

/// Lay equally sized square images out on a grid, `columns` per row, for any pixel format:
/// each image holds `side * side * channels` values, row-major. Cells after the last image
/// are filled with the default value (0).
pub fn grid<T: Copy + Default>(
    images: &[Vec<T>],
    side: usize,
    channels: usize,
    columns: usize,
) -> Vec<T> {
    if images.is_empty() {
        return Vec::new();
    }
    let columns = columns.clamp(1, images.len());
    let rows = images.len().div_ceil(columns);
    let row_values = columns * side * channels;
    let image_row = side * channels;
    let mut out = vec![T::default(); rows * side * row_values];

    for (index, image) in images.iter().enumerate() {
        let (cell_x, cell_y) = (index % columns * side, index / columns * side);
        for y in 0..side {
            let at = (cell_y + y) * row_values + cell_x * channels;
            out[at..at + image_row].copy_from_slice(&image[y * image_row..(y + 1) * image_row]);
        }
    }
    out
}

/// Cut `margin` pixels off every side of a square image of `side * side * channels` values.
pub fn crop<T: Copy>(values: &[T], side: usize, channels: usize, margin: usize) -> Vec<T> {
    let inner = side.saturating_sub(2 * margin);
    let mut out = Vec::with_capacity(inner * inner * channels);
    for y in margin..margin + inner {
        let start = (y * side + margin) * channels;
        out.extend_from_slice(&values[start..start + inner * channels]);
    }
    out
}

#[cfg(test)]
mod tests;
