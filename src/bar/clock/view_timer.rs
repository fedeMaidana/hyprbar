// ─── < Imports > ────────────────────────────────────────────────────

use std::f64::consts::{FRAC_PI_2, TAU};
use std::time::Duration;

use vello::Scene;
use vello::kurbo::{Affine, Arc, Circle, RoundedRect, Stroke, Vec2};
use vello::peniko::{Color, Fill};

use crate::components::{Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::ClockAction;
use super::state::TimerData;

// ─── < Constants > ────────────────────────────────────────────────────

const RESET_GLYPH: &str = "\u{f0450}";

/// Presets del mockup, en minutos.
const PRESETS: [u32; 4] = [1, 5, 10, 25];

const RING_SIZE: f32 = 132.0;
const RING_STROKE: f64 = 7.0;
const RING_TIME_SCALE: f32 = 2.0;
const RING_STATE_SCALE: f32 = 0.65;

const PRESET_H: f32 = 26.0;
const PRESET_GAP: f32 = 8.0;
const PRESET_RADIUS: f64 = 9.0;
const PRESET_TEXT_SCALE: f32 = 0.78;

const BUTTON_H: f32 = 34.0;
const BUTTON_GAP: f32 = 8.0;
const BUTTON_RADIUS: f64 = 10.0;
const SECTION_GAP: f32 = 14.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn height(_theme: &Theme) -> f32 {
    RING_SIZE + SECTION_GAP + PRESET_H + SECTION_GAP + BUTTON_H
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, timer: &TimerData, ctx: &mut RenderCtx<'_>) {
    let mut y = area.y;

    draw_ring(scene, area, y, timer, ctx);
    y += RING_SIZE + SECTION_GAP;

    // Presets.
    for (minutes, rect) in preset_rects(area, y) {
        let is_active = timer.duration == Duration::from_secs(u64::from(minutes) * 60);
        let is_hovered = ctx.hovered_interaction == Some(ClockAction::TimerPreset(minutes).interaction());

        let background = if is_active {
            ctx.theme.palette.pill_hover_bg
        } else if is_hovered {
            ctx.theme.palette.control_hover_bg
        } else {
            ctx.theme.palette.control_bg
        };

        let body =
            RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, PRESET_RADIUS);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let color = if is_active {
            ctx.theme.palette.text_primary
        } else {
            ctx.theme.palette.text_secondary
        };

        let size = ctx.theme.typography.size_base * PRESET_TEXT_SCALE;

        ctx.text
            .draw_centered(scene, &format!("{minutes} min"), rect, TextStyle::new(size, ctx.theme.typography.font_family, color));
    }

    y += PRESET_H + SECTION_GAP;

    // Iniciar/Pausar + reset.
    for (action, rect) in button_rects(area, y) {
        let is_hovered = ctx.hovered_interaction == Some(action.interaction());

        let (background, foreground) = match action {
            ClockAction::TimerToggle if is_hovered => (ctx.theme.palette.control_hover_bg, ctx.theme.palette.text_primary),
            ClockAction::TimerToggle => (ctx.theme.palette.accent, Color::WHITE),
            _ if is_hovered => (ctx.theme.palette.control_hover_bg, ctx.theme.palette.text_primary),
            _ => (ctx.theme.palette.control_bg, ctx.theme.palette.text_primary),
        };

        let body =
            RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, BUTTON_RADIUS);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        match action {
            ClockAction::TimerReset => {
                let size = ctx.theme.typography.size_base * 1.1;

                ctx.text
                    .draw_centered(scene, RESET_GLYPH, rect, TextStyle::new(size, ctx.theme.typography.icon_font_family, foreground));
            }
            _ => {
                let label = if timer.running() { "Pausar" } else { "Iniciar" };
                let size = ctx.theme.typography.size_base * 0.9;

                ctx.text
                    .draw_centered(scene, label, rect, TextStyle::new(size, ctx.theme.typography.font_family, foreground));
            }
        }
    }
}

pub(crate) fn hit_test(point: Point, area: Rect, _theme: &Theme) -> Option<Interaction> {
    let presets_y = area.y + RING_SIZE + SECTION_GAP;

    for (minutes, rect) in preset_rects(area, presets_y) {
        if rect.contains_point(point.x, point.y) {
            return Some(ClockAction::TimerPreset(minutes).interaction());
        }
    }

    let buttons_y = presets_y + PRESET_H + SECTION_GAP;

    button_rects(area, buttons_y)
        .into_iter()
        .find(|(_, rect)| rect.contains_point(point.x, point.y))
        .map(|(action, _)| action.interaction())
}

/// "MM:SS" del tiempo restante.
pub fn format_timer(remaining: Duration) -> String {
    let total = remaining.as_secs();

    format!("{:02}:{:02}", total / 60, total % 60)
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_ring(scene: &mut Scene, area: Rect, y: f32, timer: &TimerData, ctx: &mut RenderCtx<'_>) {
    let center_x = (area.x + area.width / 2.0) as f64;
    let center_y = (y + RING_SIZE / 2.0) as f64;
    let radius = (RING_SIZE / 2.0) as f64 - RING_STROKE;

    // Pista completa + arco con lo que queda, arrancando arriba.
    let track = Circle::new((center_x, center_y), radius);
    scene.stroke(&Stroke::new(RING_STROKE), Affine::IDENTITY, ctx.theme.palette.panel_raised, None, &track);

    let fraction = timer.fraction_remaining();

    if fraction > 0.0 {
        let sweep = f64::from(fraction) * TAU;

        let arc = Arc {
            center: (center_x, center_y).into(),
            radii: Vec2::new(radius, radius),
            start_angle: -FRAC_PI_2,
            sweep_angle: sweep,
            x_rotation: 0.0,
        };

        scene.stroke(&Stroke::new(RING_STROKE), Affine::IDENTITY, ctx.theme.palette.accent, None, &arc);
    }

    // Tiempo restante + estado, centrados dentro del anillo.
    let time_text = format_timer(timer.remaining());
    let time_size = ctx.theme.typography.size_base * RING_TIME_SCALE;
    let state_size = ctx.theme.typography.size_base * RING_STATE_SCALE;

    let state_text = if timer.running() {
        "EN MARCHA"
    } else if timer.finished {
        "TERMINADO"
    } else {
        "EN PAUSA"
    };

    let ring_rect = Rect::new(area.x, y, area.width, RING_SIZE);

    ctx.text.draw_centered(
        scene,
        &time_text,
        Rect::new(ring_rect.x, ring_rect.y - 8.0, ring_rect.width, ring_rect.height),
        TextStyle::new(time_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let state_color = if timer.finished {
        ctx.theme.palette.accent
    } else {
        ctx.theme.palette.text_secondary
    };

    ctx.text.draw_centered(
        scene,
        state_text,
        Rect::new(ring_rect.x, ring_rect.y + 18.0, ring_rect.width, ring_rect.height),
        TextStyle::new(state_size, ctx.theme.typography.font_family, state_color),
    );
}

fn preset_rects(area: Rect, y: f32) -> [(u32, Rect); 4] {
    let count = PRESETS.len() as f32;
    let width = (area.width - PRESET_GAP * (count - 1.0)) / count;

    std::array::from_fn(|index| {
        let x = area.x + index as f32 * (width + PRESET_GAP);

        (PRESETS[index], Rect::new(x, y, width, PRESET_H))
    })
}

fn button_rects(area: Rect, y: f32) -> [(ClockAction, Rect); 2] {
    let reset_w = BUTTON_H;
    let main_w = area.width - reset_w - BUTTON_GAP;

    [
        (ClockAction::TimerToggle, Rect::new(area.x, y, main_w, BUTTON_H)),
        (ClockAction::TimerReset, Rect::new(area.x + main_w + BUTTON_GAP, y, reset_w, BUTTON_H)),
    ]
}
