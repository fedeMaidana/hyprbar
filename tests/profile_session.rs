use hyprbar::bar::profile::{greeting, parse_hostname};

#[test]
fn trims_hostname_whitespace() {
    assert_eq!(parse_hostname("maidana\n"), "maidana");
    assert_eq!(parse_hostname("  host  "), "host");
}

#[test]
fn keeps_clean_hostname() {
    assert_eq!(parse_hostname("maidana"), "maidana");
}

#[test]
fn greets_by_time_of_day() {
    assert_eq!(greeting(8), "Buenos días");
    assert_eq!(greeting(15), "Buenas tardes");
    assert_eq!(greeting(22), "Buenas noches");
    assert_eq!(greeting(3), "Buenas noches");
}
