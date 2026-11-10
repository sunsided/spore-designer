use super::*;

#[test]
fn hue_shift_roundtrips() {
    let color = [200.0, 80.0, 40.0];
    let back = shift_hue(shift_hue(color, 0.3, 1.0), -0.3, 1.0);
    for i in 0..3 {
        assert!((color[i] - back[i]).abs() < 0.5, "{color:?} vs {back:?}");
    }

    let grey = shift_hue([128.0; 3], 0.5, 2.0);
    assert!(grey.iter().all(|v| (v - 128.0).abs() < 0.5));
}
