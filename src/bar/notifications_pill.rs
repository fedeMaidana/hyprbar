// ─── < Imports > ────────────────────────────────────────────────────

use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use vello::{
    Scene,
    kurbo::{Affine, Circle},
    peniko::Fill,
};

use crate::components::{Component, DropdownFrame, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::proc::spawn_detached;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

// ─── < Constants > ────────────────────────────────────────────────────

const BELL_GLYPH: &str = "\u{f009a}";
const BELL_OFF_GLYPH: &str = "\u{f009b}";
const ELLIPSIS: &str = "…";

/// Cada cuánto miramos el contrato de hyprnotify como mucho.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

const PANEL_WIDTH: f32 = 340.0;
const HEADER_HEIGHT: f32 = 38.0;
const ROW_HEIGHT: f32 = 46.0;
const MAX_VISIBLE_ROWS: usize = 8;
const EMPTY_HEIGHT: f32 = 42.0;
const CLEAR_WIDTH: f32 = 66.0;
const CLEAR_HEIGHT: f32 = 22.0;
const DOT_RADIUS: f32 = 3.0;
const SMALL_TEXT_SCALE: f32 = 0.72;
const ROW_TEXT_SCALE: f32 = 0.9;

// ─── < Enums > ────────────────────────────────────────────────────

/// Acciones del panel de notificaciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationAction {
    ClearHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
enum NoteUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

// ─── < Structs > ────────────────────────────────────────────────────

/// El contrato `state.json` que escribe hyprnotify en su cache.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct HyprnotifyState {
    #[serde(default)]
    dnd: bool,
    #[serde(default)]
    history_count: usize,
}

/// Una entrada de `history.json` (contrato de hyprnotify).
#[derive(Debug, Clone, Default, Deserialize)]
struct HistoryNote {
    #[serde(default)]
    app_name: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    urgency: NoteUrgency,
    #[serde(default)]
    closed_at_unix: u64,
}

pub struct NotificationsPill {
    state: HyprnotifyState,
    state_checked: Option<Instant>,
    state_modified: Option<SystemTime>,
    /// Historial de más nueva a más vieja, cargado con el panel abierto.
    notes: Vec<HistoryNote>,
    notes_modified: Option<SystemTime>,
    /// Scroll discreto, en filas.
    scroll: usize,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl NotificationsPill {
    pub fn new() -> Self {
        Self {
            state: HyprnotifyState::default(),
            state_checked: None,
            state_modified: None,
            notes: Vec::new(),
            notes_modified: None,
            scroll: 0,
        }
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::NOTIFICATIONS)
    }

    /// Relee `state.json` solo si pasó el intervalo y cambió el mtime.
    fn refresh_state(&mut self) {
        let now = Instant::now();

        if self.state_checked.is_some_and(|checked| now - checked < POLL_INTERVAL) {
            return;
        }

        self.state_checked = Some(now);

        let path = cache_path("state.json");
        let modified = fs::metadata(&path).ok().and_then(|meta| meta.modified().ok());

        if modified == self.state_modified {
            return;
        }

        self.state_modified = modified;

        self.state = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();
    }

    /// Relee `history.json` cuando cambia; solo corre con el panel abierto.
    fn refresh_notes(&mut self) {
        let path = cache_path("history.json");
        let modified = fs::metadata(&path).ok().and_then(|meta| meta.modified().ok());

        if modified == self.notes_modified {
            return;
        }

        self.notes_modified = modified;

        let mut notes: Vec<HistoryNote> = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();

        notes.reverse(); // el archivo va de vieja a nueva

        self.notes = notes;
        self.scroll = self.scroll.min(self.max_scroll());
    }

    fn visible_rows(&self) -> usize {
        self.notes.len().min(MAX_VISIBLE_ROWS)
    }

    fn max_scroll(&self) -> usize {
        self.notes.len().saturating_sub(MAX_VISIBLE_ROWS)
    }

    fn frame(&self, theme: &Theme) -> DropdownFrame {
        let content = if self.notes.is_empty() {
            EMPTY_HEIGHT
        } else {
            self.visible_rows() as f32 * ROW_HEIGHT
        };

        let height = theme.tokens.dropdown_padding_y * 2.0 + HEADER_HEIGHT + content;

        DropdownFrame::new(PANEL_WIDTH, height)
    }

    fn clear_rect(bounds: Rect, theme: &Theme) -> Rect {
        Rect::new(
            bounds.x + bounds.width - theme.tokens.dropdown_padding_x - CLEAR_WIDTH,
            bounds.y + theme.tokens.dropdown_padding_y + (HEADER_HEIGHT - CLEAR_HEIGHT) / 2.0,
            CLEAR_WIDTH,
            CLEAR_HEIGHT,
        )
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
        self.refresh_state();

        let active = self.is_active(ctx);

        if active {
            self.refresh_notes();
        } else if self.scroll != 0 {
            self.scroll = 0;
        }

        let hovered = ctx.hovered_interaction == Some(Interaction::Dropdown(DropdownId::NOTIFICATIONS));
        Pill::draw_with_background(scene, bounds, ctx.theme, Pill::background_for(active, hovered, ctx.theme));

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

        // Con DND la campana se apaga; activa usa el color de slot activo
        let (glyph, color) = match (self.state.dnd, active) {
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

        if self.state.history_count > 0 && !active {
            let (iw, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);

            let dot_radius = ctx.theme.tokens.notification_dot_radius;
            let dot_cx = bounds.x + pad_x + iw - dot_radius * ctx.theme.tokens.notification_dot_x_overlap_scale;
            let dot_cy = bounds.y + (bounds.height / 2.0) - icon_size * ctx.theme.tokens.notification_dot_y_icon_scale;

            let dot = Circle::new((dot_cx as f64, dot_cy as f64), dot_radius as f64);
            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);
        }
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(DropdownId::NOTIFICATIONS))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::NOTIFICATIONS)
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(self.frame(theme).bounds(surface, anchor, theme))
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let frame = self.frame(ctx.theme);
        let bounds = frame.bounds(surface, anchor, ctx.theme);

        frame.draw_background(scene, bounds, ctx.theme);

        let tokens = ctx.theme.tokens;
        let pad_x = tokens.dropdown_padding_x;
        let header_y = bounds.y + tokens.dropdown_padding_y;

        // Header: título + contador
        let title = format!("Notificaciones · {}", self.notes.len());
        ctx.text.draw_centered_v(
            scene,
            &title,
            bounds.x + pad_x,
            header_y,
            HEADER_HEIGHT,
            TextStyle::new(ctx.theme.typography.size_base, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );

        // Botón "limpiar"
        if !self.notes.is_empty() {
            let clear = Self::clear_rect(bounds, ctx.theme);
            let hovered = ctx.hovered_interaction == Some(Interaction::Notifications(NotificationAction::ClearHistory));

            let background = if hovered {
                ctx.theme.palette.control_hover_bg
            } else {
                ctx.theme.palette.control_bg
            };

            Pill::draw_with_background(scene, clear, ctx.theme, background);

            let label = "limpiar";
            let label_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;
            let (lw, _) = ctx.text.measure(label, label_size, &ctx.theme.typography.font_family);

            ctx.text.draw_centered_v(
                scene,
                label,
                clear.x + (clear.width - lw).max(0.0) / 2.0,
                clear.y,
                clear.height,
                TextStyle::new(label_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );
        }

        let divider_y = header_y + HEADER_HEIGHT;
        DropdownFrame::draw_divider(scene, bounds.x + pad_x, divider_y, bounds.width - pad_x * 2.0, ctx.theme);

        // Historial vacío
        if self.notes.is_empty() {
            ctx.text.draw_centered_v(
                scene,
                "sin novedades",
                bounds.x + pad_x,
                divider_y,
                EMPTY_HEIGHT,
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
        let text_x = bounds.x + pad_x + 12.0;
        let text_w = bounds.width - (text_x - bounds.x) - pad_x;
        let summary_size = ctx.theme.typography.size_base * ROW_TEXT_SCALE;
        let meta_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;

        for (slot, note) in self.notes.iter().skip(self.scroll).take(MAX_VISIBLE_ROWS).enumerate() {
            let row_y = divider_y + slot as f32 * ROW_HEIGHT;

            if note.urgency == NoteUrgency::Critical {
                let dot = Circle::new(((bounds.x + pad_x + 3.0) as f64, (row_y + ROW_HEIGHT / 2.0) as f64), DOT_RADIUS as f64);
                scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);
            }

            let summary = fit_text(ctx, &note.summary, summary_size, text_w);
            ctx.text.draw_centered_v(
                scene,
                &summary,
                text_x,
                row_y,
                ROW_HEIGHT * 0.55,
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

            let meta = fit_text(ctx, &meta, meta_size, text_w);
            ctx.text.draw_centered_v(
                scene,
                &meta,
                text_x,
                row_y + ROW_HEIGHT * 0.5,
                ROW_HEIGHT * 0.45,
                TextStyle::new(meta_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );
        }
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        if self.notes.is_empty() {
            return None;
        }

        let bounds = self.frame(theme).bounds(surface, anchor, theme);
        let clear = Self::clear_rect(bounds, theme);

        contains(clear, point).then_some(Interaction::Notifications(NotificationAction::ClearHistory))
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> bool {
        let Interaction::Notifications(NotificationAction::ClearHistory) = interaction else {
            return false;
        };

        if let Err(error) = spawn_detached("hyprnotify", &["history", "clear"]) {
            log::warn!("no se pudo limpiar el historial: {error}");
            return false;
        }

        // Optimista: el daemon reescribe los contratos enseguida
        self.notes.clear();
        self.state.history_count = 0;
        self.scroll = 0;

        true
    }

    fn handle_scroll(&mut self, delta: f64) -> bool {
        if self.notes.len() <= MAX_VISIBLE_ROWS {
            return false;
        }

        let target = if delta > 0.0 {
            (self.scroll + 1).min(self.max_scroll())
        } else {
            self.scroll.saturating_sub(1)
        };

        if target == self.scroll {
            return false;
        }

        self.scroll = target;
        true
    }

    fn reset_scroll(&mut self) {
        self.scroll = 0;
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn contains(rect: Rect, point: Point) -> bool {
    point.x >= rect.x && point.x <= rect.x + rect.width && point.y >= rect.y && point.y <= rect.y + rect.height
}

/// Trunca con "…" para que entre en `max_width`.
fn fit_text(ctx: &mut RenderCtx<'_>, text: &str, size: f32, max_width: f32) -> String {
    let family = ctx.theme.typography.font_family.clone();
    let (full, _) = ctx.text.measure(text, size, &family);

    if full <= max_width {
        return text.to_owned();
    }

    let mut fitted = String::new();

    for ch in text.chars() {
        let candidate = format!("{fitted}{ch}{ELLIPSIS}");

        if ctx.text.measure(&candidate, size, &family).0 > max_width {
            break;
        }

        fitted.push(ch);
    }

    fitted.push_str(ELLIPSIS);
    fitted
}

/// Edad compacta: "ahora", "5m", "3h", "2d".
fn age(seconds: u64) -> String {
    match seconds {
        0..60 => "ahora".to_owned(),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn cache_path(file: &str) -> PathBuf {
    let base = if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home)
    } else {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cache")
    };

    base.join("hyprnotify").join(file)
}
