// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{DropdownFrame, Interaction, Panel, Point, RenderCtx, evenly_spaced_rects};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::{SystemAction, SystemTab};
use super::power::PowerAction;
use super::state::SystemData;
use super::{view_network, view_power, view_system, view_updates};

// ─── < Constants > ────────────────────────────────────────────────────

const ARCH_GLYPH: &str = "\u{f08c7}";
const VALUE_PLACEHOLDER: &str = "—";

const PAD: f32 = 14.0;

const HEADER_H: f32 = 40.0;
const HEADER_GAP: f32 = 10.0;
const TILE_RADIUS: f64 = 12.0;
const TILE_TEXT_GAP: f32 = 12.0;
const TITLE_SCALE: f32 = 1.3;
const SUBTITLE_SCALE: f32 = 0.8;

const BADGE_H: f32 = 20.0;
const BADGE_PADDING_X: f32 = 9.0;
const BADGE_DOT_RADIUS: f32 = 2.5;
const BADGE_TEXT_SCALE: f32 = 0.72;

const TAB_H: f32 = 30.0;
const TAB_INSET: f32 = 3.0;
/// Aire entre los segmentos de la barra de tabs.
const TAB_SEGMENT_GAP: f32 = 6.0;
const TAB_BAR_RADIUS: f64 = 10.0;
const TAB_RADIUS: f64 = 8.0;
const TAB_TEXT_SCALE: f32 = 0.85;
const TAB_DOT_RADIUS: f32 = 2.5;
const TAB_GAP: f32 = 12.0;

const FOOTER_GAP: f32 = 10.0;
const FOOTER_H: f32 = 40.0;
const FOOTER_BUTTON_GAP: f32 = 10.0;
const FOOTER_BUTTON_RADIUS: f64 = 12.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct SystemPanel<'a> {
    pub data: &'a SystemData,
    pub active_tab: SystemTab,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SystemPanel<'_> {
    /// Altura con la pestaña más alta; dimensiona la superficie de la barra.
    pub fn max_height(theme: &Theme) -> f32 {
        let tallest = view_system::max_height(theme)
            .max(view_network::max_height(theme))
            .max(view_power::max_height(theme))
            .max(view_updates::max_height(theme));

        shell_overhead() + tallest
    }

    fn height(&self, theme: &Theme) -> f32 {
        shell_overhead() + self.view_height(theme)
    }

    /// La pestaña activa, con fallback a System si Power quedó oculta
    /// (por ejemplo, porque no hay batería).
    fn effective_tab(&self) -> SystemTab {
        if self.active_tab == SystemTab::Power && self.data.battery.is_none() {
            SystemTab::System
        } else {
            self.active_tab
        }
    }

    fn view_height(&self, theme: &Theme) -> f32 {
        match self.effective_tab() {
            SystemTab::System => view_system::height(self.data, theme),
            SystemTab::Network => view_network::height(self.data, theme),
            SystemTab::Power => view_power::height(self.data, theme),
            SystemTab::Updates => view_updates::height(self.data, theme),
        }
    }

    fn view_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        Rect::new(
            bounds.x + PAD,
            bounds.y + PAD + HEADER_H + HEADER_GAP + TAB_H + TAB_GAP,
            bounds.width - PAD * 2.0,
            self.view_height(theme),
        )
    }
}

impl Panel for SystemPanel<'_> {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.system_panel_width, self.height(theme))
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let inner_x = bounds.x + PAD;
        let inner_width = bounds.width - PAD * 2.0;

        draw_header(scene, inner_x, bounds.y + PAD, inner_width, self.data, ctx);

        let tab_bar = Rect::new(inner_x, bounds.y + PAD + HEADER_H + HEADER_GAP, inner_width, TAB_H);
        draw_tab_bar(scene, tab_bar, self.effective_tab(), self.data, ctx);

        let area = self.view_area(bounds, ctx.theme);

        match self.effective_tab() {
            SystemTab::System => view_system::draw(scene, area, self.data, ctx),
            SystemTab::Network => view_network::draw(scene, area, self.data, ctx),
            SystemTab::Power => view_power::draw(scene, area, self.data, ctx),
            SystemTab::Updates => view_updates::draw(scene, area, self.data, ctx),
        }

        let footer_y = bounds.y + bounds.height - PAD - FOOTER_H;

        DropdownFrame::draw_divider(scene, inner_x, footer_y - FOOTER_GAP, inner_width, ctx.theme);
        draw_footer(scene, bounds, ctx);
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        let tab_bar = Rect::new(bounds.x + PAD, bounds.y + PAD + HEADER_H + HEADER_GAP, bounds.width - PAD * 2.0, TAB_H);

        for (tab, rect) in tab_segment_rects(tab_bar, visible_tabs(self.data)) {
            if rect.contains_point(point.x, point.y) {
                return Some(SystemAction::SelectTab(tab).interaction());
            }
        }

        for (action, rect) in footer_button_rects(bounds) {
            if rect.contains_point(point.x, point.y) {
                return Some(SystemAction::Power(action).interaction());
            }
        }

        let area = self.view_area(bounds, theme);

        match self.effective_tab() {
            SystemTab::Network => view_network::hit_test(point, area, self.data, theme),
            SystemTab::Updates => view_updates::hit_test(point, area, self.data, theme),
            SystemTab::System | SystemTab::Power => None,
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn shell_overhead() -> f32 {
    PAD + HEADER_H + HEADER_GAP + TAB_H + TAB_GAP + FOOTER_GAP * 2.0 + FOOTER_H + PAD
}

fn draw_header(scene: &mut Scene, x: f32, y: f32, width: f32, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    // Tile con el logo.
    let tile = RoundedRect::new(x as f64, y as f64, (x + HEADER_H) as f64, (y + HEADER_H) as f64, TILE_RADIUS);
    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_raised, None, &tile);

    let logo_size = ctx.theme.typography.size_base * 1.9;

    ctx.text.draw_centered(
        scene,
        ARCH_GLYPH,
        Rect::new(x, y, HEADER_H, HEADER_H),
        TextStyle::new(logo_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.accent),
    );

    // Título y subtítulo.
    let text_x = x + HEADER_H + TILE_TEXT_GAP;
    let title_size = ctx.theme.typography.size_base * TITLE_SCALE;
    let subtitle_size = ctx.theme.typography.size_base * SUBTITLE_SCALE;

    ctx.text.draw_centered_v(
        scene,
        "Arch Linux",
        text_x,
        y + 2.0,
        HEADER_H * 0.55,
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let kernel = data
        .kernel
        .as_deref()
        .map(super::metrics::short_kernel_version)
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    let subtitle = format!("Kernel {kernel} · {}", std::env::consts::ARCH);

    ctx.text.draw_centered_v(
        scene,
        &subtitle,
        text_x,
        y + HEADER_H * 0.52,
        HEADER_H * 0.45,
        TextStyle::new(subtitle_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    // Badge de uptime con puntito verde.
    let Some(uptime) = data.metrics.and_then(|metrics| metrics.uptime_seconds) else {
        return;
    };

    let label = format!("up {}", short_uptime(uptime));
    let text_size = ctx.theme.typography.size_base * BADGE_TEXT_SCALE;
    let (label_width, _) = ctx.text.measure(&label, text_size, ctx.theme.typography.font_family);

    let dot_slot = BADGE_DOT_RADIUS * 2.0 + 6.0;
    let badge_width = BADGE_PADDING_X * 2.0 + dot_slot + label_width;
    let badge_x = x + width - badge_width;
    let badge_y = y + (HEADER_H * 0.55 - BADGE_H) / 2.0;

    let badge = RoundedRect::new(
        badge_x as f64,
        badge_y as f64,
        (badge_x + badge_width) as f64,
        (badge_y + BADGE_H) as f64,
        (BADGE_H / 2.0) as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.control_bg, None, &badge);

    let dot =
        Circle::new(((badge_x + BADGE_PADDING_X + BADGE_DOT_RADIUS) as f64, (badge_y + BADGE_H / 2.0) as f64), BADGE_DOT_RADIUS as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.positive, None, &dot);

    ctx.text.draw_centered_v(
        scene,
        &label,
        badge_x + BADGE_PADDING_X + dot_slot,
        badge_y,
        BADGE_H,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

/// Sin batería no hay nada que mostrar en Power: la pestaña se oculta.
/// Los botones de apagado del footer no dependen de esto.
fn visible_tabs(data: &SystemData) -> &'static [SystemTab] {
    if data.battery.is_some() {
        &SystemTab::ALL
    } else {
        &[SystemTab::System, SystemTab::Network, SystemTab::Updates]
    }
}

fn tab_segment_rects(bar: Rect, tabs: &[SystemTab]) -> impl Iterator<Item = (SystemTab, Rect)> + '_ {
    let inner = Rect::new(bar.x + TAB_INSET, bar.y + TAB_INSET, bar.width - TAB_INSET * 2.0, bar.height - TAB_INSET * 2.0);
    let count = tabs.len() as f32;
    let segment = (inner.width - TAB_SEGMENT_GAP * (count - 1.0)) / count;

    tabs.iter().enumerate().map(move |(index, &tab)| {
        let rect = Rect::new(inner.x + index as f32 * (segment + TAB_SEGMENT_GAP), inner.y, segment, inner.height);

        (tab, rect)
    })
}

fn draw_tab_bar(scene: &mut Scene, bar: Rect, active: SystemTab, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let container = RoundedRect::new(bar.x as f64, bar.y as f64, (bar.x + bar.width) as f64, (bar.y + bar.height) as f64, TAB_BAR_RADIUS);
    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_inset, None, &container);

    let text_size = ctx.theme.typography.size_base * TAB_TEXT_SCALE;

    for (tab, rect) in tab_segment_rects(bar, visible_tabs(data)) {
        let is_active = tab == active;
        let hovered = ctx.hovered_interaction == Some(SystemAction::SelectTab(tab).interaction());

        if is_active || hovered {
            let background = if is_active {
                ctx.theme.palette.pill_hover_bg
            } else {
                ctx.theme.palette.panel_raised
            };

            let segment =
                RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, TAB_RADIUS);

            scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &segment);
        }

        let color = if is_active {
            ctx.theme.palette.text_primary
        } else {
            ctx.theme.palette.text_secondary
        };

        let (label_width, _) = ctx.text.measure(tab.label(), text_size, ctx.theme.typography.font_family);
        let has_dot = tab == SystemTab::Updates && data.updates.pending.is_some_and(|count| count > 0);
        let dot_slot = if has_dot { TAB_DOT_RADIUS * 2.0 + 5.0 } else { 0.0 };
        let label_x = rect.x + (rect.width - label_width - dot_slot) / 2.0;

        ctx.text.draw_centered_v(
            scene,
            tab.label(),
            label_x,
            rect.y,
            rect.height,
            TextStyle::new(text_size, ctx.theme.typography.font_family, color),
        );

        if has_dot {
            let dot = Circle::new(
                ((label_x + label_width + 5.0 + TAB_DOT_RADIUS) as f64, (rect.y + rect.height / 2.0) as f64),
                TAB_DOT_RADIUS as f64,
            );

            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);
        }
    }
}

fn footer_button_rects(bounds: Rect) -> [(PowerAction, Rect); 4] {
    let inner_x = bounds.x + PAD;
    let inner_width = bounds.width - PAD * 2.0;
    let y = bounds.y + bounds.height - PAD - FOOTER_H;

    let rects: [Rect; 4] = evenly_spaced_rects(inner_x, y, inner_width, FOOTER_H, FOOTER_BUTTON_GAP);

    std::array::from_fn(|index| (PowerAction::ALL[index], rects[index]))
}

fn draw_footer(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

    for (action, rect) in footer_button_rects(bounds) {
        let is_hovered = ctx.hovered_interaction == Some(SystemAction::Power(action).interaction());
        let (background, foreground) = button_colors(action, is_hovered, ctx.theme);

        let body = RoundedRect::new(
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            (rect.y + rect.height) as f64,
            FOOTER_BUTTON_RADIUS,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        ctx.text
            .draw_centered(scene, action.glyph(), rect, TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, foreground));
    }
}

fn button_colors(action: PowerAction, hovered: bool, theme: &Theme) -> (Color, Color) {
    match (action, hovered) {
        // The destructive action reads as such before hovering.
        (PowerAction::Shutdown, false) => (theme.palette.control_bg, theme.palette.meter_critical),
        (PowerAction::Shutdown, true) => (theme.palette.danger_bg, theme.palette.danger_text),
        (_, false) => (theme.palette.control_bg, theme.palette.text_primary),
        (_, true) => (theme.palette.control_hover_bg, theme.palette.text_primary),
    }
}

/// "32m", "3h 12m", "2d 4h".
fn short_uptime(seconds: u64) -> String {
    let minutes = (seconds / 60) % 60;
    let hours = (seconds / 3600) % 24;
    let days = seconds / 86400;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
