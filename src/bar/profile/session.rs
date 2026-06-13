// ─── < Imports > ────────────────────────────────────────────────────

use std::env;
use std::fs;

// ─── < Constants > ────────────────────────────────────────────────────

const HOSTNAME_PATH: &str = "/etc/hostname";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn user_name() -> String {
    env::var("USER")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "usuario".to_string())
}

pub fn host_name() -> String {
    if let Some(host) = env::var("HOSTNAME")
        .ok()
        .map(|value| parse_hostname(&value))
        .filter(|value| !value.is_empty())
    {
        return host;
    }

    fs::read_to_string(HOSTNAME_PATH)
        .ok()
        .map(|value| parse_hostname(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "host".to_string())
}

pub fn parse_hostname(raw: &str) -> String {
    raw.trim().to_string()
}
