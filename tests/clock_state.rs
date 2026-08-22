use std::time::Duration;

use chrono::Weekday;
use hyprbar::bar::clock::{Repeat, TimerData, format_stopwatch, format_timer};

#[test]
fn repeat_matches_the_right_days() {
    assert!(Repeat::Daily.matches(Weekday::Mon));
    assert!(Repeat::Daily.matches(Weekday::Sun));

    assert!(Repeat::Weekdays.matches(Weekday::Fri));
    assert!(!Repeat::Weekdays.matches(Weekday::Sat));

    assert!(Repeat::Weekend.matches(Weekday::Sun));
    assert!(!Repeat::Weekend.matches(Weekday::Wed));
}

#[test]
fn repeat_codes_round_trip() {
    for repeat in Repeat::ALL {
        assert_eq!(Repeat::from_code(repeat.code()), Some(repeat));
    }

    assert_eq!(Repeat::from_code(9), None);
}

#[test]
fn formats_stopwatch_times() {
    assert_eq!(format_stopwatch(Duration::ZERO), ("00:00".to_string(), ",00".to_string()));
    assert_eq!(format_stopwatch(Duration::from_millis(83_450)), ("01:23".to_string(), ",45".to_string()));
    assert_eq!(format_stopwatch(Duration::from_secs(3_671)), ("1:01:11".to_string(), ",00".to_string()));
}

#[test]
fn formats_timer_remaining() {
    assert_eq!(format_timer(Duration::from_secs(300)), "05:00");
    assert_eq!(format_timer(Duration::from_secs(61)), "01:01");
    assert_eq!(format_timer(Duration::ZERO), "00:00");
}

#[test]
fn timer_presets_reset_the_countdown() {
    let mut timer = TimerData::default();

    timer.set_minutes(10);

    assert_eq!(timer.duration, Duration::from_secs(600));
    assert_eq!(timer.remaining(), Duration::from_secs(600));
    assert!(!timer.running());
    assert!((timer.fraction_remaining() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn finished_timer_restarts_full_on_toggle() {
    let mut timer = TimerData {
        remaining_at_pause: Duration::ZERO,
        finished: true,
        ..TimerData::default()
    };

    timer.toggle();

    assert!(timer.running());
    assert!(!timer.finished);
    assert!(timer.remaining() > Duration::from_secs(290));
}
