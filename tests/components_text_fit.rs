use hyprbar::components::{ELLIPSIS, truncate_to_width};
use hyprbar::render::TextEngine;

const FAMILY: &str = "sans-serif";
const SIZE: f32 = 14.0;

#[test]
fn returns_text_unchanged_when_it_fits() {
    let mut engine = TextEngine::new();

    let text = "corto";
    let (width, _) = engine.measure(text, SIZE, FAMILY);

    assert_eq!(truncate_to_width(&mut engine, text, SIZE, FAMILY, width + 1.0), text);
}

#[test]
fn truncates_with_ellipsis_when_too_wide() {
    let mut engine = TextEngine::new();

    let text = "un texto bastante largo que no entra";
    let (full_width, _) = engine.measure(text, SIZE, FAMILY);

    let max_width = full_width / 2.0;
    let fitted = truncate_to_width(&mut engine, text, SIZE, FAMILY, max_width);

    assert!(fitted.ends_with(ELLIPSIS));
    assert!(fitted.chars().count() < text.chars().count());
    assert!(engine.measure(&fitted, SIZE, FAMILY).0 <= max_width);
}

#[test]
fn truncation_matches_widest_prefix_that_fits() {
    let mut engine = TextEngine::new();

    let text = "abcdefghijklmnop";
    let (full_width, _) = engine.measure(text, SIZE, FAMILY);
    let max_width = full_width * 0.6;

    let fitted = truncate_to_width(&mut engine, text, SIZE, FAMILY, max_width);
    let kept = fitted.chars().count() - ELLIPSIS.chars().count();

    // One more character than the binary search kept must not fit.
    let wider: String = text.chars().take(kept + 1).chain(ELLIPSIS.chars()).collect();

    assert!(engine.measure(&wider, SIZE, FAMILY).0 > max_width);
}

#[test]
fn handles_multibyte_text_without_panicking() {
    let mut engine = TextEngine::new();

    let text = "año célebre ñandú — 日本語のテキスト";
    let fitted = truncate_to_width(&mut engine, text, SIZE, FAMILY, 40.0);

    assert!(fitted.ends_with(ELLIPSIS));
}

#[test]
fn degenerate_width_yields_bare_ellipsis() {
    let mut engine = TextEngine::new();

    assert_eq!(truncate_to_width(&mut engine, "cualquier cosa", SIZE, FAMILY, 0.0), ELLIPSIS);
}
