// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{DateTime, NaiveDate, Offset, Utc};
use chrono_tz::Tz;

// ─── < Constants > ────────────────────────────────────────────────────

pub const WORLD_ZONES: [(&str, Tz); 5] = [
    ("Madrid", chrono_tz::Europe::Madrid),
    ("Nueva York", chrono_tz::America::New_York),
    ("San Francisco", chrono_tz::America::Los_Angeles),
    ("Tokio", chrono_tz::Asia::Tokyo),
    ("Sídney", chrono_tz::Australia::Sydney),
];

pub const TOMORROW_TAG: &str = "mañ.";
pub const YESTERDAY_TAG: &str = "ayer";

const MINUTES_PER_HOUR: i32 = 60;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn zone_offset_minutes(now_utc: DateTime<Utc>, tz: Tz) -> i32 {
    now_utc.with_timezone(&tz).offset().fix().local_minus_utc() / 60
}

pub fn offset_label(diff_minutes: i32) -> String {
    let sign = if diff_minutes < 0 { "-" } else { "+" };

    let total = diff_minutes.abs();
    let hours = total / MINUTES_PER_HOUR;
    let minutes = total % MINUTES_PER_HOUR;

    if diff_minutes == 0 {
        return "0h".to_string();
    }

    if minutes == 0 {
        format!("{sign}{hours}h")
    } else {
        format!("{sign}{hours}:{minutes:02}")
    }
}

pub fn day_tag(local_date: NaiveDate, remote_date: NaiveDate) -> Option<&'static str> {
    if remote_date > local_date {
        return Some(TOMORROW_TAG);
    }

    if remote_date < local_date {
        return Some(YESTERDAY_TAG);
    }

    None
}

pub fn utc_label(offset_minutes: i32) -> String {
    if offset_minutes == 0 {
        return "UTC".to_string();
    }

    let sign = if offset_minutes < 0 { "-" } else { "+" };

    let total = offset_minutes.abs();
    let hours = total / MINUTES_PER_HOUR;
    let minutes = total % MINUTES_PER_HOUR;

    if minutes == 0 {
        format!("UTC{sign}{hours}")
    } else {
        format!("UTC{sign}{hours}:{minutes:02}")
    }
}

pub fn zone_display_name(iana: &str) -> String {
    iana.rsplit('/').next().unwrap_or(iana).replace('_', " ")
}

pub fn local_zone_display_name() -> String {
    iana_time_zone::get_timezone()
        .map(|iana| zone_display_name(&iana))
        .unwrap_or_else(|_| "hora local".to_string())
}
