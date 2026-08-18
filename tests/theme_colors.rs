use hyprbar::theme::contrast_text_for;
use vello::peniko::Color;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba8(r, g, b, 0xff)
}

#[test]
fn light_backgrounds_get_dark_text() {
    let white = rgb(0xff, 0xff, 0xff);
    let pastel = rgb(0xe8, 0xe4, 0xff);
    // Amarillo puro: 0.299 + 0.587 = 0.886, bien arriba del umbral.
    let yellow = rgb(0xff, 0xff, 0x00);

    assert_eq!(contrast_text_for(white), contrast_text_for(pastel));
    assert_eq!(contrast_text_for(white), contrast_text_for(yellow));
}

#[test]
fn dark_backgrounds_get_light_text() {
    let black = rgb(0x00, 0x00, 0x00);
    let navy = rgb(0x10, 0x18, 0x40);
    // Verde puro queda en 0.587, justo debajo del umbral de 0.6.
    let green = rgb(0x00, 0xff, 0x00);
    // Rojo puro: 0.299.
    let red = rgb(0xff, 0x00, 0x00);

    assert_eq!(contrast_text_for(black), contrast_text_for(navy));
    assert_eq!(contrast_text_for(black), contrast_text_for(green));
    assert_eq!(contrast_text_for(black), contrast_text_for(red));
}

#[test]
fn light_and_dark_backgrounds_get_different_text() {
    let white = rgb(0xff, 0xff, 0xff);
    let black = rgb(0x00, 0x00, 0x00);

    assert_ne!(contrast_text_for(white), contrast_text_for(black));
}

#[test]
fn alpha_does_not_change_the_decision() {
    let white = rgb(0xff, 0xff, 0xff);
    let translucent_white = Color::from_rgba8(0xff, 0xff, 0xff, 0x20);

    assert_eq!(contrast_text_for(white), contrast_text_for(translucent_white));
}
