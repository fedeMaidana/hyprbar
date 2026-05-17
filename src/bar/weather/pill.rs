use vello::Scene;

use crate::components::{Component, Pill, RenderCtx};
use crate::render::{Rect, TextStyle};

use super::config::WeatherConfig;
use super::fetcher::spawn_fetcher;
use super::icons::{UNKNOWN_WEATHER_ICON, weather_icon};
use super::state::{WeatherSnapshot, WeatherStore};

const INNER_GAP: f32 = 4.0;
const ICON_SCALE: f32 = 1.2;

pub struct WeatherPill {
    store: WeatherStore,
}

impl WeatherPill {
    pub fn new(config: WeatherConfig) -> Self {
        let store = WeatherStore::new();

        spawn_fetcher(config, store.clone());

        Self { store }
    }

    fn current_parts(&self) -> (&'static str, String) {
        match self.store.snapshot() {
            Some(snapshot) => weather_parts(snapshot),
            None => (UNKNOWN_WEATHER_ICON, "—".to_string()),
        }
    }
}

impl Component for WeatherPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let (icon, text) = self.current_parts();

        let icon_size = icon_size(ctx);
        let text_size = text_size(ctx);

        let (icon_width, _) =
            ctx.text
                .measure(icon, icon_size, &ctx.theme.typography.icon_font_family);

        let (text_width, _) = ctx
            .text
            .measure(&text, text_size, &ctx.theme.typography.font_family);

        let width = icon_width + INNER_GAP + text_width + ctx.theme.tokens.pill_padding_x * 2.0;

        (width, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let (icon, text) = self.current_parts();

        draw_weather_icon(scene, bounds, ctx, icon);
        draw_weather_text(scene, bounds, ctx, icon, &text);
    }
}

fn weather_parts(snapshot: WeatherSnapshot) -> (&'static str, String) {
    let icon = weather_icon(snapshot.weather_code);
    let text = format!("{}°", snapshot.temp_c.round() as i32);

    (icon, text)
}

fn draw_weather_icon(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>, icon: &str) {
    let pad_x = ctx.theme.tokens.pill_padding_x;
    let size = icon_size(ctx);

    ctx.text.draw_centered_v(
        scene,
        icon,
        bounds.x + pad_x,
        bounds.y,
        bounds.height,
        TextStyle::new(
            size,
            &ctx.theme.typography.icon_font_family,
            ctx.theme.palette.text_primary,
        ),
    );
}

fn draw_weather_text(
    scene: &mut Scene,
    bounds: Rect,
    ctx: &mut RenderCtx<'_>,
    icon: &str,
    text: &str,
) {
    let pad_x = ctx.theme.tokens.pill_padding_x;
    let icon_size = icon_size(ctx);
    let size = text_size(ctx);

    let (icon_width, _) = ctx
        .text
        .measure(icon, icon_size, &ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        text,
        bounds.x + pad_x + icon_width + INNER_GAP,
        bounds.y,
        bounds.height,
        TextStyle::new(
            size,
            &ctx.theme.typography.font_family,
            ctx.theme.palette.text_primary,
        ),
    );
}

fn icon_size(ctx: &RenderCtx<'_>) -> f32 {
    ctx.theme.typography.size_base * ICON_SCALE
}

fn text_size(ctx: &RenderCtx<'_>) -> f32 {
    ctx.theme.typography.size_base
}
