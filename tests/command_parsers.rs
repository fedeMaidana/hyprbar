use hyprbar::bar::command_center::{
    AudioState, MediaState, parse_active_ssid, parse_bluetoothctl_powered, parse_brightnessctl_machine, parse_nmcli_radio,
    parse_playerctl_metadata, parse_wpctl_volume,
};

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

#[test]
fn parses_nmcli_radio_states() {
    assert!(parse_nmcli_radio("enabled\n").unwrap());
    assert!(!parse_nmcli_radio("disabled\n").unwrap());
    assert!(parse_nmcli_radio("whatever").is_err());
}

#[test]
fn parses_active_ssid() {
    let output = "no:OtherNetwork\nyes:MiRed5G\nno:Vecino\n";

    assert_eq!(parse_active_ssid(output), Some("MiRed5G".to_string()));
}

#[test]
fn returns_none_when_no_active_ssid() {
    assert_eq!(parse_active_ssid("no:OtherNetwork\n"), None);
    assert_eq!(parse_active_ssid(""), None);
}

#[test]
fn parses_bluetoothctl_powered_states() {
    let powered = "Controller AA:BB:CC:DD:EE:FF (public)\n\tName: pc\n\tPowered: yes\n";
    let off = "Controller AA:BB:CC:DD:EE:FF (public)\n\tPowered: no\n";

    assert_eq!(parse_bluetoothctl_powered(powered), Some(true));
    assert_eq!(parse_bluetoothctl_powered(off), Some(false));
}

#[test]
fn detects_missing_bluetooth_controller() {
    assert_eq!(parse_bluetoothctl_powered("No default controller available\n"), None);
}
