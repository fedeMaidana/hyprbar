use hyprbar::bar::command_center::{AudioState, parse_active_ssid, parse_nmcli_radio, parse_wpctl_volume};

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
