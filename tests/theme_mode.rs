use hyprbar::theme::ThemeMode;

#[test]
fn parses_persisted_values_case_insensitively() {
    assert_eq!(ThemeMode::from_persisted("dark"), Some(ThemeMode::Dark));
    assert_eq!(ThemeMode::from_persisted("light"), Some(ThemeMode::Light));
    assert_eq!(ThemeMode::from_persisted("Dark"), Some(ThemeMode::Dark));
    assert_eq!(ThemeMode::from_persisted("LIGHT"), Some(ThemeMode::Light));
    assert_eq!(ThemeMode::from_persisted("  light\n"), Some(ThemeMode::Light));
}

#[test]
fn rejects_unknown_values() {
    assert_eq!(ThemeMode::from_persisted(""), None);
    assert_eq!(ThemeMode::from_persisted("banana"), None);
    assert_eq!(ThemeMode::from_persisted("dark light"), None);
}

#[test]
fn as_str_round_trips_through_from_persisted() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        assert_eq!(ThemeMode::from_persisted(mode.as_str()), Some(mode));
    }
}

#[test]
fn toggled_alternates_between_the_two_modes() {
    assert_eq!(ThemeMode::Dark.toggled(), ThemeMode::Light);
    assert_eq!(ThemeMode::Light.toggled(), ThemeMode::Dark);
    assert_eq!(ThemeMode::Dark.toggled().toggled(), ThemeMode::Dark);
}
