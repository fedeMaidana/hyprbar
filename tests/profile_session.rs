use hyprbar::bar::profile::parse_hostname;

#[test]
fn trims_hostname_whitespace() {
    assert_eq!(parse_hostname("maidana\n"), "maidana");
    assert_eq!(parse_hostname("  host  "), "host");
}

#[test]
fn keeps_clean_hostname() {
    assert_eq!(parse_hostname("maidana"), "maidana");
}
