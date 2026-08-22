// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, Interaction, Panel, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::{WeatherAction, WeatherTab};
use super::icons::{weather_description, weather_icon, weather_icon_color};
use super::state::WeatherData;
use super::{view_air, view_forecast, view_sea};

// ─── < Constants > ────────────────────────────────────────────────────

const PAD: f32 = 16.0;

const HEADER_H: f32 = 84.0;
const HEADER_TITLE_H: f32 = 26.0;
const HEADER_DESC_H: f32 = 18.0;
const HEADER_UPDATED_H: f32 = 16.0;
const HEADER_UPDATED_GAP: f32 = 6.0;
const UPDATED_DOT_RADIUS: f32 = 2.5;
const HEADER_ICON_SCALE: f32 = 2.2;
const HEADER_TEMP_SCALE: f32 = 2.6;
const HEADER_ICON_GAP: f32 = 10.0;

const STATS_H: f32 = 52.0;
const STATS_LABEL_H: f32 = 18.0;
const STAT_ICON_GAP: f32 = 6.0;

const HUMIDITY_GLYPH: &str = "\u{f058e}";
const WIND_GLYPH: &str = "\u{f059d}";
const RAIN_GLYPH: &str = "\u{f0597}";

const TAB_H: f32 = 34.0;
const TAB_INSET: f32 = 3.0;
const TAB_SEGMENT_GAP: f32 = 6.0;
const TAB_BAR_RADIUS: f64 = 12.0;
const TAB_RADIUS: f64 = 10.0;
const TAB_TEXT_SCALE: f32 = 0.82;

const SECTION_GAP: f32 = 14.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WeatherPanel<'a> {
    pub data: &'a WeatherData,
    pub active_tab: WeatherTab,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherPanel<'_> {
    pub fn max_height(theme: &Theme) -> f32 {
        let tallest = view_forecast::height(theme)
            .max(view_air::height(theme))
            .max(view_sea::max_height(theme));

        shell_overhead() + tallest
    }

    fn height(&self, theme: &Theme) -> f32 {
        shell_overhead() + self.view_height(theme)
    }

    fn view_height(&self, theme: &Theme) -> f32 {
        match self.active_tab {
            WeatherTab::Forecast => view_forecast::height(theme),
            WeatherTab::AirUv => view_air::height(theme),
            WeatherTab::Sea => view_sea::height(self.data.sea.as_ref(), theme),
        }
    }

    fn view_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        Rect::new(
            bounds.x + PAD,
            bounds.y + PAD + HEADER_H + STATS_H + SECTION_GAP + TAB_H + SECTION_GAP,
            bounds.width - PAD * 2.0,
            self.view_height(theme),
        )
    }
}

impl Panel for WeatherPanel<'_> {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.weather_panel_width, self.height(theme))
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let inner_x = bounds.x + PAD;
        let inner_width = bounds.width - PAD * 2.0;
        let mut y = bounds.y + PAD;

        draw_header(scene, inner_x, y, inner_width, self.data, ctx);
        y += HEADER_H;

        draw_stats(scene, inner_x, y, inner_width, self.data, ctx);
        y += STATS_H + SECTION_GAP;

        draw_tab_bar(scene, Rect::new(inner_x, y, inner_width, TAB_H), self.active_tab, ctx);

        let area = self.view_area(bounds, ctx.theme);

        match self.active_tab {
            WeatherTab::Forecast => {
                if let Some(snapshot) = &self.data.snapshot {
                    view_forecast::draw(scene, area, snapshot, ctx);
                }
            }
            WeatherTab::AirUv => {
                let uv = self.data.snapshot.as_ref().and_then(|snapshot| snapshot.uv.as_ref());

                view_air::draw(scene, area, uv, self.data.air.as_ref(), ctx);
            }
            WeatherTab::Sea => view_sea::draw(scene, area, self.data.sea.as_ref(), ctx),
        }
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        let tab_bar = Rect::new(bounds.x + PAD, bounds.y + PAD + HEADER_H + STATS_H + SECTION_GAP, bounds.width - PAD * 2.0, TAB_H);

        tab_segment_rects(tab_bar)
            .into_iter()
            .find(|(_, rect)| rect.contains_point(point.x, point.y))
            .map(|(tab, _)| WeatherAction::SelectTab(tab).interaction())
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn shell_overhead() -> f32 {
    PAD + HEADER_H + STATS_H + SECTION_GAP + TAB_H + SECTION_GAP + PAD
}

fn draw_header(scene: &mut Scene, x: f32, y: f32, width: f32, data: &WeatherData, ctx: &mut RenderCtx<'_>) {
    let title_size = ctx.theme.typography.size_base * 1.35;
    let desc_size = ctx.theme.typography.size_base * 0.85;
    let updated_size = ctx.theme.typography.size_base * 0.75;

    let city = data.location_label.as_deref().unwrap_or("Clima");

    ctx.text.draw_centered_v(
        scene,
        city,
        x,
        y,
        HEADER_TITLE_H,
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let description = data
        .snapshot
        .as_ref()
        .map(|snapshot| weather_description(snapshot.weather_code))
        .unwrap_or("sin datos todavía");

    ctx.text.draw_centered_v(
        scene,
        description,
        x,
        y + HEADER_TITLE_H,
        HEADER_DESC_H,
        TextStyle::new(desc_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    // "Actualizado hace N min" con su puntito verde.
    if let Some(updated_at) = data.updated_at {
        let label = updated_label(updated_at);
        let dot_y = y + HEADER_TITLE_H + HEADER_DESC_H + HEADER_UPDATED_GAP + HEADER_UPDATED_H / 2.0;

        let dot = vello::kurbo::Circle::new(((x + UPDATED_DOT_RADIUS) as f64, dot_y as f64), UPDATED_DOT_RADIUS as f64);

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.positive, None, &dot);

        ctx.text.draw_centered_v(
            scene,
            &label,
            x + UPDATED_DOT_RADIUS * 2.0 + 6.0,
            y + HEADER_TITLE_H + HEADER_DESC_H + HEADER_UPDATED_GAP,
            HEADER_UPDATED_H,
            TextStyle::new(updated_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }

    // Derecha: icono grande + temperatura + sensación.
    let Some(snapshot) = &data.snapshot else {
        return;
    };

    let temp_size = ctx.theme.typography.size_base * HEADER_TEMP_SCALE;
    let icon_size = ctx.theme.typography.size_base * HEADER_ICON_SCALE;

    let temp = format!("{}°", snapshot.temp_c.round() as i32);
    let (temp_width, _) = ctx.text.measure(&temp, temp_size, ctx.theme.typography.font_family);
    let temp_x = x + width - temp_width;

    ctx.text.draw_centered_v(
        scene,
        &temp,
        temp_x,
        y,
        HEADER_TITLE_H + HEADER_DESC_H,
        TextStyle::new(temp_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let icon = weather_icon(snapshot.weather_code);
    let icon_color = weather_icon_color(snapshot.weather_code, &ctx.theme.palette);
    let (icon_width, _) = ctx.text.measure(icon, icon_size, ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        icon,
        temp_x - HEADER_ICON_GAP - icon_width,
        y,
        HEADER_TITLE_H + HEADER_DESC_H,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, icon_color),
    );

    if let Some(feels) = snapshot.feels_like_c {
        let feels_label = format!("Sensación {}°", feels.round() as i32);
        let feels_size = ctx.theme.typography.size_base * 0.8;
        let (feels_width, _) = ctx.text.measure(&feels_label, feels_size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            &feels_label,
            x + width - feels_width,
            y + HEADER_TITLE_H + HEADER_DESC_H + HEADER_UPDATED_GAP,
            HEADER_UPDATED_H,
            TextStyle::new(feels_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}

/// HUMEDAD | VIENTO | PRECIPITACIÓN, con divisores verticales.
fn draw_stats(scene: &mut Scene, x: f32, y: f32, width: f32, data: &WeatherData, ctx: &mut RenderCtx<'_>) {
    let snapshot = data.snapshot.as_ref();

    let humidity = snapshot
        .and_then(|snapshot| snapshot.humidity_percent)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".to_string());

    let wind = snapshot
        .and_then(|snapshot| snapshot.wind_kmh)
        .map(|value| format!("{}", value.round() as i32))
        .unwrap_or_else(|| "—".to_string());

    let rain = snapshot
        .and_then(|snapshot| snapshot.precipitation_percent)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".to_string());

    let stats: [(&str, &str, &str, &str); 3] = [
        (HUMIDITY_GLYPH, "HUMEDAD", &humidity, "%"),
        (WIND_GLYPH, "VIENTO", &wind, "km/h"),
        (RAIN_GLYPH, "PRECIPITACIÓN", &rain, "%"),
    ];

    let column = width / stats.len() as f32;

    DropdownFrame::draw_divider(scene, x, y, width, ctx.theme);

    let label_size = ctx.theme.typography.size_base * 0.68;
    let icon_size = ctx.theme.typography.size_base * 0.9;
    let value_size = ctx.theme.typography.size_base * 1.15;
    let unit_size = ctx.theme.typography.size_base * 0.78;

    for (index, (glyph, label, value, unit)) in stats.iter().enumerate() {
        let column_x = x + index as f32 * column;

        if index > 0 {
            let divider =
                vello::kurbo::Rect::new((column_x - 8.0) as f64, (y + 10.0) as f64, (column_x - 7.0) as f64, (y + STATS_H - 6.0) as f64);

            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_divider, None, &divider);
        }

        ctx.text.draw_centered_v(
            scene,
            glyph,
            column_x,
            y + 8.0,
            STATS_LABEL_H,
            TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.text_secondary),
        );

        ctx.text.draw_centered_v(
            scene,
            label,
            column_x + icon_size + STAT_ICON_GAP,
            y + 8.0,
            STATS_LABEL_H,
            TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        let (value_width, _) = ctx.text.measure(value, value_size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            value,
            column_x,
            y + 8.0 + STATS_LABEL_H,
            STATS_H - STATS_LABEL_H - 8.0,
            TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );

        ctx.text.draw_centered_v(
            scene,
            unit,
            column_x + value_width + 4.0,
            y + 8.0 + STATS_LABEL_H,
            STATS_H - STATS_LABEL_H - 8.0,
            TextStyle::new(unit_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}

fn tab_segment_rects(bar: Rect) -> [(WeatherTab, Rect); 3] {
    let inner = Rect::new(bar.x + TAB_INSET, bar.y + TAB_INSET, bar.width - TAB_INSET * 2.0, bar.height - TAB_INSET * 2.0);
    let count = WeatherTab::ALL.len() as f32;
    let segment = (inner.width - TAB_SEGMENT_GAP * (count - 1.0)) / count;

    std::array::from_fn(|index| {
        let tab = WeatherTab::ALL[index];
        let rect = Rect::new(inner.x + index as f32 * (segment + TAB_SEGMENT_GAP), inner.y, segment, inner.height);

        (tab, rect)
    })
}

fn draw_tab_bar(scene: &mut Scene, bar: Rect, active: WeatherTab, ctx: &mut RenderCtx<'_>) {
    let container = RoundedRect::new(bar.x as f64, bar.y as f64, (bar.x + bar.width) as f64, (bar.y + bar.height) as f64, TAB_BAR_RADIUS);
    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_inset, None, &container);

    let text_size = ctx.theme.typography.size_base * TAB_TEXT_SCALE;

    for (tab, rect) in tab_segment_rects(bar) {
        let is_active = tab == active;
        let hovered = ctx.hovered_interaction == Some(WeatherAction::SelectTab(tab).interaction());

        if is_active || hovered {
            let background = if is_active {
                ctx.theme.palette.pill_hover_bg
            } else {
                ctx.theme.palette.panel_raised
            };

            let segment =
                RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, TAB_RADIUS);

            scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &segment);

            if is_active {
                scene.stroke(&vello::kurbo::Stroke::new(1.0), Affine::IDENTITY, ctx.theme.palette.accent, None, &segment);
            }
        }

        let color = if is_active {
            ctx.theme.palette.text_primary
        } else {
            ctx.theme.palette.text_secondary
        };

        ctx.text
            .draw_centered(scene, tab.label(), rect, TextStyle::new(text_size, ctx.theme.typography.font_family, color));
    }
}

fn updated_label(updated_at: Instant) -> String {
    let minutes = updated_at.elapsed().as_secs() / 60;

    match minutes {
        0 => "Actualizado recién".to_string(),
        1 => "Actualizado hace 1 min".to_string(),
        minutes => format!("Actualizado hace {minutes} min"),
    }
}
