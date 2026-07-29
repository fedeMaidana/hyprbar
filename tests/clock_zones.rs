use chrono::{TimeZone, Utc};
use hyprbar::bar::clock::{is_daytime, offset_label, utc_label, zone_display_name, zone_offset_minutes};

#[test]
fn computes_zone_offset_with_dst() {
    let summer = Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap();
    let winter = Utc.with_ymd_and_hms(2026, 1, 11, 12, 0, 0).unwrap();

    assert_eq!(zone_offset_minutes(summer, chrono_tz::Asia::Tokyo), 540);
    assert_eq!(zone_offset_minutes(summer, chrono_tz::Europe::Madrid), 120);
    assert_eq!(zone_offset_minutes(winter, chrono_tz::Europe::Madrid), 60);
}

#[test]
fn formats_whole_hour_offsets() {
    assert_eq!(offset_label(300), "+5h");
    assert_eq!(offset_label(-240), "-4h");
    assert_eq!(offset_label(0), "0h");
}

#[test]
fn formats_fractional_offsets() {
    assert_eq!(offset_label(330), "+5:30");
    assert_eq!(offset_label(-210), "-3:30");
    assert_eq!(offset_label(30), "+0:30");
}

#[test]
fn classifies_day_and_night_hours() {
    assert!(is_daytime(7));
    assert!(is_daytime(12));
    assert!(is_daytime(18));
    assert!(!is_daytime(19));
    assert!(!is_daytime(23));
    assert!(!is_daytime(6));
}

#[test]
fn formats_utc_labels() {
    assert_eq!(utc_label(-180), "UTC-3");
    assert_eq!(utc_label(330), "UTC+5:30");
    assert_eq!(utc_label(0), "UTC");
}

#[test]
fn shortens_iana_zone_names() {
    assert_eq!(zone_display_name("America/Argentina/Buenos_Aires"), "Buenos Aires");
    assert_eq!(zone_display_name("Europe/Madrid"), "Madrid");
    assert_eq!(zone_display_name("UTC"), "UTC");
}
