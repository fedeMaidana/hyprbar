// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::app::WorkerHandle;
use crate::components::{Component, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::config::WeatherConfig;
use super::fetcher::spawn_fetcher;
use super::icons::{UNKNOWN_WEATHER_ICON, weather_icon};
use super::panel::WeatherPanel;
use super::state::{WeatherSnapshot, WeatherStore};

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WeatherPill {
    store: WeatherStore,
    _fetcher: Option<WorkerHandle>,
    frame_parts: Option<WeatherParts>,
}

struct WeatherParts {
    icon: &'static str,
    text: String,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherPill {
    pub fn new(config: WeatherConfig) -> Self {
        let store = WeatherStore::new();
        let fetcher = spawn_fetcher(config, store.clone());

        Self {
            store,
            _fetcher: fetcher,
            frame_parts: None,
        }
    }

    fn current_parts(&self) -> WeatherParts {
        match self.store.snapshot() {
            Some(snapshot) => weather_parts(&snapshot),
            None => WeatherParts {
                icon: UNKNOWN_WEATHER_ICON,
                text: "—".to_string(),
            },
        }
    }

    fn take_frame_parts(&mut self) -> WeatherParts {
        self.frame_parts.take().unwrap_or_else(|| self.current_parts())
    }
}

impl Component for WeatherPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let parts = self.current_parts();

        let icon_size = icon_size(ctx);
        let text_size = text_size(ctx);

        let (icon_width, _) = ctx.text.measure(parts.icon, icon_size, &ctx.theme.typography.icon_font_family);

        let (text_width, _) = ctx.text.measure(&parts.text, text_size, &ctx.theme.typography.font_family);

        let width = icon_width + ctx.theme.tokens.weather_inner_gap + text_width + ctx.theme.tokens.pill_padding_x * 2.0;

        self.frame_parts = Some(parts);

        (width, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let parts = self.take_frame_parts();

        draw_weather_icon(scene, bounds, ctx, parts.icon);
        draw_weather_text(scene, bounds, ctx, parts.icon, &parts.text);
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(DropdownId::WEATHER))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::WEATHER)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let data = self.store.data();

        WeatherPanel::draw(scene, surface, anchor, &data, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(WeatherPanel::bounds(surface, anchor, theme))
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn weather_parts(snapshot: &WeatherSnapshot) -> WeatherParts {
    WeatherParts {
        icon: weather_icon(snapshot.weather_code),
        text: format!("{}°", snapshot.temp_c.round() as i32),
    }
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
        TextStyle::new(size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.text_primary),
    );
}

fn draw_weather_text(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>, icon: &str, text: &str) {
    let pad_x = ctx.theme.tokens.pill_padding_x;
    let icon_size = icon_size(ctx);
    let size = text_size(ctx);

    let (icon_width, _) = ctx.text.measure(icon, icon_size, &ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        text,
        bounds.x + pad_x + icon_width + ctx.theme.tokens.weather_inner_gap,
        bounds.y,
        bounds.height,
        TextStyle::new(size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
}

fn icon_size(ctx: &RenderCtx<'_>) -> f32 {
    ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale
}

fn text_size(ctx: &RenderCtx<'_>) -> f32 {
    ctx.theme.typography.size_base
}
