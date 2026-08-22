// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{DropdownFrame, Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::ClockAction;
use super::state::StopwatchData;

// ─── < Constants > ────────────────────────────────────────────────────

const RESET_GLYPH: &str = "\u{f0450}";

const DISPLAY_H: f32 = 52.0;
const DISPLAY_SCALE: f32 = 2.2;
const CENTIS_SCALE: f32 = 1.2;

const BUTTON_H: f32 = 34.0;
const BUTTON_GAP: f32 = 8.0;
const BUTTON_RADIUS: f64 = 10.0;
const SECTION_GAP: f32 = 12.0;

const LAP_ROW_H: f32 = 22.0;
const LAP_TEXT_SCALE: f32 = 0.82;
const HINT_H: f32 = 34.0;

/// Vueltas visibles; el resto se resume en "+N más".
const VISIBLE_LAPS: usize = 5;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    DISPLAY_H + SECTION_GAP + BUTTON_H + SECTION_GAP + VISIBLE_LAPS as f32 * LAP_ROW_H + LAP_ROW_H
}

pub(crate) fn height(stopwatch: &StopwatchData, _theme: &Theme) -> f32 {
    DISPLAY_H + SECTION_GAP + BUTTON_H + SECTION_GAP + laps_height(stopwatch)
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, stopwatch: &StopwatchData, ctx: &mut RenderCtx<'_>) {
    let mut y = area.y;

    // Tiempo grande centrado, con centésimas más chicas.
    let (main, centis) = format_stopwatch(stopwatch.elapsed());

    let main_size = ctx.theme.typography.size_base * DISPLAY_SCALE;
    let centis_size = ctx.theme.typography.size_base * CENTIS_SCALE;

    let (main_width, _) = ctx.text.measure(&main, main_size, ctx.theme.typography.font_family);
    let (centis_width, _) = ctx.text.measure(&centis, centis_size, ctx.theme.typography.font_family);

    let text_x = area.x + (area.width - main_width - centis_width) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        &main,
        text_x,
        y,
        DISPLAY_H,
        TextStyle::new(main_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        &centis,
        text_x + main_width,
        y,
        DISPLAY_H,
        TextStyle::new(centis_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    y += DISPLAY_H + SECTION_GAP;

    // Iniciar/Pausar + Vuelta + reset.
    for (action, rect) in button_rects(area, y) {
        let is_hovered = ctx.hovered_interaction == Some(action.interaction());
        let (background, foreground, label) = button_style(action, stopwatch, is_hovered, ctx.theme);

        let body =
            RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, BUTTON_RADIUS);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let (style, text) = match action {
            ClockAction::StopwatchReset => {
                (TextStyle::new(ctx.theme.typography.size_base * 1.1, ctx.theme.typography.icon_font_family, foreground), RESET_GLYPH)
            }
            _ => (TextStyle::new(ctx.theme.typography.size_base * 0.9, ctx.theme.typography.font_family, foreground), label),
        };

        ctx.text.draw_centered(scene, text, rect, style);
    }

    y += BUTTON_H + SECTION_GAP;

    // Vueltas (o el hint cuando no hay ninguna).
    if stopwatch.laps.is_empty() {
        let size = ctx.theme.typography.size_base * LAP_TEXT_SCALE;

        ctx.text.draw_centered(
            scene,
            "Pulsa Vuelta para marcar un parcial",
            Rect::new(area.x, y, area.width, HINT_H),
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        return;
    }

    DropdownFrame::draw_divider(scene, area.x, y, area.width, ctx.theme);
    y += 6.0;

    let size = ctx.theme.typography.size_base * LAP_TEXT_SCALE;
    let hidden = stopwatch.laps.len().saturating_sub(VISIBLE_LAPS);

    // Las últimas vueltas arriba: lo recién marcado se ve al toque.
    for (shown, (index, lap)) in stopwatch.laps.iter().enumerate().rev().take(VISIBLE_LAPS).enumerate() {
        let row_y = y + shown as f32 * LAP_ROW_H;
        let (main, centis) = format_stopwatch(*lap);
        let time_text = format!("{main}{centis}");

        ctx.text.draw_centered_v(
            scene,
            &format!("Vuelta {}", index + 1),
            area.x,
            row_y,
            LAP_ROW_H,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        let (time_width, _) = ctx.text.measure(&time_text, size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            &time_text,
            area.x + area.width - time_width,
            row_y,
            LAP_ROW_H,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );
    }

    if hidden > 0 {
        let row_y = y + VISIBLE_LAPS as f32 * LAP_ROW_H;

        ctx.text.draw_centered_v(
            scene,
            &format!("+{hidden} más"),
            area.x,
            row_y,
            LAP_ROW_H,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}

pub(crate) fn hit_test(point: Point, area: Rect, _theme: &Theme) -> Option<Interaction> {
    let buttons_y = area.y + DISPLAY_H + SECTION_GAP;

    button_rects(area, buttons_y)
        .into_iter()
        .find(|(_, rect)| rect.contains_point(point.x, point.y))
        .map(|(action, _)| action.interaction())
}

/// "MM:SS" + ",cc"; pasa a "H:MM:SS" arriba de la hora.
pub fn format_stopwatch(elapsed: Duration) -> (String, String) {
    let total_centis = elapsed.as_millis() / 10;
    let centis = (total_centis % 100) as u32;
    let total_seconds = (total_centis / 100) as u64;

    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;

    let main = if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    };

    (main, format!(",{centis:02}"))
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn laps_height(stopwatch: &StopwatchData) -> f32 {
    if stopwatch.laps.is_empty() {
        return HINT_H;
    }

    let visible = stopwatch.laps.len().min(VISIBLE_LAPS) as f32;
    let more = if stopwatch.laps.len() > VISIBLE_LAPS { LAP_ROW_H } else { 0.0 };

    6.0 + visible * LAP_ROW_H + more
}

fn button_rects(area: Rect, y: f32) -> [(ClockAction, Rect); 3] {
    // Reset cuadrado a la derecha; el resto se reparte en dos.
    let reset_w = BUTTON_H;
    let flexible = (area.width - reset_w - BUTTON_GAP * 2.0) / 2.0;

    [
        (ClockAction::StopwatchToggle, Rect::new(area.x, y, flexible, BUTTON_H)),
        (ClockAction::StopwatchLap, Rect::new(area.x + flexible + BUTTON_GAP, y, flexible, BUTTON_H)),
        (ClockAction::StopwatchReset, Rect::new(area.x + area.width - reset_w, y, reset_w, BUTTON_H)),
    ]
}

fn button_style(action: ClockAction, stopwatch: &StopwatchData, hovered: bool, theme: &Theme) -> (Color, Color, &'static str) {
    match action {
        ClockAction::StopwatchToggle => {
            let label = if stopwatch.running() { "Pausar" } else { "Iniciar" };
            let background = if hovered {
                theme.palette.control_hover_bg
            } else {
                theme.palette.accent
            };
            let foreground = if hovered { theme.palette.text_primary } else { Color::WHITE };

            (background, foreground, label)
        }
        ClockAction::StopwatchLap => {
            // Sin cronómetro corriendo la vuelta no hace nada: se atenúa.
            let foreground = if stopwatch.running() {
                theme.palette.text_primary
            } else {
                theme.palette.text_secondary
            };

            let background = if hovered && stopwatch.running() {
                theme.palette.control_hover_bg
            } else {
                theme.palette.control_bg
            };

            (background, foreground, "Vuelta")
        }
        _ => {
            let background = if hovered {
                theme.palette.control_hover_bg
            } else {
                theme.palette.control_bg
            };

            (background, theme.palette.text_primary, "")
        }
    }
}
