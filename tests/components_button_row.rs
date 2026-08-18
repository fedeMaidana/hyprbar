use hyprbar::components::evenly_spaced_rects;
use hyprbar::render::Rect;

fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.001, "{a} != {b}");
}

#[test]
fn splits_row_into_equal_buttons_with_gaps() {
    let rects: [Rect; 3] = evenly_spaced_rects(10.0, 5.0, 100.0, 30.0, 5.0);

    // (100 - 2*5) / 3 = 30 de ancho cada uno.
    for rect in &rects {
        assert_close(rect.width, 30.0);
        assert_close(rect.height, 30.0);
        assert_close(rect.y, 5.0);
    }

    assert_close(rects[0].x, 10.0);
    assert_close(rects[1].x, 45.0);
    assert_close(rects[2].x, 80.0);

    // El último botón termina exactamente donde termina la fila.
    assert_close(rects[2].x + rects[2].width, 110.0);
}

#[test]
fn single_button_takes_the_whole_row() {
    let [rect]: [Rect; 1] = evenly_spaced_rects(0.0, 0.0, 50.0, 20.0, 8.0);

    assert_close(rect.width, 50.0);
    assert_close(rect.x, 0.0);
}

#[test]
fn two_buttons_split_like_the_profile_panel() {
    let [left, right]: [Rect; 2] = evenly_spaced_rects(16.0, 100.0, 216.0, 34.0, 8.0);

    assert_close(left.width, 104.0);
    assert_close(right.width, 104.0);
    assert_close(right.x, 128.0);
}
