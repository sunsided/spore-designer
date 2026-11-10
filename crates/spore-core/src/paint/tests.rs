use super::field::{paint_chunk, paint_field_window};
use crate::Rgba;
use crate::params::Field;

#[test]
fn chunks_are_seamless() {
    let field = Field::default();
    for (name, make) in crate::presets::ALL {
        let design = make();
        let (size, growth) = (24, 0.8);
        let left = paint_chunk(&design, &field, 3, growth, (-1, 0), size, 0);
        let right = paint_chunk(&design, &field, 3, growth, (0, 0), size, 0);
        let side = left.side;

        // One window two chunks wide, upper half: must match both chunks exactly.
        let wide = paint_field_window(
            &design,
            &field,
            3,
            growth,
            size,
            Rgba::window(2 * side, -(side as i32), 0),
        );
        for y in 0..side {
            for x in 0..2 * side {
                let (canvas_x, canvas_y) = (x as i32 - side as i32, y as i32);
                let chunk = if canvas_x < 0 { &left } else { &right };
                let expected = chunk.pixels
                    [(canvas_y - chunk.top) as usize * side + (canvas_x - chunk.left) as usize];
                assert_eq!(
                    wide.pixels[y * 2 * side + x],
                    expected,
                    "{name}: seam at ({canvas_x}, {canvas_y})"
                );
            }
        }
        assert!(right.coverage() > 0.05, "{name}: empty chunk");

        // A chunk painted with a margin and cropped is the chunk itself.
        let padded = paint_chunk(&design, &field, 3, growth, (0, 0), size, 2);
        assert!(
            padded.cropped(2) == right,
            "{name}: margin changes the chunk"
        );
    }
}
