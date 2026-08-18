use hyprbar::theme::hyprcolor::parse_hex_color;
use vello::peniko::Color;

#[test]
fn parses_valid_hex_colors() {
    assert_eq!(parse_hex_color("#ffffff"), Some(Color::from_rgba8(0xff, 0xff, 0xff, 0xff)));
    assert_eq!(parse_hex_color("#000000"), Some(Color::from_rgba8(0x00, 0x00, 0x00, 0xff)));
    assert_eq!(parse_hex_color("#89b4fa"), Some(Color::from_rgba8(0x89, 0xb4, 0xfa, 0xff)));
    assert_eq!(parse_hex_color("#AbCdEf"), Some(Color::from_rgba8(0xab, 0xcd, 0xef, 0xff)));
}

#[test]
fn rejects_missing_prefix_and_wrong_lengths() {
    assert_eq!(parse_hex_color("89b4fa"), None);
    assert_eq!(parse_hex_color("#fff"), None);
    assert_eq!(parse_hex_color("#ffffff00"), None);
    assert_eq!(parse_hex_color("#"), None);
    assert_eq!(parse_hex_color(""), None);
}

#[test]
fn rejects_non_hex_ascii() {
    assert_eq!(parse_hex_color("#gggggg"), None);
    assert_eq!(parse_hex_color("#12345z"), None);
    assert_eq!(parse_hex_color("# 12345"), None);
}

/// Regresión: `len()` mide bytes; un char multibyte que cae justo en 6
/// bytes rompía el slice por no caer en un límite de char.
#[test]
fn rejects_multibyte_input_without_panicking() {
    // "€" ocupa 3 bytes: "#a€aa" tiene exactamente 6 bytes tras el '#'.
    assert_eq!(parse_hex_color("#a€aa"), None);
    // "ñ" ocupa 2 bytes: 6 bytes justos de nuevo.
    assert_eq!(parse_hex_color("#ññña"), None);
    assert_eq!(parse_hex_color("#ααα"), None);
}
