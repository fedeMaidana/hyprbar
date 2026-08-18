// ─── < Imports > ────────────────────────────────────────────────────

use vello::{
    Scene,
    kurbo::{Affine, Circle},
    peniko::Fill,
};

use crate::components::{DropdownFrame, Interaction, Panel, Pill, Point, RenderCtx, truncate_to_width};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::NotificationAction;
use super::state::{MAX_VISIBLE_ROWS, NoteUrgency, NotificationsData, age, unix_now};

// ─── < Constants > ────────────────────────────────────────────────────

const SMALL_TEXT_SCALE: f32 = 0.72;
const ROW_TEXT_SCALE: f32 = 0.9;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct NotificationsPanel<'a> {
    pub(crate) data: &'a NotificationsData,
    /// Scroll discreto, en filas.
    pub(crate) scroll: usize,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl NotificationsPanel<'_> {
    /// Altura con el historial lleno; dimensiona la superficie de la barra.
    pub(crate) fn max_height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.dropdown_padding_y * 2.0 + tokens.notification_header_height + MAX_VISIBLE_ROWS as f32 * tokens.notification_row_height
    }

    fn clear_rect(bounds: Rect, theme: &Theme) -> Rect {
        let tokens = theme.tokens;

        Rect::new(
            bounds.x + bounds.width - tokens.dropdown_padding_x - tokens.notification_clear_width,
            bounds.y + tokens.dropdown_padding_y + (tokens.notification_header_height - tokens.notification_clear_height) / 2.0,
            tokens.notification_clear_width,
            tokens.notification_clear_height,
        )
    }
}

impl Panel for NotificationsPanel<'_> {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        let tokens = theme.tokens;

        let content = if self.data.notes.is_empty() {
            tokens.notification_empty_height
        } else {
            self.data.visible_rows() as f32 * tokens.notification_row_height
        };

        let height = tokens.dropdown_padding_y * 2.0 + tokens.notification_header_height + content;

        DropdownFrame::new(tokens.notification_panel_width, height)
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let tokens = ctx.theme.tokens;
        let pad_x = tokens.dropdown_padding_x;
        let header_height = tokens.notification_header_height;
        let row_height = tokens.notification_row_height;
        let header_y = bounds.y + tokens.dropdown_padding_y;

        let notes = &self.data.notes;

        // Header: título + contador
        let title = format!("Notificaciones · {}", notes.len());
        ctx.text.draw_centered_v(
            scene,
            &title,
            bounds.x + pad_x,
            header_y,
            header_height,
            TextStyle::new(ctx.theme.typography.size_base, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );

        // Botón "limpiar"
        if !notes.is_empty() {
            let clear = Self::clear_rect(bounds, ctx.theme);
            let hovered = ctx.hovered_interaction == Some(NotificationAction::ClearHistory.interaction());

            let background = if hovered {
                ctx.theme.palette.control_hover_bg
            } else {
                ctx.theme.palette.control_bg
            };

            Pill::draw_with_background(scene, clear, ctx.theme, background);

            let label = "limpiar";
            let label_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;

            ctx.text.draw_centered(
                scene,
                label,
                clear,
                TextStyle::new(label_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );
        }

        let divider_y = header_y + header_height;
        DropdownFrame::draw_divider(scene, bounds.x + pad_x, divider_y, bounds.width - pad_x * 2.0, ctx.theme);

        // Historial vacío
        if notes.is_empty() {
            ctx.text.draw_centered_v(
                scene,
                "sin novedades",
                bounds.x + pad_x,
                divider_y,
                tokens.notification_empty_height,
                TextStyle::new(
                    ctx.theme.typography.size_base * ROW_TEXT_SCALE,
                    &ctx.theme.typography.font_family,
                    ctx.theme.palette.text_secondary,
                ),
            );
            return;
        }

        // Filas visibles (scroll discreto por fila)
        let now_unix = unix_now();
        let text_x = bounds.x + pad_x + tokens.notification_row_text_inset;
        let text_w = bounds.width - (text_x - bounds.x) - pad_x;
        let summary_size = ctx.theme.typography.size_base * ROW_TEXT_SCALE;
        let meta_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;

        for (slot, note) in notes.iter().skip(self.scroll).take(MAX_VISIBLE_ROWS).enumerate() {
            let row_y = divider_y + slot as f32 * row_height;

            if note.urgency == NoteUrgency::Critical {
                let dot = Circle::new(
                    ((bounds.x + pad_x + tokens.notification_row_dot_radius) as f64, (row_y + row_height / 2.0) as f64),
                    tokens.notification_row_dot_radius as f64,
                );
                scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);
            }

            let summary = truncate_to_width(ctx.text, &note.summary, summary_size, &ctx.theme.typography.font_family, text_w);
            ctx.text.draw_centered_v(
                scene,
                &summary,
                text_x,
                row_y,
                row_height * 0.55,
                TextStyle::new(summary_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
            );

            let mut meta = age(now_unix.saturating_sub(note.closed_at_unix));
            if !note.app_name.is_empty() {
                meta.push_str(" · ");
                meta.push_str(&note.app_name);
            }
            if !note.body.is_empty() {
                meta.push_str(" — ");
                meta.push_str(note.body.lines().next().unwrap_or(""));
            }

            let meta = truncate_to_width(ctx.text, &meta, meta_size, &ctx.theme.typography.font_family, text_w);
            ctx.text.draw_centered_v(
                scene,
                &meta,
                text_x,
                row_y + row_height * 0.5,
                row_height * 0.45,
                TextStyle::new(meta_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );
        }
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        if self.data.notes.is_empty() {
            return None;
        }

        let clear = Self::clear_rect(bounds, theme);

        clear
            .contains_point(point.x, point.y)
            .then_some(NotificationAction::ClearHistory.interaction())
    }
}
