// ─── < Imports > ────────────────────────────────────────────────────

use vello::{
    Scene,
    kurbo::{Affine, Circle},
    peniko::Fill,
};

use crate::components::{Component, DropdownId, Interaction, InteractionOutcome, Panel, Pill, Point, RenderCtx};
use crate::proc::spawn_detached;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::NotificationAction;
use super::panel::NotificationsPanel;
use super::state::{MAX_VISIBLE_ROWS, NotificationsState};

// ─── < Constants > ────────────────────────────────────────────────────

pub(crate) const NOTIFICATIONS_DROPDOWN: DropdownId = DropdownId::new("notifications");

const BELL_GLYPH: &str = "\u{f009a}";
const BELL_OFF_GLYPH: &str = "\u{f009b}";

// ─── < Structs > ────────────────────────────────────────────────────

pub struct NotificationsPill {
    state: NotificationsState,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl NotificationsPill {
    pub fn new() -> Self {
        Self {
            state: NotificationsState::new(),
        }
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(NOTIFICATIONS_DROPDOWN)
    }
}

impl Default for NotificationsPill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for NotificationsPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
        let (iw, _) = ctx.text.measure(BELL_GLYPH, size, &ctx.theme.typography.icon_font_family);
        let w = iw + ctx.theme.tokens.pill_padding_x * 2.0;
        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        self.state.refresh_state();

        let active = self.is_active(ctx);

        if active {
            self.state.refresh_notes();
        } else if self.state.scroll != 0 {
            self.state.scroll = 0;
        }

        let hovered = ctx.hovered_interaction == Some(Interaction::Dropdown(NOTIFICATIONS_DROPDOWN));
        Pill::draw_with_background(scene, bounds, ctx.theme, Pill::background_for(active, hovered, ctx.theme));

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

        // Con DND la campana se apaga; activa usa el color de slot activo
        let (glyph, color) = match (self.state.state.dnd, active) {
            (_, true) => (BELL_GLYPH, ctx.theme.palette.slot_active_text),
            (true, false) => (BELL_OFF_GLYPH, ctx.theme.palette.text_secondary),
            (false, false) => (BELL_GLYPH, ctx.theme.palette.text_primary),
        };

        ctx.text.draw_centered_v(
            scene,
            glyph,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, color),
        );

        if self.state.state.history_count > 0 && !active {
            let (iw, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);

            let dot_radius = ctx.theme.tokens.notification_dot_radius;
            let dot_cx = bounds.x + pad_x + iw - dot_radius * ctx.theme.tokens.notification_dot_x_overlap_scale;
            let dot_cy = bounds.y + (bounds.height / 2.0) - icon_size * ctx.theme.tokens.notification_dot_y_icon_scale;

            let dot = Circle::new((dot_cx as f64, dot_cy as f64), dot_radius as f64);
            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);
        }
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(NOTIFICATIONS_DROPDOWN))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(NOTIFICATIONS_DROPDOWN)
    }

    fn dropdown_max_height(&self, theme: &Theme) -> f32 {
        NotificationsPanel::max_height(theme)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        NotificationsPanel { state: &self.state }.render(scene, surface, anchor, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(NotificationsPanel { state: &self.state }.bounds(surface, anchor, theme))
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        NotificationsPanel { state: &self.state }.hit_test(point, surface, anchor, theme)
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> Option<InteractionOutcome> {
        let NotificationAction::ClearHistory = NotificationAction::from_interaction(interaction)?;

        if let Err(error) = spawn_detached("hyprnotify", &["history", "clear"]) {
            log::warn!("no se pudo limpiar el historial: {error}");
            return Some(InteractionOutcome::quiet());
        }

        // Optimista: el daemon reescribe los contratos enseguida
        self.state.clear();

        Some(InteractionOutcome::redraw())
    }

    fn handle_scroll(&mut self, delta: f64) -> bool {
        if self.state.notes.len() <= MAX_VISIBLE_ROWS {
            return false;
        }

        let target = if delta > 0.0 {
            (self.state.scroll + 1).min(self.state.max_scroll())
        } else {
            self.state.scroll.saturating_sub(1)
        };

        if target == self.state.scroll {
            return false;
        }

        self.state.scroll = target;
        true
    }

    fn reset_scroll(&mut self) {
        self.state.scroll = 0;
    }
}
