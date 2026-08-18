// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, Local, NaiveDate};
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, Panel, PanelHeader, RenderCtx};
use crate::locale::weekday_abbrev;
use crate::render::{Rect, TextStyle};
use crate::theme::{Palette, Theme};

use super::icons::{UNKNOWN_WEATHER_ICON, weather_description, weather_icon, weather_icon_color};
use super::state::{DailyForecast, WeatherData, WeatherSnapshot};

// ─── < Constants > ────────────────────────────────────────────────────

const FORECAST_COLUMNS: usize = 5;
const VALUE_PLACEHOLDER: &str = "—";
const FALLBACK_TITLE: &str = "Clima";
const TODAY_LABEL: &str = "hoy";
const PRECIP_EMPHASIS_MIN: u8 = 50;

const HUMIDITY_GLYPH: &str = "\u{f058e}";
const WIND_GLYPH: &str = "\u{f059d}";
const PRECIPITATION_GLYPH: &str = "\u{f0576}";

const DETAIL_ICON_SCALE: f32 = 0.9;
const FORECAST_DAY_SCALE: f32 = 0.7;
const FORECAST_ICON_SCALE: f32 = 1.3;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WeatherPanel<'a> {
    pub data: &'a WeatherData,
}

struct DetailItem {
    glyph: &'static str,
    text: String,
    emphasis: bool,
}

struct ForecastColumn {
    day: &'static str,
    icon: &'static str,
    icon_color: vello::peniko::Color,
    max: String,
    min: String,
    is_today: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherPanel<'_> {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.dropdown_panel_padding_y * 2.0
            + tokens.dropdown_header_height
            + tokens.dropdown_section_gap
            + tokens.weather_details_row_height
            + tokens.dropdown_section_gap
            + forecast_height(theme)
    }
}

impl Panel for WeatherPanel<'_> {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.dropdown_panel_width, Self::height(theme))
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let theme = ctx.theme;
        let tokens = theme.tokens;
        let data = self.data;

        let inner_x = bounds.x + tokens.dropdown_panel_padding_x;
        let inner_width = bounds.width - tokens.dropdown_panel_padding_x * 2.0;
        let mut y = bounds.y + tokens.dropdown_panel_padding_y;

        draw_header(scene, inner_x, y, inner_width, data, ctx);

        y += tokens.dropdown_header_height + tokens.dropdown_section_gap;

        let items = detail_items(data.snapshot.as_ref());

        draw_details_row(scene, inner_x, y, inner_width, &items, ctx);

        y += tokens.weather_details_row_height;

        DropdownFrame::draw_divider(scene, inner_x, y + tokens.dropdown_section_gap / 2.0, inner_width, theme);

        y += tokens.dropdown_section_gap;

        let daily = data.snapshot.as_ref().map(|snapshot| snapshot.daily.as_slice()).unwrap_or(&[]);
        let today = Local::now().date_naive();

        draw_forecast(scene, inner_x, y, inner_width, daily, today, ctx);
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn forecast_height(theme: &Theme) -> f32 {
    let tokens = theme.tokens;

    tokens.weather_forecast_day_height
        + tokens.weather_forecast_icon_height
        + tokens.weather_forecast_max_height
        + tokens.weather_forecast_min_height
}

fn draw_header(scene: &mut Scene, x: f32, y: f32, width: f32, data: &WeatherData, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let base = ctx.theme.typography.size_base;
    let header_height = tokens.dropdown_header_height;

    let snapshot = data.snapshot.as_ref();

    let title = data.location_label.as_deref().map(city_name).unwrap_or(FALLBACK_TITLE);
    let subtitle = snapshot
        .map(|snapshot| weather_description(snapshot.weather_code))
        .unwrap_or(VALUE_PLACEHOLDER);

    PanelHeader { title, subtitle }.draw(scene, x, y, ctx);

    let temp_size = base * tokens.weather_temp_scale;
    let st_size = base * tokens.dropdown_subtitle_scale;

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
    let icon_size = base * tokens.weather_header_icon_scale;

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

    let rain_likely = snapshot
        .and_then(|snapshot| snapshot.precipitation_percent)
        .is_some_and(|value| value >= PRECIP_EMPHASIS_MIN);

    [
        DetailItem {
            glyph: HUMIDITY_GLYPH,
            text: humidity.unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
            emphasis: false,
        },
        DetailItem {
            glyph: WIND_GLYPH,
            text: wind.unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
            emphasis: false,
        },
        DetailItem {
            glyph: PRECIPITATION_GLYPH,
            text: precipitation.unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
            emphasis: rain_likely,
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
    let text_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_body_scale;

    let (icon_width, _) = ctx.text.measure(item.glyph, icon_size, &ctx.theme.typography.icon_font_family);
    let (text_width, _) = ctx.text.measure(&item.text, text_size, &ctx.theme.typography.font_family);

    icon_width + ctx.theme.tokens.weather_inner_gap + text_width
}

fn draw_detail_item(scene: &mut Scene, x: f32, y: f32, item: &DetailItem, ctx: &mut RenderCtx<'_>) {
    let row_height = ctx.theme.tokens.weather_details_row_height;
    let icon_size = ctx.theme.typography.size_base * DETAIL_ICON_SCALE;
    let text_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_body_scale;

    // Emphasized items (e.g. likely rain) light up in accent.
    let (icon_color, text_color) = if item.emphasis {
        (ctx.theme.palette.accent, ctx.theme.palette.accent)
    } else {
        (ctx.theme.palette.text_secondary, ctx.theme.palette.text_primary)
    };

    let (icon_width, _) = ctx.text.measure(item.glyph, icon_size, &ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        item.glyph,
        x,
        y,
        row_height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, icon_color),
    );

    ctx.text.draw_centered_v(
        scene,
        &item.text,
        x + icon_width + ctx.theme.tokens.weather_inner_gap,
        y,
        row_height,
        TextStyle::new(text_size, &ctx.theme.typography.font_family, text_color),
    );
}

fn forecast_columns(daily: &[DailyForecast], today: NaiveDate, palette: &Palette) -> Vec<ForecastColumn> {
    let mut columns: Vec<ForecastColumn> = daily
        .iter()
        .take(FORECAST_COLUMNS)
        .map(|forecast| {
            let is_today = forecast.date == today;

            ForecastColumn {
                day: if is_today {
                    TODAY_LABEL
                } else {
                    weekday_abbrev(forecast.date.weekday().num_days_from_monday() as usize)
                },
                icon: weather_icon(forecast.weather_code),
                icon_color: weather_icon_color(forecast.weather_code, palette),
                max: format!("{}°", forecast.max_c.round() as i32),
                min: format!("{}°", forecast.min_c.round() as i32),
                is_today,
            }
        })
        .collect();

    while columns.len() < FORECAST_COLUMNS {
        columns.push(ForecastColumn {
            day: VALUE_PLACEHOLDER,
            icon: UNKNOWN_WEATHER_ICON,
            icon_color: palette.text_secondary,
            max: VALUE_PLACEHOLDER.to_string(),
            min: VALUE_PLACEHOLDER.to_string(),
            is_today: false,
        });
    }

    columns
}

fn draw_forecast(scene: &mut Scene, x: f32, y: f32, width: f32, daily: &[DailyForecast], today: NaiveDate, ctx: &mut RenderCtx<'_>) {
    let columns = forecast_columns(daily, today, &ctx.theme.palette);
    let column_width = width / FORECAST_COLUMNS as f32;

    for (index, column) in columns.iter().enumerate() {
        let column_x = x + index as f32 * column_width;

        if column.is_today {
            draw_today_highlight(scene, column_x, y, column_width, ctx);
        }

        draw_forecast_column(scene, column_x, y, column_width, column, ctx);
    }
}

fn draw_today_highlight(scene: &mut Scene, x: f32, y: f32, width: f32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let pill_x = x + tokens.weather_today_pill_padding_x;
    let pill_width = width - tokens.weather_today_pill_padding_x * 2.0;
    let pill_y = y - tokens.weather_today_pill_inset_y;
    let pill_height = forecast_height(ctx.theme) + tokens.weather_today_pill_inset_y * 2.0;

    let body = RoundedRect::new(
        pill_x as f64,
        pill_y as f64,
        (pill_x + pill_width) as f64,
        (pill_y + pill_height) as f64,
        tokens.weather_today_pill_radius as f64,
    );

    let color = ctx.theme.palette.accent.with_alpha(tokens.date_week_highlight_alpha);

    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &body);
}

fn draw_forecast_column(scene: &mut Scene, x: f32, y: f32, width: f32, column: &ForecastColumn, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let base = ctx.theme.typography.size_base;

    let mut cell_y = y;

    // Today's label speaks for itself and picks up the accent.
    let day_color = if column.is_today {
        ctx.theme.palette.accent
    } else {
        ctx.theme.palette.text_secondary
    };

    ctx.text.draw_centered(
        scene,
        column.day,
        Rect::new(x, cell_y, width, tokens.weather_forecast_day_height),
        TextStyle::new(base * FORECAST_DAY_SCALE, &ctx.theme.typography.font_family, day_color),
    );

    cell_y += tokens.weather_forecast_day_height;

    ctx.text.draw_centered(
        scene,
        column.icon,
        Rect::new(x, cell_y, width, tokens.weather_forecast_icon_height),
        TextStyle::new(base * FORECAST_ICON_SCALE, &ctx.theme.typography.icon_font_family, column.icon_color),
    );

    cell_y += tokens.weather_forecast_icon_height;

    ctx.text.draw_centered(
        scene,
        &column.max,
        Rect::new(x, cell_y, width, tokens.weather_forecast_max_height),
        TextStyle::new(base * tokens.dropdown_body_scale, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    cell_y += tokens.weather_forecast_max_height;

    ctx.text.draw_centered(
        scene,
        &column.min,
        Rect::new(x, cell_y, width, tokens.weather_forecast_min_height),
        TextStyle::new(base * tokens.dropdown_subtitle_scale, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn city_name(label: &str) -> &str {
    label.split(',').next().map(str::trim).unwrap_or(label)
}
