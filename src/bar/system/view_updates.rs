// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::SystemAction;
use super::charts::{
    card_height, draw_big_value, draw_card_background, draw_info_card, draw_section_label, section_card, section_card_height,
};
use super::state::SystemData;

// ─── < Constants > ────────────────────────────────────────────────────

const VALUE_PLACEHOLDER: &str = "—";

const TERMINAL_GLYPH: &str = "\u{f018d}";
const CHECK_GLYPH: &str = "\u{f05e0}";
const UPDATE_COMMAND_LABEL: &str = "pacman -Syu";

// Estado "al día": solo un tilde grande centrado con su mensaje debajo.
const ALL_CLEAR_ICON_SCALE: f32 = 2.6;
const ALL_CLEAR_ICON_H: f32 = 44.0;
const ALL_CLEAR_GAP: f32 = 6.0;
const ALL_CLEAR_TEXT_H: f32 = 18.0;
const ALL_CLEAR_H: f32 = ALL_CLEAR_ICON_H + ALL_CLEAR_GAP + ALL_CLEAR_TEXT_H;

const LABEL_H: f32 = 14.0;
const MAIN_H: f32 = 30.0;
const META_H: f32 = 14.0;
const INNER_GAP: f32 = 4.0;
const SECTION_GAP: f32 = 10.0;

const BUTTON_H: f32 = 38.0;
const BUTTON_RADIUS: f64 = 12.0;

const EMPTY_LIST_H: f32 = 36.0;

/// Cuántos paquetes entran en la lista (el resto va al "+N more").
const VISIBLE_PACKAGES: usize = 5;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    header_height() + SECTION_GAP + card_height(VISIBLE_PACKAGES) + SECTION_GAP + META_H + SECTION_GAP + BUTTON_H
}

pub(crate) fn height(data: &SystemData, _theme: &Theme) -> f32 {
    if is_all_clear(data) {
        return ALL_CLEAR_H;
    }

    header_height() + SECTION_GAP + list_height(data) + SECTION_GAP + meta_height(data) + BUTTON_H
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    if is_all_clear(data) {
        draw_all_clear(scene, area, ctx);

        return;
    }

    let updates = &data.updates;
    let mut y = area.y;

    // PENDING: contador grande en su tarjeta.
    let card = section_card(scene, Rect::new(area.x, y, area.width, header_height()), ctx.theme);
    let mut row = card.y;

    draw_section_label(scene, card.x, row, LABEL_H, "PENDING", ctx);
    row += LABEL_H + INNER_GAP;

    let count_text = updates
        .pending
        .map(|count| count.to_string())
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    let end_x = draw_big_value(scene, card.x, row, MAIN_H, &count_text, "", ctx);

    let word = if updates.pending == Some(1) { "package" } else { "packages" };
    let word_size = ctx.theme.typography.size_base * 0.82;

    ctx.text.draw_centered_v(
        scene,
        word,
        end_x + 6.0,
        row,
        MAIN_H,
        TextStyle::new(word_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    y += header_height() + SECTION_GAP;

    // Lista de paquetes.
    if updates.packages.is_empty() {
        let size = ctx.theme.typography.size_base * 0.9;

        let empty_card = Rect::new(area.x, y, area.width, EMPTY_LIST_H);
        draw_card_background(scene, empty_card, ctx.theme);

        ctx.text.draw_centered(
            scene,
            "sin datos todavía",
            empty_card,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        y += EMPTY_LIST_H + SECTION_GAP;
    } else {
        let rows: Vec<(&str, String)> = updates
            .packages
            .iter()
            .take(VISIBLE_PACKAGES)
            .map(|package| (package.name.as_str(), package.version.clone()))
            .collect();

        let card = Rect::new(area.x, y, area.width, card_height(rows.len()));
        draw_info_card(scene, card, &rows, None, ctx);

        y += card.height + SECTION_GAP;
    }

    // Meta: cuántos quedan fuera de la lista. Sin resto, no ocupa lugar.
    let extra = updates
        .pending
        .map(|count| count as usize)
        .filter(|count| *count > VISIBLE_PACKAGES);

    if let Some(count) = extra {
        let more = format!("+{} more", count - VISIBLE_PACKAGES);
        let size = ctx.theme.typography.size_base * 0.78;

        ctx.text.draw_centered_v(
            scene,
            &more,
            area.x,
            y,
            META_H,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        y += META_H + SECTION_GAP;
    }

    // Botón pacman -Syu.
    draw_update_button(scene, Rect::new(area.x, y, area.width, BUTTON_H), ctx);
}

pub(crate) fn hit_test(point: Point, area: Rect, data: &SystemData, _theme: &Theme) -> Option<Interaction> {
    if is_all_clear(data) {
        return None;
    }

    let button_y = area.y + header_height() + SECTION_GAP + list_height(data) + SECTION_GAP + meta_height(data);
    let button = Rect::new(area.x, button_y, area.width, BUTTON_H);

    button
        .contains_point(point.x, point.y)
        .then_some(SystemAction::RunUpdate.interaction())
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn header_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + MAIN_H)
}

/// Al día confirmado: cero pendientes. Sin datos todavía no cuenta.
fn is_all_clear(data: &SystemData) -> bool {
    data.updates.pending == Some(0) && data.updates.packages.is_empty()
}

/// Tilde grande centrado con el mensaje debajo; reemplaza toda la vista.
fn draw_all_clear(scene: &mut Scene, area: Rect, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * ALL_CLEAR_ICON_SCALE;
    let text_size = ctx.theme.typography.size_base * 0.95;

    let (icon_width, _) = ctx.text.measure(CHECK_GLYPH, icon_size, ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        CHECK_GLYPH,
        area.x + (area.width - icon_width) / 2.0,
        area.y,
        ALL_CLEAR_ICON_H,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.positive),
    );

    let message = "no hay paquetes por actualizar";
    let (text_width, _) = ctx.text.measure(message, text_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        message,
        area.x + (area.width - text_width) / 2.0,
        area.y + ALL_CLEAR_ICON_H + ALL_CLEAR_GAP,
        ALL_CLEAR_TEXT_H,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn list_height(data: &SystemData) -> f32 {
    if data.updates.packages.is_empty() {
        EMPTY_LIST_H
    } else {
        card_height(data.updates.packages.len().min(VISIBLE_PACKAGES))
    }
}

/// La fila "+N more" solo existe si la lista quedó corta.
fn meta_height(data: &SystemData) -> f32 {
    let pending = data.updates.pending.unwrap_or(0) as usize;

    if pending > VISIBLE_PACKAGES { META_H + SECTION_GAP } else { 0.0 }
}

fn draw_update_button(scene: &mut Scene, rect: Rect, ctx: &mut RenderCtx<'_>) {
    let accent = ctx.theme.palette.accent;
    let hovered = ctx.hovered_interaction == Some(SystemAction::RunUpdate.interaction());

    // Gris con texto accent; el hover lo invierte con texto blanco.
    let (background, foreground) = if hovered {
        (accent, Color::WHITE)
    } else {
        (ctx.theme.palette.control_bg, accent)
    };

    let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, BUTTON_RADIUS);
    scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let label_size = ctx.theme.typography.size_base * 0.95;

    let (icon_width, _) = ctx.text.measure(TERMINAL_GLYPH, icon_size, ctx.theme.typography.icon_font_family);
    let (label_width, _) = ctx.text.measure(UPDATE_COMMAND_LABEL, label_size, ctx.theme.typography.font_family);

    let group_width = icon_width + 8.0 + label_width;
    let group_x = rect.x + (rect.width - group_width) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        TERMINAL_GLYPH,
        group_x,
        rect.y,
        rect.height,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, foreground),
    );

    ctx.text.draw_centered_v(
        scene,
        UPDATE_COMMAND_LABEL,
        group_x + icon_width + 8.0,
        rect.y,
        rect.height,
        TextStyle::new(label_size, ctx.theme.typography.font_family, foreground),
    );
}
