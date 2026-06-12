// ─── < Imports > ────────────────────────────────────────────────────

use chrono::Datelike;
use vello::Scene;

use crate::components::{DropdownFrame, RenderCtx};
use crate::locale::weekday_abbrev;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::icons::{UNKNOWN_WEATHER_ICON, weather_description, weather_icon};
use super::state::{DailyForecast, WeatherData, WeatherSnapshot};

// ─── < Constants > ────────────────────────────────────────────────────

const FORECAST_COLUMNS: usize = 5;
const VALUE_PLACEHOLDER: &str = "—";
const FALLBACK_TITLE: &str = "Clima";

const HUMIDITY_GLYPH: &str = "\u{f058e}";
const WIND_GLYPH: &str = "\u{f059d}";
const PRECIPITATION_GLYPH: &str = "\u{f0576}";

const SUBTITLE_TEXT_SCALE: f32 = 0.78;
const ST_TEXT_SCALE: f32 = 0.7;
const DETAIL_ICON_SCALE: f32 = 0.9;
const DETAIL_TEXT_SCALE: f32 = 0.78;
const FORECAST_DAY_SCALE: f32 = 0.7;
const FORECAST_ICON_SCALE: f32 = 1.3;
const FORECAST_MAX_SCALE: f32 = 0.78;
const FORECAST_MIN_SCALE: f32 = 0.7;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WeatherPanel;

struct DetailItem {
    glyph: &'static str,
    text: String,
}

struct ForecastColumn {
    day: &'static str,
    icon: &'static str,
    max: String,
    min: String,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherPanel {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.weather_panel_padding_y * 2.0
            + tokens.weather_header_height
            + tokens.weather_section_gap
            + tokens.weather_details_row_height
            + tokens.weather_section_gap
            + tokens.weather_forecast_day_height
            + tokens.weather_forecast_icon_height
            + tokens.weather_forecast_max_height
            + tokens.weather_forecast_min_height
    }

    pub fn bounds(surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        Self::frame(theme).bounds(surface, anchor, theme)
    }

    pub fn draw(scene: &mut Scene, surface: Rect, anchor: Rect, data: &WeatherData, ctx: &mut RenderCtx<'_>) {
        let theme = ctx.theme;
        let tokens = theme.tokens;

        let frame = Self::frame(theme);
        let bounds = frame.bounds(surface, anchor, theme);

        frame.draw_background(scene, bounds, theme);

        let inner_x = bounds.x + tokens.weather_panel_padding_x;
        let inner_width = bounds.width - tokens.weather_panel_padding_x * 2.0;
        let mut y = bounds.y + tokens.weather_panel_padding_y;

        draw_header(scene, inner_x, y, inner_width, data, ctx);

        y += tokens.weather_header_height + tokens.weather_section_gap;

        let items = detail_items(data.snapshot.as_ref());

        draw_details_row(scene, inner_x, y, inner_width, &items, ctx);

        y += tokens.weather_details_row_height + tokens.weather_section_gap;

        let daily = data.snapshot.as_ref().map(|snapshot| snapshot.daily.as_slice()).unwrap_or(&[]);

        draw_forecast(scene, inner_x, y, inner_width, daily, ctx);
    }

    fn frame(theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.weather_panel_width, Self::height(theme))
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_header(scene: &mut Scene, x: f32, y: f32, width: f32, data: &WeatherData, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let header_height = tokens.weather_header_height;

    let title_size = ctx.theme.typography.size_base;
    let subtitle_size = title_size * SUBTITLE_TEXT_SCALE;

    let snapshot = data.snapshot.as_ref();

    let title = data.location_label.as_deref().map(city_name).unwrap_or(FALLBACK_TITLE);
    let subtitle = snapshot
        .map(|snapshot| weather_description(snapshot.weather_code))
        .unwrap_or(VALUE_PLACEHOLDER);

    ctx.text.draw_centered_v(
        scene,
        title,
        x,
        y,
        header_height * 0.55,
        TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        subtitle,
        x,
        y + header_height * 0.55,
        header_height * 0.45,
        TextStyle::new(subtitle_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    let temp_size = title_size * tokens.weather_temp_scale;
    let st_size = title_size * ST_TEXT_SCALE;

    let temp_text = snapshot
        .map(|snapshot| format!("{}°", snapshot.temp_c.round() as i32))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    let st_text = snapshot
        .and_then(|snapshot| snapshot.feels_like_c)
        .map(|value| format!("ST {}°", value.round() as i32));

    let temp_box_height = header_height * 0.6;

    let (temp_width, _) = ctx.text.measure(&temp_text, temp_size, &ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &temp_text,
        x + width - temp_width,
        y,
        temp_box_height,
        TextStyle::new(temp_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let mut block_width = temp_width;

    if let Some(st_text) = &st_text {
        let (st_width, _) = ctx.text.measure(st_text, st_size, &ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            st_text,
            x + width - st_width,
            y + temp_box_height,
            header_height - temp_box_height,
            TextStyle::new(st_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        block_width = block_width.max(st_width);
    }

    let icon = snapshot
        .map(|snapshot| weather_icon(snapshot.weather_code))
        .unwrap_or(UNKNOWN_WEATHER_ICON);
    let icon_size = title_size * tokens.weather_header_icon_scale;

    let (icon_width, _) = ctx.text.measure(icon, icon_size, &ctx.theme.typography.icon_font_family);
    let icon_x = x + width - block_width - tokens.weather_inner_gap * 2.0 - icon_width;

    ctx.text.draw_centered_v(
        scene,
        icon,
        icon_x,
        y,
        header_height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.accent),
    );
}

fn detail_items(snapshot: Option<&WeatherSnapshot>) -> [DetailItem; 3] {
    let humidity = snapshot
        .and_then(|snapshot| snapshot.humidity_percent)
        .map(|value| format!("{value}%"));

    let wind = snapshot
        .and_then(|snapshot| snapshot.wind_kmh)
        .map(|value| format!("{} km/h", value.round() as i32));

    let precipitation = snapshot
        .and_then(|snapshot| snapshot.precipitation_percent)
        .map(|value| format!("{value}%"));

    [
        DetailItem {
            glyph: HUMIDITY_GLYPH,
            text: humidity.unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
        },
        DetailItem {
            glyph: WIND_GLYPH,
            text: wind.unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
        },
        DetailItem {
            glyph: PRECIPITATION_GLYPH,
            text: precipitation.unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
        },
    ]
}

fn draw_details_row(scene: &mut Scene, x: f32, y: f32, width: f32, items: &[DetailItem; 3], ctx: &mut RenderCtx<'_>) {
    let center_width = detail_item_width(&items[1], ctx);
    let right_width = detail_item_width(&items[2], ctx);

    let positions = [x, x + (width - center_width) / 2.0, x + width - right_width];

    for (item, item_x) in items.iter().zip(positions) {
        draw_detail_item(scene, item_x, y, item, ctx);
    }
}

fn detail_item_width(item: &DetailItem, ctx: &mut RenderCtx<'_>) -> f32 {
    let icon_size = ctx.theme.typography.size_base * DETAIL_ICON_SCALE;
    let text_size = ctx.theme.typography.size_base * DETAIL_TEXT_SCALE;

    let (icon_width, _) = ctx.text.measure(item.glyph, icon_size, &ctx.theme.typography.icon_font_family);
    let (text_width, _) = ctx.text.measure(&item.text, text_size, &ctx.theme.typography.font_family);

    icon_width + ctx.theme.tokens.weather_inner_gap + text_width
}

fn draw_detail_item(scene: &mut Scene, x: f32, y: f32, item: &DetailItem, ctx: &mut RenderCtx<'_>) {
    let row_height = ctx.theme.tokens.weather_details_row_height;
    let icon_size = ctx.theme.typography.size_base * DETAIL_ICON_SCALE;
    let text_size = ctx.theme.typography.size_base * DETAIL_TEXT_SCALE;

    let (icon_width, _) = ctx.text.measure(item.glyph, icon_size, &ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        item.glyph,
        x,
        y,
        row_height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.text_secondary),
    );

    ctx.text.draw_centered_v(
        scene,
        &item.text,
        x + icon_width + ctx.theme.tokens.weather_inner_gap,
        y,
        row_height,
        TextStyle::new(text_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
}

fn forecast_columns(daily: &[DailyForecast]) -> Vec<ForecastColumn> {
    let mut columns: Vec<ForecastColumn> = daily
        .iter()
        .take(FORECAST_COLUMNS)
        .map(|forecast| ForecastColumn {
            day: weekday_abbrev(forecast.date.weekday().num_days_from_monday() as usize),
            icon: weather_icon(forecast.weather_code),
            max: format!("{}°", forecast.max_c.round() as i32),
            min: format!("{}°", forecast.min_c.round() as i32),
        })
        .collect();

    while columns.len() < FORECAST_COLUMNS {
        columns.push(ForecastColumn {
            day: VALUE_PLACEHOLDER,
            icon: UNKNOWN_WEATHER_ICON,
            max: VALUE_PLACEHOLDER.to_string(),
            min: VALUE_PLACEHOLDER.to_string(),
        });
    }

    columns
}

fn draw_forecast(scene: &mut Scene, x: f32, y: f32, width: f32, daily: &[DailyForecast], ctx: &mut RenderCtx<'_>) {
    let columns = forecast_columns(daily);
    let column_width = width / FORECAST_COLUMNS as f32;

    for (index, column) in columns.iter().enumerate() {
        let column_x = x + index as f32 * column_width;

        draw_forecast_column(scene, column_x, y, column_width, column, ctx);
    }
}

fn draw_forecast_column(scene: &mut Scene, x: f32, y: f32, width: f32, column: &ForecastColumn, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let base = ctx.theme.typography.size_base;

    let mut cell_y = y;

    draw_centered_text(
        scene,
        column.day,
        x,
        cell_y,
        width,
        tokens.weather_forecast_day_height,
        base * FORECAST_DAY_SCALE,
        &ctx.theme.typography.font_family.clone(),
        ctx.theme.palette.text_secondary,
        ctx,
    );

    cell_y += tokens.weather_forecast_day_height;

    draw_centered_text(
        scene,
        column.icon,
        x,
        cell_y,
        width,
        tokens.weather_forecast_icon_height,
        base * FORECAST_ICON_SCALE,
        &ctx.theme.typography.icon_font_family.clone(),
        ctx.theme.palette.text_primary,
        ctx,
    );

    cell_y += tokens.weather_forecast_icon_height;

    draw_centered_text(
        scene,
        &column.max,
        x,
        cell_y,
        width,
        tokens.weather_forecast_max_height,
        base * FORECAST_MAX_SCALE,
        &ctx.theme.typography.font_family.clone(),
        ctx.theme.palette.text_primary,
        ctx,
    );

    cell_y += tokens.weather_forecast_max_height;

    draw_centered_text(
        scene,
        &column.min,
        x,
        cell_y,
        width,
        tokens.weather_forecast_min_height,
        base * FORECAST_MIN_SCALE,
        &ctx.theme.typography.font_family.clone(),
        ctx.theme.palette.text_secondary,
        ctx,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_centered_text(
    scene: &mut Scene,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    size: f32,
    family: &str,
    color: vello::peniko::Color,
    ctx: &mut RenderCtx<'_>,
) {
    let (text_width, _) = ctx.text.measure(text, size, family);

    ctx.text
        .draw_centered_v(scene, text, x + (width - text_width) / 2.0, y, height, TextStyle::new(size, family, color));
}

fn city_name(label: &str) -> &str {
    label.split(',').next().map(str::trim).unwrap_or(label)
}
