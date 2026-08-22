// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::ClockAction;
use super::state::{Alarm, AlarmEditor, Repeat};

// ─── < Constants > ────────────────────────────────────────────────────

const PLUS_GLYPH: &str = "\u{f0415}";

const ALARM_ROW_H: f32 = 46.0;
const ALARM_TIME_SCALE: f32 = 1.35;
const ALARM_SUB_SCALE: f32 = 0.75;
const EMPTY_H: f32 = 40.0;
const NEW_ROW_H: f32 = 34.0;

/// Alarmas visibles; el resto se resume en "+N más".
const VISIBLE_ALARMS: usize = 6;

const SWITCH_W: f32 = 38.0;
const SWITCH_H: f32 = 21.0;
const KNOB_MARGIN: f32 = 3.0;

// Editor: hora con - y +, chips de repetición y botonera.
const PICKER_H: f32 = 48.0;
const PICKER_BUTTON: f32 = 26.0;
const PICKER_BUTTON_RADIUS: f64 = 8.0;
const PICKER_TIME_SCALE: f32 = 2.0;
const PICKER_GROUP_GAP: f32 = 10.0;

const CHIP_H: f32 = 26.0;
const CHIP_GAP: f32 = 8.0;
const CHIP_RADIUS: f64 = 9.0;
const CHIP_TEXT_SCALE: f32 = 0.75;

const EDITOR_BUTTON_H: f32 = 32.0;
const EDITOR_BUTTON_GAP: f32 = 8.0;
const EDITOR_BUTTON_RADIUS: f64 = 10.0;
const EDITOR_GAP: f32 = 14.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(theme: &Theme) -> f32 {
    let list = VISIBLE_ALARMS as f32 * ALARM_ROW_H + ALARM_ROW_H + NEW_ROW_H;

    list.max(editor_height(true, theme))
}

pub(crate) fn height(alarms: &[Alarm], editor: Option<&AlarmEditor>, theme: &Theme) -> f32 {
    if let Some(editor) = editor {
        return editor_height(editor.index.is_some(), theme);
    }

    if alarms.is_empty() {
        return EMPTY_H + NEW_ROW_H;
    }

    let visible = alarms.len().min(VISIBLE_ALARMS) as f32;
    let more = if alarms.len() > VISIBLE_ALARMS { ALARM_ROW_H * 0.5 } else { 0.0 };

    visible * ALARM_ROW_H + more + NEW_ROW_H
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, alarms: &[Alarm], editor: Option<&AlarmEditor>, ctx: &mut RenderCtx<'_>) {
    if let Some(editor) = editor {
        draw_editor(scene, area, editor, ctx);

        return;
    }

    let mut y = area.y;

    if alarms.is_empty() {
        let size = ctx.theme.typography.size_base * 0.9;

        ctx.text.draw_centered(
            scene,
            "sin alarmas",
            Rect::new(area.x, y, area.width, EMPTY_H),
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        y += EMPTY_H;
    } else {
        for (index, alarm) in alarms.iter().enumerate().take(VISIBLE_ALARMS) {
            draw_alarm_row(scene, Rect::new(area.x, y, area.width, ALARM_ROW_H), index, alarm, ctx);
            y += ALARM_ROW_H;
        }

        let hidden = alarms.len().saturating_sub(VISIBLE_ALARMS);

        if hidden > 0 {
            let size = ctx.theme.typography.size_base * ALARM_SUB_SCALE;

            ctx.text.draw_centered_v(
                scene,
                &format!("+{hidden} más"),
                area.x,
                y,
                ALARM_ROW_H * 0.5,
                TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );

            y += ALARM_ROW_H * 0.5;
        }
    }

    // "+ Nueva alarma".
    let is_hovered = ctx.hovered_interaction == Some(ClockAction::NewAlarm.interaction());

    let color = if is_hovered {
        ctx.theme.palette.text_primary
    } else {
        ctx.theme.palette.text_secondary
    };

    let icon_size = ctx.theme.typography.size_base;
    let size = ctx.theme.typography.size_base * 0.9;

    ctx.text.draw_centered_v(
        scene,
        PLUS_GLYPH,
        area.x,
        y,
        NEW_ROW_H,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, color),
    );

    ctx.text.draw_centered_v(
        scene,
        "Nueva alarma",
        area.x + 22.0,
        y,
        NEW_ROW_H,
        TextStyle::new(size, ctx.theme.typography.font_family, color),
    );
}

pub(crate) fn hit_test(point: Point, area: Rect, alarms: &[Alarm], editor: Option<&AlarmEditor>, theme: &Theme) -> Option<Interaction> {
    if let Some(editor) = editor {
        return editor_hit_test(point, area, editor, theme);
    }

    let mut y = area.y;

    if alarms.is_empty() {
        y += EMPTY_H;
    } else {
        for index in 0..alarms.len().min(VISIBLE_ALARMS) {
            let row = Rect::new(area.x, y, area.width, ALARM_ROW_H);

            if row.contains_point(point.x, point.y) {
                // El tercio derecho es del switch; el resto edita.
                let switch_zone = area.x + area.width - SWITCH_W * 2.0;

                return if point.x >= switch_zone {
                    Some(ClockAction::ToggleAlarm(index).interaction())
                } else {
                    Some(ClockAction::EditAlarm(index).interaction())
                };
            }

            y += ALARM_ROW_H;
        }

        if alarms.len() > VISIBLE_ALARMS {
            y += ALARM_ROW_H * 0.5;
        }
    }

    let new_row = Rect::new(area.x, y, area.width, NEW_ROW_H);

    new_row
        .contains_point(point.x, point.y)
        .then(|| ClockAction::NewAlarm.interaction())
}

// ─── < Private Functions: Lista > ────────────────────────────────────────────────────

fn draw_alarm_row(scene: &mut Scene, row: Rect, index: usize, alarm: &Alarm, ctx: &mut RenderCtx<'_>) {
    let time_size = ctx.theme.typography.size_base * ALARM_TIME_SCALE;
    let sub_size = ctx.theme.typography.size_base * ALARM_SUB_SCALE;

    let time_color = if alarm.enabled {
        ctx.theme.palette.text_primary
    } else {
        ctx.theme.palette.text_secondary
    };

    ctx.text.draw_centered_v(
        scene,
        &alarm.time_text(),
        row.x,
        row.y + 4.0,
        row.height * 0.55,
        TextStyle::new(time_size, ctx.theme.typography.font_family, time_color),
    );

    ctx.text.draw_centered_v(
        scene,
        &alarm.subtitle(),
        row.x,
        row.y + row.height * 0.52,
        row.height * 0.4,
        TextStyle::new(sub_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    draw_switch(scene, row, index, alarm.enabled, ctx);
}

fn draw_switch(scene: &mut Scene, row: Rect, index: usize, enabled: bool, ctx: &mut RenderCtx<'_>) {
    let x = row.x + row.width - SWITCH_W;
    let y = row.y + (row.height - SWITCH_H) / 2.0;
    let radius = (SWITCH_H / 2.0) as f64;

    let hovered = ctx.hovered_interaction == Some(ClockAction::ToggleAlarm(index).interaction());

    let track_color = if enabled {
        ctx.theme.palette.accent
    } else if hovered {
        ctx.theme.palette.slot_hover_bg
    } else {
        ctx.theme.palette.slot_inactive_bg
    };

    let track = RoundedRect::new(x as f64, y as f64, (x + SWITCH_W) as f64, (y + SWITCH_H) as f64, radius);
    scene.fill(Fill::NonZero, Affine::IDENTITY, track_color, None, &track);

    let knob_radius = SWITCH_H / 2.0 - KNOB_MARGIN;

    let knob_x = if enabled {
        x + SWITCH_W - KNOB_MARGIN - knob_radius
    } else {
        x + KNOB_MARGIN + knob_radius
    };

    let knob = Circle::new((knob_x as f64, (y + SWITCH_H / 2.0) as f64), knob_radius as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, Color::WHITE, None, &knob);
}

// ─── < Private Functions: Editor > ────────────────────────────────────────────────────

fn editor_height(existing: bool, _theme: &Theme) -> f32 {
    let delete = if existing { EDITOR_BUTTON_H + EDITOR_BUTTON_GAP } else { 0.0 };

    PICKER_H + EDITOR_GAP + CHIP_H + EDITOR_GAP + EDITOR_BUTTON_H + delete
}

fn draw_editor(scene: &mut Scene, area: Rect, editor: &AlarmEditor, ctx: &mut RenderCtx<'_>) {
    let mut y = area.y;

    draw_picker(scene, area, y, editor, ctx);
    y += PICKER_H + EDITOR_GAP;

    // Chips de repetición.
    for (repeat, rect) in chip_rects(area, y) {
        let is_active = editor.repeat == repeat;
        let is_hovered = ctx.hovered_interaction == Some(ClockAction::EditorRepeat(repeat).interaction());

        let (background, color) = if is_active {
            (ctx.theme.palette.slot_active_bg, ctx.theme.palette.slot_active_text)
        } else if is_hovered {
            (ctx.theme.palette.control_hover_bg, ctx.theme.palette.text_primary)
        } else {
            (ctx.theme.palette.control_bg, ctx.theme.palette.text_secondary)
        };

        let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, CHIP_RADIUS);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let size = ctx.theme.typography.size_base * CHIP_TEXT_SCALE;

        ctx.text
            .draw_centered(scene, repeat.label(), rect, TextStyle::new(size, ctx.theme.typography.font_family, color));
    }

    y += CHIP_H + EDITOR_GAP;

    // Cancelar / Guardar (+ Borrar si es una alarma existente).
    for (action, rect) in editor_button_rects(area, y, editor.index.is_some()) {
        let is_hovered = ctx.hovered_interaction == Some(action.interaction());

        let (background, foreground, label) = match action {
            ClockAction::EditorSave if is_hovered => (ctx.theme.palette.control_hover_bg, ctx.theme.palette.text_primary, "Guardar"),
            ClockAction::EditorSave => (ctx.theme.palette.accent, Color::WHITE, "Guardar"),
            ClockAction::DeleteAlarm if is_hovered => (ctx.theme.palette.danger_bg, ctx.theme.palette.danger_text, "Borrar"),
            ClockAction::DeleteAlarm => (ctx.theme.palette.control_bg, ctx.theme.palette.meter_critical, "Borrar"),
            _ if is_hovered => (ctx.theme.palette.control_hover_bg, ctx.theme.palette.text_primary, "Cancelar"),
            _ => (ctx.theme.palette.control_bg, ctx.theme.palette.text_primary, "Cancelar"),
        };

        let body = RoundedRect::new(
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            (rect.y + rect.height) as f64,
            EDITOR_BUTTON_RADIUS,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let size = ctx.theme.typography.size_base * 0.85;

        ctx.text
            .draw_centered(scene, label, rect, TextStyle::new(size, ctx.theme.typography.font_family, foreground));
    }
}

/// − HH + : − MM +, centrado.
fn draw_picker(scene: &mut Scene, area: Rect, y: f32, editor: &AlarmEditor, ctx: &mut RenderCtx<'_>) {
    let time_size = ctx.theme.typography.size_base * PICKER_TIME_SCALE;

    for (action, rect) in picker_button_rects(area, y) {
        let is_hovered = ctx.hovered_interaction == Some(action.interaction());

        let background = if is_hovered {
            ctx.theme.palette.control_hover_bg
        } else {
            ctx.theme.palette.control_bg
        };

        let body = RoundedRect::new(
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            (rect.y + rect.height) as f64,
            PICKER_BUTTON_RADIUS,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let glyph = match action {
            ClockAction::EditorHourUp | ClockAction::EditorMinuteUp => "+",
            _ => "−",
        };

        ctx.text.draw_centered(
            scene,
            glyph,
            rect,
            TextStyle::new(ctx.theme.typography.size_base, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );
    }

    // Hora y minutos entre sus botones.
    let [hour_slot, minute_slot] = picker_value_slots(area, y);

    ctx.text.draw_centered(
        scene,
        &format!("{:02}", editor.hour),
        hour_slot,
        TextStyle::new(time_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered(
        scene,
        &format!("{:02}", editor.minute),
        minute_slot,
        TextStyle::new(time_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let separator_x = area.x + area.width / 2.0;

    ctx.text.draw_centered(
        scene,
        ":",
        Rect::new(separator_x - 6.0, y, 12.0, PICKER_H),
        TextStyle::new(time_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn editor_hit_test(point: Point, area: Rect, editor: &AlarmEditor, _theme: &Theme) -> Option<Interaction> {
    let picker_y = area.y;

    for (action, rect) in picker_button_rects(area, picker_y) {
        if rect.contains_point(point.x, point.y) {
            return Some(action.interaction());
        }
    }

    let chips_y = picker_y + PICKER_H + EDITOR_GAP;

    for (repeat, rect) in chip_rects(area, chips_y) {
        if rect.contains_point(point.x, point.y) {
            return Some(ClockAction::EditorRepeat(repeat).interaction());
        }
    }

    let buttons_y = chips_y + CHIP_H + EDITOR_GAP;

    editor_button_rects(area, buttons_y, editor.index.is_some())
        .into_iter()
        .find(|(_, rect)| rect.contains_point(point.x, point.y))
        .map(|(action, _)| action.interaction())
}

/// Los cuatro botones −/+ del picker: [− HH +] [− MM +].
fn picker_button_rects(area: Rect, y: f32) -> [(ClockAction, Rect); 4] {
    let group_width = PICKER_BUTTON * 2.0 + PICKER_GROUP_GAP * 2.0 + value_width();
    let total = group_width * 2.0 + PICKER_GROUP_GAP * 2.0 + 12.0;
    let start_x = area.x + (area.width - total) / 2.0;
    let button_y = y + (PICKER_H - PICKER_BUTTON) / 2.0;

    let hour_x = start_x;
    let minute_x = start_x + group_width + PICKER_GROUP_GAP * 2.0 + 12.0;

    [
        (ClockAction::EditorHourDown, Rect::new(hour_x, button_y, PICKER_BUTTON, PICKER_BUTTON)),
        (
            ClockAction::EditorHourUp,
            Rect::new(hour_x + PICKER_BUTTON + PICKER_GROUP_GAP * 2.0 + value_width(), button_y, PICKER_BUTTON, PICKER_BUTTON),
        ),
        (ClockAction::EditorMinuteDown, Rect::new(minute_x, button_y, PICKER_BUTTON, PICKER_BUTTON)),
        (
            ClockAction::EditorMinuteUp,
            Rect::new(minute_x + PICKER_BUTTON + PICKER_GROUP_GAP * 2.0 + value_width(), button_y, PICKER_BUTTON, PICKER_BUTTON),
        ),
    ]
}

fn picker_value_slots(area: Rect, y: f32) -> [Rect; 2] {
    let rects = picker_button_rects(area, y);

    let hour_start = rects[0].1.x + PICKER_BUTTON + PICKER_GROUP_GAP;
    let minute_start = rects[2].1.x + PICKER_BUTTON + PICKER_GROUP_GAP;

    [
        Rect::new(hour_start, y, value_width(), PICKER_H),
        Rect::new(minute_start, y, value_width(), PICKER_H),
    ]
}

/// Ancho reservado para "00" en grande.
fn value_width() -> f32 {
    34.0
}

fn chip_rects(area: Rect, y: f32) -> [(Repeat, Rect); 3] {
    let count = Repeat::ALL.len() as f32;
    let width = (area.width - CHIP_GAP * (count - 1.0)) / count;

    std::array::from_fn(|index| {
        let x = area.x + index as f32 * (width + CHIP_GAP);

        (Repeat::ALL[index], Rect::new(x, y, width, CHIP_H))
    })
}

fn editor_button_rects(area: Rect, y: f32, existing: bool) -> Vec<(ClockAction, Rect)> {
    let width = (area.width - EDITOR_BUTTON_GAP) / 2.0;

    let mut rects = vec![
        (ClockAction::EditorCancel, Rect::new(area.x, y, width, EDITOR_BUTTON_H)),
        (ClockAction::EditorSave, Rect::new(area.x + width + EDITOR_BUTTON_GAP, y, width, EDITOR_BUTTON_H)),
    ];

    if existing {
        let delete_y = y + EDITOR_BUTTON_H + EDITOR_BUTTON_GAP;

        rects.push((ClockAction::DeleteAlarm, Rect::new(area.x, delete_y, area.width, EDITOR_BUTTON_H)));
    }

    rects
}
