// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use anyhow::{Result, anyhow};
use serde::Deserialize;

// ─── < Constants > ────────────────────────────────────────────────────

const LOCATION_API_URL: &str = "https://ipapi.co/json/";

/// Techo duro por request, para que el worker siempre pueda apagarse.
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedLocation {
    pub coordinates: Coordinates,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IpLocationResponse {
    latitude: Option<f64>,
    longitude: Option<f64>,
    city: Option<String>,
    country_name: Option<String>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn detect_location() -> Result<DetectedLocation> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(HTTP_TIMEOUT)).build().into();

    let mut response = agent.get(LOCATION_API_URL).call()?;
    let body = response.body_mut().read_to_string()?;

    parse_detected_location(&body)
}

pub fn parse_detected_location(body: &str) -> Result<DetectedLocation> {
    let response: IpLocationResponse = serde_json::from_str(body)?;

    let latitude = response
        .latitude
        .ok_or_else(|| anyhow!("la respuesta de ubicación no trae latitude"))?;
    let longitude = response
        .longitude
        .ok_or_else(|| anyhow!("la respuesta de ubicación no trae longitude"))?;

    Ok(DetectedLocation {
        coordinates: Coordinates::new(latitude, longitude),
        label: location_label(response.city, response.country_name),
    })
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn location_label(city: Option<String>, country_name: Option<String>) -> Option<String> {
    match (city, country_name) {
        (Some(city), Some(country)) => Some(format!("{city}, {country}")),
        (Some(city), None) => Some(city),
        (None, Some(country)) => Some(country),
        (None, None) => None,
    }
}
