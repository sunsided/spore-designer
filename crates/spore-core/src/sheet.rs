//! Sprite sheets: several equally sized frames laid out on one grid image.

use crate::Rgba;

/// Lay equally sized frames out as straight-alpha RGBA8, `columns` per row. Rows fill from the
/// top left; cells after the last frame stay transparent. The image is `columns * side` wide.
pub fn sheet(frames: &[Rgba], columns: usize) -> Vec<u8> {
    let Some(first) = frames.first() else {
        return Vec::new();
    };
    let (side, columns) = (first.side, columns.clamp(1, frames.len()));
    let rows = frames.len().div_ceil(columns);
    let row_bytes = columns * side * 4;
    let mut out = vec![0; rows * side * row_bytes];

    for (index, frame) in frames.iter().enumerate() {
        let pixels = frame.to_rgba8();
        let (cell_x, cell_y) = (index % columns * side, index / columns * side);
        for y in 0..side {
            let at = (cell_y + y) * row_bytes + cell_x * 4;
            out[at..at + side * 4].copy_from_slice(&pixels[y * side * 4..(y + 1) * side * 4]);
        }
    }
    out
}

// ------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{presets, render};

    #[test]
    fn one_row() {
        let design = presets::penicillium();
        let frames = [render(&design, 1, 0.3, 8), render(&design, 1, 1.0, 8)];
        let side = frames[0].side;
        assert_eq!(frames[1].side, side, "stages of one design share a size");

        let image = sheet(&frames, 2);
        assert_eq!(image.len(), 2 * side * side * 4);

        // Row 3 of the sheet: frame 0's row 3, then frame 1's row 3.
        let (frame_row, sheet_row) = (side * 4, 2 * side * 4);
        assert_eq!(
            &image[3 * sheet_row..3 * sheet_row + frame_row],
            &frames[0].to_rgba8()[3 * frame_row..4 * frame_row]
        );
        assert_eq!(
            &image[3 * sheet_row + frame_row..4 * sheet_row],
            &frames[1].to_rgba8()[3 * frame_row..4 * frame_row]
        );
    }

    #[test]
    fn wraps_into_rows() {
        let design = presets::penicillium();
        let frames: Vec<Rgba> = (0..3)
            .map(|i| render(&design, 1, 0.3 + 0.3 * i as f32, 8))
            .collect();
        let side = frames[0].side;
        let image = sheet(&frames, 2);

        // Two columns, two rows; frame 2 starts the second row, the last cell stays empty.
        assert_eq!(image.len(), 2 * side * 2 * side * 4);
        let row_of = |y: usize, column: usize| {
            &image[(y * 2 * side + column * side) * 4..(y * 2 * side + (column + 1) * side) * 4]
        };
        assert_eq!(row_of(side, 0), &frames[2].to_rgba8()[..side * 4]);
        assert!(row_of(side, 1).iter().all(|&byte| byte == 0));
    }
}
