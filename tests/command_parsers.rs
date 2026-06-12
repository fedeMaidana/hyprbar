use hyprbar::bar::command_center::{AudioState, MediaState, parse_brightnessctl_machine, parse_playerctl_metadata, parse_wpctl_volume};

#[test]
fn parses_wpctl_volume() {
    assert_eq!(
        parse_wpctl_volume("Volume: 0.65\n").unwrap(),
        AudioState {
            volume: 0.65,
            muted: false
        }
    );
}

#[test]
fn parses_wpctl_muted_volume() {
    assert_eq!(parse_wpctl_volume("Volume: 0.65 [MUTED]\n").unwrap(), AudioState { volume: 0.65, muted: true });
}

#[test]
fn fails_on_unexpected_wpctl_output() {
    assert!(parse_wpctl_volume("garbage").is_err());
}

#[test]
fn parses_brightnessctl_machine_output() {
    let fraction = parse_brightnessctl_machine("intel_backlight,backlight,4437,38%,11725\n").unwrap();

    assert!((fraction - 4437.0 / 11725.0).abs() < 0.0001);
}

#[test]
fn fails_on_short_brightnessctl_output() {
    assert!(parse_brightnessctl_machine("a,b,c").is_err());
}

#[test]
fn fails_on_zero_brightness_max() {
    assert!(parse_brightnessctl_machine("dev,backlight,10,0%,0").is_err());
}

#[test]
fn parses_playing_media_metadata() {
    let media = parse_playerctl_metadata("Playing\tParanoid Android\tRadiohead\n").unwrap();

    assert_eq!(
        media,
        MediaState {
            playing: true,
            title: "Paranoid Android".to_string(),
            artist: "Radiohead".to_string(),
        }
    );
}

#[test]
fn parses_paused_media_without_artist() {
    let media = parse_playerctl_metadata("Paused\tSome Title").unwrap();

    assert!(!media.playing);
    assert_eq!(media.artist, "");
}
