use hyprbar::locale::{month_name, weekday_abbrev, weekday_name};

#[test]
fn maps_month_names() {
    assert_eq!(month_name(1), "enero");
    assert_eq!(month_name(12), "diciembre");
}

#[test]
fn falls_back_for_out_of_range_months() {
    assert_eq!(month_name(0), "?");
    assert_eq!(month_name(13), "?");
}

#[test]
fn maps_weekday_names_and_abbrevs() {
    assert_eq!(weekday_name(0), "lunes");
    assert_eq!(weekday_name(6), "domingo");
    assert_eq!(weekday_abbrev(3), "jue");
    assert_eq!(weekday_abbrev(7), "?");
}
