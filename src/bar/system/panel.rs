// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{DropdownFrame, Interaction, Panel, PanelHeader, Point, RenderCtx, evenly_spaced_rects};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::SystemAction;
use super::metrics::{MemoryInfo, short_kernel_version};
use super::power::PowerAction;
use super::state::{MetricsSnapshot, SystemData};

// ─── < Constants > ────────────────────────────────────────────────────

const PANEL_TITLE: &str = "Arch Linux";
const VALUE_PLACEHOLDER: &str = "—";
const METRIC_ROW_COUNT: f32 = 3.0;
const KIB_PER_GIB: f64 = 1024.0 * 1024.0;

const CPU_GLYPH: &str = "\u{f0ee0}";
const RAM_GLYPH: &str = "\u{f035b}";
const TEMP_GLYPH: &str = "\u{f050f}";
const UPDATES_GLYPH: &str = "\u{f03d7}";

const ICON_TEXT_SCALE: f32 = 0.95;
const HEADER_TITLE_SHARE: f32 = 0.55;

const BADGE_TEXT_SCALE: f32 = 0.82;
const BADGE_PADDING_X: f32 = 7.0;
const BADGE_VERTICAL_INSET: f32 = 3.0;

const UPDATES_CHIP_BG_ALPHA: f32 = 0.16;
const UPDATES_HOVER_PAD_X: f32 = 7.0;
const UPDATES_HOVER_PAD_Y: f32 = 3.0;
const UPDATES_HOVER_RADIUS: f32 = 8.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct SystemPanel<'a> {
    pub data: &'a SystemData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeterSeverity {
    Normal,
    Warning,
    Critical,
}

struct MetricRow {
    icon: &'static str,
    label: &'static str,
    value: String,
    fraction: Option<f32>,
    severity: MeterSeverity,
}

/// A small right-aligned rounded chip (uptime, pending updates).
struct Badge<'a> {
    text: &'a str,
    background: Color,
    foreground: Color,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SystemPanel<'_> {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.dropdown_panel_padding_y * 2.0
            + tokens.dropdown_header_height
            + tokens.dropdown_section_gap * 3.0
            + tokens.system_metric_row_height * METRIC_ROW_COUNT
            + tokens.system_metric_gap * (METRIC_ROW_COUNT - 1.0)
            + tokens.system_updates_row_height
            + tokens.system_button_height
    }
}

impl Panel for SystemPanel<'_> {
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
        y += tokens.dropdown_header_height;

        DropdownFrame::draw_divider(scene, inner_x, y + tokens.dropdown_section_gap / 2.0, inner_width, theme);

        y += tokens.dropdown_section_gap;

        let rows = metric_rows(data.metrics, theme);

        for (index, row) in rows.iter().enumerate() {
            let row_y = y + index as f32 * (tokens.system_metric_row_height + tokens.system_metric_gap);
            draw_metric_row(scene, inner_x, row_y, inner_width, row, ctx);
        }

        y += tokens.system_metric_row_height * METRIC_ROW_COUNT
            + tokens.system_metric_gap * (METRIC_ROW_COUNT - 1.0)
            + tokens.dropdown_section_gap;

        if ctx.hovered_interaction == Some(SystemAction::Updates.interaction()) {
            draw_updates_hover(scene, updates_row_rect(bounds, theme), ctx.theme);
        }

        draw_updates_row(scene, inner_x, y, inner_width, data.pending_updates, ctx);

        let buttons_y = bounds.y + bounds.height - tokens.dropdown_panel_padding_y - tokens.system_button_height;

        DropdownFrame::draw_divider(scene, inner_x, buttons_y - tokens.dropdown_section_gap / 2.0, inner_width, theme);

        draw_power_buttons(scene, bounds, ctx);
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        for (action, rect) in power_button_rects(bounds, theme) {
            if rect.contains_point(point.x, point.y) {
                return Some(SystemAction::Power(action).interaction());
            }
        }

        if updates_row_rect(bounds, theme).contains_point(point.x, point.y) {
            return Some(SystemAction::Updates.interaction());
        }

        None
    }
}

impl Badge<'_> {
    fn draw(&self, scene: &mut Scene, right_x: f32, row_y: f32, row_height: f32, ctx: &mut RenderCtx<'_>) {
        let text_size = ctx.theme.typography.size_base * BADGE_TEXT_SCALE;
        let (text_width, _) = ctx.text.measure(self.text, text_size, ctx.theme.typography.font_family);

        let badge_height = (row_height - BADGE_VERTICAL_INSET * 2.0).max(text_size + 4.0);
        let badge_width = text_width + BADGE_PADDING_X * 2.0;
        let badge_x = right_x - badge_width;
        let badge_y = row_y + (row_height - badge_height) / 2.0;
        let radius = (badge_height / 2.0) as f64;

        let body =
            RoundedRect::new(badge_x as f64, badge_y as f64, (badge_x + badge_width) as f64, (badge_y + badge_height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, self.background, None, &body);

        ctx.text.draw_centered_v(
            scene,
            self.text,
            badge_x + BADGE_PADDING_X,
            badge_y,
            badge_height,
            TextStyle::new(text_size, ctx.theme.typography.font_family, self.foreground),
        );
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_header(scene: &mut Scene, x: f32, y: f32, width: f32, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let subtitle = kernel_label(data);

    PanelHeader {
        title: PANEL_TITLE,
        subtitle: &subtitle,
    }
    .draw(scene, x, y, ctx);

    if let Some(uptime) = uptime_label(data) {
        let badge = Badge {
            text: &uptime,
            background: ctx.theme.palette.panel_raised,
            foreground: ctx.theme.palette.text_secondary,
        };

        badge.draw(scene, x + width, y, ctx.theme.tokens.dropdown_header_height * HEADER_TITLE_SHARE, ctx);
    }
}

fn kernel_label(data: &SystemData) -> String {
    match data.kernel.as_deref() {
        Some(raw) => format!("Kernel {}", short_kernel_version(raw)),
        None => format!("Kernel {VALUE_PLACEHOLDER}"),
    }
}

fn uptime_label(data: &SystemData) -> Option<String> {
    data.metrics
        .and_then(|metrics| metrics.uptime_seconds)
        .map(|seconds| format!("up {}", format_uptime(seconds)))
}

fn metric_rows(metrics: Option<MetricsSnapshot>, theme: &Theme) -> [MetricRow; 3] {
    let cpu = metrics.and_then(|metrics| metrics.cpu_percent);
    let memory = metrics.and_then(|metrics| metrics.memory);
    let temperature = metrics.and_then(|metrics| metrics.temperature_c);

    let cpu_fraction = cpu.map(|value| (value / 100.0).clamp(0.0, 1.0));
    let ram_fraction = memory.map(|memory| memory.used_fraction().clamp(0.0, 1.0));
    let temp_fraction = temperature.map(|value| (value / theme.tokens.system_temp_gauge_max_c).clamp(0.0, 1.0));

    [
        MetricRow {
            icon: CPU_GLYPH,
            label: "CPU",
            value: cpu
                .map(|value| format!("{value:.0}%"))
                .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
            fraction: cpu_fraction,
            severity: cpu_fraction
                .map(|fraction| load_severity(fraction, theme))
                .unwrap_or(MeterSeverity::Normal),
        },
        MetricRow {
            icon: RAM_GLYPH,
            label: "RAM",
            value: memory.map(format_memory).unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
            fraction: ram_fraction,
            severity: ram_fraction
                .map(|fraction| load_severity(fraction, theme))
                .unwrap_or(MeterSeverity::Normal),
        },
        MetricRow {
            icon: TEMP_GLYPH,
            label: "Temp",
            value: temperature
                .map(|value| format!("{value:.0}°C"))
                .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string()),
            fraction: temp_fraction,
            severity: temperature
                .map(|value| temp_severity(value, theme))
                .unwrap_or(MeterSeverity::Normal),
        },
    ]
}

fn load_severity(fraction: f32, theme: &Theme) -> MeterSeverity {
    severity_from(fraction, theme.tokens.system_load_warn_fraction, theme.tokens.system_load_crit_fraction)
}

fn temp_severity(temp_c: f32, theme: &Theme) -> MeterSeverity {
    severity_from(temp_c, theme.tokens.system_temp_warn_c, theme.tokens.system_temp_crit_c)
}

fn severity_from(value: f32, warn: f32, critical: f32) -> MeterSeverity {
    if value >= critical {
        MeterSeverity::Critical
    } else if value >= warn {
        MeterSeverity::Warning
    } else {
        MeterSeverity::Normal
    }
}

fn meter_color(severity: MeterSeverity, theme: &Theme) -> Color {
    match severity {
        MeterSeverity::Normal => theme.palette.accent,
        MeterSeverity::Warning => theme.palette.meter_warning,
        MeterSeverity::Critical => theme.palette.meter_critical,
    }
}

fn draw_metric_row(scene: &mut Scene, x: f32, y: f32, width: f32, row: &MetricRow, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let label_size = ctx.theme.typography.size_base * tokens.dropdown_body_scale;
    let value_size = ctx.theme.typography.size_base * tokens.dropdown_body_scale;
    let text_box_height = tokens.system_metric_row_height - tokens.system_meter_height - tokens.system_meter_gap;

    let label_x = draw_row_icon(scene, x, y, text_box_height, row.icon, ctx);

    ctx.text.draw_centered_v(
        scene,
        row.label,
        label_x,
        y,
        text_box_height,
        TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    let (value_width, _) = ctx.text.measure(&row.value, value_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &row.value,
        x + width - value_width,
        y,
        text_box_height,
        TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let meter_y = y + text_box_height + tokens.system_meter_gap;

    draw_meter(scene, x, meter_y, width, row.fraction, row.severity, ctx.theme);
}

fn draw_row_icon(scene: &mut Scene, x: f32, y: f32, box_height: f32, glyph: &str, ctx: &mut RenderCtx<'_>) -> f32 {
    let icon_size = ctx.theme.typography.size_base * ICON_TEXT_SCALE;

    ctx.text.draw_centered_v(
        scene,
        glyph,
        x,
        y,
        box_height,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.text_secondary),
    );

    // Fixed slot keeps every row label aligned regardless of glyph width.
    x + ctx.theme.tokens.system_icon_slot
}

fn draw_meter(scene: &mut Scene, x: f32, y: f32, width: f32, fraction: Option<f32>, severity: MeterSeverity, theme: &Theme) {
    let height = theme.tokens.system_meter_height;
    let radius = (height / 2.0) as f64;

    let track = RoundedRect::new(x as f64, y as f64, (x + width) as f64, (y + height) as f64, radius);
    scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.panel_raised, None, &track);

    let Some(fraction) = fraction else {
        return;
    };

    let fill_width = (width * fraction).max(height);
    let fill = RoundedRect::new(x as f64, y as f64, (x + fill_width) as f64, (y + height) as f64, radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, meter_color(severity, theme), None, &fill);
}

fn updates_row_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    let inner_x = bounds.x + tokens.dropdown_panel_padding_x;
    let inner_width = bounds.width - tokens.dropdown_panel_padding_x * 2.0;

    let y = bounds.y
        + tokens.dropdown_panel_padding_y
        + tokens.dropdown_header_height
        + tokens.dropdown_section_gap
        + tokens.system_metric_row_height * METRIC_ROW_COUNT
        + tokens.system_metric_gap * (METRIC_ROW_COUNT - 1.0)
        + tokens.dropdown_section_gap;

    Rect::new(
        inner_x - UPDATES_HOVER_PAD_X,
        y - UPDATES_HOVER_PAD_Y,
        inner_width + UPDATES_HOVER_PAD_X * 2.0,
        tokens.system_updates_row_height + UPDATES_HOVER_PAD_Y * 2.0,
    )
}

fn draw_updates_hover(scene: &mut Scene, rect: Rect, theme: &Theme) {
    let body = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.width) as f64,
        (rect.y + rect.height) as f64,
        UPDATES_HOVER_RADIUS as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.control_hover_bg, None, &body);
}

fn draw_updates_row(scene: &mut Scene, x: f32, y: f32, width: f32, pending: Option<u32>, ctx: &mut RenderCtx<'_>) {
    let text_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_body_scale;
    let row_height = ctx.theme.tokens.system_updates_row_height;

    let label_x = draw_row_icon(scene, x, y, row_height, UPDATES_GLYPH, ctx);

    ctx.text.draw_centered_v(
        scene,
        "Updates",
        label_x,
        y,
        row_height,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    if let Some(count) = pending
        && count > 0
    {
        let text = format!("{count} pending");

        let badge = Badge {
            text: &text,
            background: ctx.theme.palette.meter_warning.with_alpha(UPDATES_CHIP_BG_ALPHA),
            foreground: ctx.theme.palette.meter_warning,
        };

        badge.draw(scene, x + width, y, row_height, ctx);

        return;
    }

    let value = match pending {
        Some(0) => "up to date",
        _ => VALUE_PLACEHOLDER,
    };

    let (value_width, _) = ctx.text.measure(value, text_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        value,
        x + width - value_width,
        y,
        row_height,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn draw_power_buttons(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let radius = ctx.theme.tokens.system_button_radius as f64;

    for (action, rect) in power_button_rects(bounds, ctx.theme) {
        let is_hovered = ctx.hovered_interaction == Some(SystemAction::Power(action).interaction());

        let (background, foreground) = button_colors(action, is_hovered, ctx.theme);

        let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let glyph = action.glyph();

        ctx.text
            .draw_centered(scene, glyph, rect, TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, foreground));
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

fn power_button_rects(bounds: Rect, theme: &Theme) -> [(PowerAction, Rect); 3] {
    let tokens = theme.tokens;

    let inner_x = bounds.x + tokens.dropdown_panel_padding_x;
    let inner_width = bounds.width - tokens.dropdown_panel_padding_x * 2.0;
    let y = bounds.y + bounds.height - tokens.dropdown_panel_padding_y - tokens.system_button_height;

    let rects: [Rect; 3] = evenly_spaced_rects(inner_x, y, inner_width, tokens.system_button_height, tokens.system_button_gap);

    std::array::from_fn(|index| (PowerAction::ALL[index], rects[index]))
}

fn format_memory(memory: MemoryInfo) -> String {
    let used = memory.used_kb() as f64 / KIB_PER_GIB;
    let total = memory.total_kb as f64 / KIB_PER_GIB;

    format!("{used:.1} / {total:.0} GiB")
}

fn format_uptime(seconds: u64) -> String {
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
