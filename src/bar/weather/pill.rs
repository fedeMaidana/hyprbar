// ─── < Imports > ────────────────────────────────────────────────────

use calloop::channel::Sender;
use vello::Scene;
use vello::peniko::Color;

use crate::app::WorkerHandle;
use crate::components::{Component, DropdownId, Interaction, Panel, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::config::WeatherConfig;
use super::fetcher::spawn_fetcher;
use super::icons::{UNKNOWN_WEATHER_ICON, weather_icon};
use super::panel::WeatherPanel;
use super::state::{WeatherData, WeatherSnapshot, WeatherStore};

// ─── < Constants > ────────────────────────────────────────────────────

pub(crate) const WEATHER_DROPDOWN: DropdownId = DropdownId::new("weather");

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WeatherPill {
    store: WeatherStore,
    _fetcher: Option<WorkerHandle>,
    /// Copia local del store, refrescada solo cuando la generación
    /// cambió: `measure` corre en cada frame y clonar el pronóstico
    /// entero cada vez era puro desperdicio.
    data: WeatherData,
    seen_generation: u64,
    frame_parts: Option<WeatherParts>,
}

struct WeatherParts {
    icon: &'static str,
    text: String,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherPill {
    pub fn new(config: WeatherConfig, redraw_signal: Sender<()>) -> Self {
        let store = WeatherStore::new();
        let fetcher = spawn_fetcher(config, store.clone(), redraw_signal);

        Self {
            store,
            _fetcher: fetcher,
            data: WeatherData::default(),
            seen_generation: 0,
            frame_parts: None,
        }
    }

    fn sync_data(&mut self) {
        let generation = self.store.generation();

        if generation != self.seen_generation {
            self.data = self.store.data();
            self.seen_generation = generation;
        }
    }

    fn current_parts(&self) -> WeatherParts {
        match &self.data.snapshot {
            Some(snapshot) => weather_parts(snapshot),
            None => WeatherParts {
                icon: UNKNOWN_WEATHER_ICON,
                text: "—".to_string(),
            },
        }
    }

    fn take_frame_parts(&mut self) -> WeatherParts {
        self.frame_parts.take().unwrap_or_else(|| self.current_parts())
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(WEATHER_DROPDOWN)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        let hovered = ctx.hovered_interaction == Some(Interaction::Dropdown(WEATHER_DROPDOWN));

        Pill::background_for(self.is_active(ctx), hovered, ctx.theme)
    }

    fn content_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
    }
}

impl Component for WeatherPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        self.sync_data();

        let parts = self.current_parts();

        let icon_size = icon_size(ctx);
        let text_size = text_size(ctx);

        let (icon_width, _) = ctx.text.measure(parts.icon, icon_size, ctx.theme.typography.icon_font_family);

        let (text_width, _) = ctx.text.measure(&parts.text, text_size, ctx.theme.typography.font_family);

        let width = icon_width + ctx.theme.tokens.weather_inner_gap + text_width + ctx.theme.tokens.pill_padding_x * 2.0;

        self.frame_parts = Some(parts);

        (width, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw_with_background(scene, bounds, ctx.theme, self.background_color(ctx));

        let parts = self.take_frame_parts();
        let color = self.content_color(ctx);

        draw_weather_icon(scene, bounds, ctx, parts.icon, color);
        draw_weather_text(scene, bounds, ctx, parts.icon, &parts.text, color);
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(WEATHER_DROPDOWN))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(WEATHER_DROPDOWN)
    }

    fn dropdown_max_height(&self, theme: &Theme) -> f32 {
        WeatherPanel::height(theme)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        WeatherPanel { data: &self.data }.render(scene, surface, anchor, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(WeatherPanel { data: &self.data }.bounds(surface, anchor, theme))
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn weather_parts(snapshot: &WeatherSnapshot) -> WeatherParts {
    WeatherParts {
        icon: weather_icon(snapshot.weather_code),
        text: format!("{}°", snapshot.temp_c.round() as i32),
    }
}

fn draw_weather_icon(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>, icon: &str, color: Color) {
    let pad_x = ctx.theme.tokens.pill_padding_x;
    let size = icon_size(ctx);

    ctx.text.draw_centered_v(
        scene,
        icon,
        bounds.x + pad_x,
        bounds.y,
        bounds.height,
        TextStyle::new(size, ctx.theme.typography.icon_font_family, color),
    );
}

fn draw_weather_text(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>, icon: &str, text: &str, color: Color) {
    let pad_x = ctx.theme.tokens.pill_padding_x;
    let icon_size = icon_size(ctx);
    let size = text_size(ctx);

    let (icon_width, _) = ctx.text.measure(icon, icon_size, ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        text,
        bounds.x + pad_x + icon_width + ctx.theme.tokens.weather_inner_gap,
        bounds.y,
        bounds.height,
        TextStyle::new(size, ctx.theme.typography.font_family, color),
    );
}

fn icon_size(ctx: &RenderCtx<'_>) -> f32 {
    ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale
}

fn text_size(ctx: &RenderCtx<'_>) -> f32 {
    ctx.theme.typography.size_base
}
