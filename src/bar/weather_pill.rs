//! Pill del clima.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use vello::Scene;

use crate::components::{Component, Pill, RenderCtx};
use crate::render::Rect;

#[derive(Debug, Clone)]
pub struct WeatherConfig {
    pub latitude: f64,
    pub longitude: f64,
    pub fetch_interval: Duration,
}

impl WeatherConfig {
    pub fn mar_del_plata() -> Self {
        Self {
            latitude: -38.0023,
            longitude: -57.5575,
            fetch_interval: Duration::from_secs(600),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct WeatherSnapshot {
    temp_c: f32,
    weather_code: u32,
}

pub struct WeatherPill {
    state: Arc<Mutex<Option<WeatherSnapshot>>>,
}

impl WeatherPill {
    pub fn new(config: WeatherConfig) -> Self {
        let state: Arc<Mutex<Option<WeatherSnapshot>>> = Arc::new(Mutex::new(None));

        let state_clone = Arc::clone(&state);
        thread::spawn(move || fetcher_loop(config, state_clone));

        Self { state }
    }

    fn current_parts(&self) -> (&'static str, String) {
        match *self.state.lock().unwrap() {
            Some(snap) => {
                let icon = weather_icon(snap.weather_code);
                let text = format!("{}°", snap.temp_c.round() as i32);
                (icon, text)
            }
            None => ("\u{e374}", "—".to_string()),
        }
    }
}

impl Component for WeatherPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let (icon, text) = self.current_parts();
        let icon_size = ctx.theme.typography.size_base * 1.2;
        let text_size = ctx.theme.typography.size_base;

        let (iw, _) = ctx
            .text
            .measure(icon, icon_size, &ctx.theme.typography.icon_font_family);
        let (tw, _) = ctx
            .text
            .measure(&text, text_size, &ctx.theme.typography.font_family);

        let inner_gap = 4.0;
        let w = iw + inner_gap + tw + ctx.theme.tokens.pill_padding_x * 2.0;
        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let (icon, text) = self.current_parts();
        let pad_x = ctx.theme.tokens.pill_padding_x;
        let icon_size = ctx.theme.typography.size_base * 1.2;
        let text_size = ctx.theme.typography.size_base;
        let inner_gap = 4.0;

        let (iw, _) = ctx
            .text
            .measure(icon, icon_size, &ctx.theme.typography.icon_font_family);

        ctx.text.draw_centered_v(
            scene,
            icon,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            icon_size,
            &ctx.theme.typography.icon_font_family,
            ctx.theme.palette.text_primary,
        );

        ctx.text.draw_centered_v(
            scene,
            &text,
            bounds.x + pad_x + iw + inner_gap,
            bounds.y,
            bounds.height,
            text_size,
            &ctx.theme.typography.font_family,
            ctx.theme.palette.text_primary,
        );
    }
}

// ============================================================
// Fetcher en background
// ============================================================

fn fetcher_loop(config: WeatherConfig, state: Arc<Mutex<Option<WeatherSnapshot>>>) {
    loop {
        match fetch_once(&config) {
            Ok(snap) => {
                log::info!(
                    "weather: {}°C code={}",
                    snap.temp_c.round() as i32,
                    snap.weather_code
                );
                *state.lock().unwrap() = Some(snap);
            }
            Err(e) => {
                log::warn!("weather fetch failed: {e}");
            }
        }
        thread::sleep(config.fetch_interval);
    }
}

fn fetch_once(config: &WeatherConfig) -> anyhow::Result<WeatherSnapshot> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code",
        config.latitude, config.longitude
    );
    // ureq 3: Response<Body>, leemos via body_mut().read_to_string()
    let mut response = ureq::get(&url).call()?;
    let body = response.body_mut().read_to_string()?;
    let json: serde_json::Value = serde_json::from_str(&body)?;
    let current = &json["current"];
    Ok(WeatherSnapshot {
        temp_c: current["temperature_2m"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("missing temperature_2m"))? as f32,
        weather_code: current["weather_code"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("missing weather_code"))? as u32,
    })
}

fn weather_icon(code: u32) -> &'static str {
    match code {
        0 => "\u{e30d}",
        1 | 2 => "\u{e302}",
        3 => "\u{e312}",
        45 | 48 => "\u{e313}",
        51..=57 => "\u{e319}",
        61..=67 => "\u{e318}",
        71..=77 => "\u{e31a}",
        80..=82 => "\u{e318}",
        85 | 86 => "\u{e31a}",
        95..=99 => "\u{e31d}",
        _ => "\u{e374}",
    }
}