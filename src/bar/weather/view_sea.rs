// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::components::RenderCtx;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::state::{SeaInfo, Tide};

// ─── < Constants > ────────────────────────────────────────────────────

const UP_GLYPH: &str = "\u{f005d}";
const DOWN_GLYPH: &str = "\u{f0045}";

const LABEL_H: f32 = 16.0;
const VALUE_H: f32 = 26.0;
const SECTION_GAP: f32 = 14.0;

const CARD_RADIUS: f64 = 12.0;
const TIDE_ROW_H: f32 = 34.0;
const CARD_PADDING_X: f32 = 12.0;

const EMPTY_H: f32 = 48.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    LABEL_H + VALUE_H + SECTION_GAP + TIDE_ROW_H * 2.0
}

pub(crate) fn height(sea: Option<&SeaInfo>, _theme: &Theme) -> f32 {
    if sea.is_some() {
        LABEL_H + VALUE_H + SECTION_GAP + TIDE_ROW_H * 2.0
    } else {
        EMPTY_H
    }
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, sea: Option<&SeaInfo>, ctx: &mut RenderCtx<'_>) {
    let Some(sea) = sea else {
        let size = ctx.theme.typography.size_base * 0.9;

        ctx.text.draw_centered(
            scene,
            "sin datos del mar para esta ubicación",
            Rect::new(area.x, area.y, area.width, EMPTY_H),
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        return;
    };

    let mut y = area.y;
    let half = area.width / 2.0;

    // AGUA | OLAS.
    let label_size = ctx.theme.typography.size_base * 0.72;
    let value_size = ctx.theme.typography.size_base * 1.15;
    let unit_size = ctx.theme.typography.size_base * 0.82;

    let label_style = TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary);

    ctx.text.draw_centered_v(scene, "AGUA", area.x, y, LABEL_H, label_style);
    ctx.text.draw_centered_v(scene, "OLAS", area.x + half, y, LABEL_H, label_style);

    y += LABEL_H;

    let water = sea
        .water_temp_c
        .map(|temp| format!("{}°", temp.round() as i32))
        .unwrap_or_else(|| "—".to_string());

    ctx.text.draw_centered_v(
        scene,
        &water,
        area.x,
        y,
        VALUE_H,
        TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let waves = sea
        .wave_height_m
        .map(|height| format!("{height:.1} m").replace('.', ","))
        .unwrap_or_else(|| "—".to_string());

    let (waves_width, _) = ctx.text.measure(&waves, value_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &waves,
        area.x + half,
        y,
        VALUE_H,
        TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    if let Some(direction) = sea.wave_direction {
        ctx.text.draw_centered_v(
            scene,
            &format!("del {direction}"),
            area.x + half + waves_width + 6.0,
            y,
            VALUE_H,
            TextStyle::new(unit_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }

    y += VALUE_H + SECTION_GAP;

    // Tarjeta de mareas.
    let card = Rect::new(area.x, y, area.width, TIDE_ROW_H * 2.0);
    let body = RoundedRect::new(card.x as f64, card.y as f64, (card.x + card.width) as f64, (card.y + card.height) as f64, CARD_RADIUS);

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_raised, None, &body);

    draw_tide_row(scene, Rect::new(card.x, card.y, card.width, TIDE_ROW_H), UP_GLYPH, "Marea alta", sea.tide_high.as_ref(), ctx);

    let divider = vello::kurbo::Rect::new(
        (card.x + CARD_PADDING_X) as f64,
        (card.y + TIDE_ROW_H - 0.5) as f64,
        (card.x + card.width - CARD_PADDING_X) as f64,
        (card.y + TIDE_ROW_H + 0.5) as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_divider, None, &divider);

    draw_tide_row(
        scene,
        Rect::new(card.x, card.y + TIDE_ROW_H, card.width, TIDE_ROW_H),
        DOWN_GLYPH,
        "Marea baja",
        sea.tide_low.as_ref(),
        ctx,
    );
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_tide_row(scene: &mut Scene, row: Rect, glyph: &str, label: &str, tide: Option<&Tide>, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * 0.9;
    let text_size = ctx.theme.typography.size_base * 0.9;
    let small_size = ctx.theme.typography.size_base * 0.78;

    ctx.text.draw_centered_v(
        scene,
        glyph,
        row.x + CARD_PADDING_X,
        row.y,
        row.height,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.accent),
    );

    ctx.text.draw_centered_v(
        scene,
        label,
        row.x + CARD_PADDING_X + 20.0,
        row.y,
        row.height,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let Some(tide) = tide else {
        return;
    };

    let height = format!("{:.1} m", tide.height_m).replace('.', ",");
    let (height_width, _) = ctx.text.measure(&height, small_size, ctx.theme.typography.font_family);
    let height_x = row.x + row.width - CARD_PADDING_X - height_width;

    ctx.text.draw_centered_v(
        scene,
        &height,
        height_x,
        row.y,
        row.height,
        TextStyle::new(small_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    let (time_width, _) = ctx.text.measure(&tide.time, text_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &tide.time,
        height_x - 12.0 - time_width,
        row.y,
        row.height,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
}
