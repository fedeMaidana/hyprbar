// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, Interaction, Panel, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::{ClockAction, ClockTab};
use super::state::{AlarmEditor, ClockData};
use super::{view_alarms, view_clock, view_stopwatch, view_timer};

// ─── < Constants > ────────────────────────────────────────────────────

const PAD: f32 = 14.0;

const TAB_H: f32 = 30.0;
const TAB_INSET: f32 = 3.0;
const TAB_SEGMENT_GAP: f32 = 6.0;
const TAB_BAR_RADIUS: f64 = 10.0;
const TAB_RADIUS: f64 = 8.0;
const TAB_TEXT_SCALE: f32 = 0.78;
const TAB_GAP: f32 = 14.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ClockPanel<'a> {
    pub data: &'a ClockData,
    pub active_tab: ClockTab,
    pub editor: Option<&'a AlarmEditor>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ClockPanel<'_> {
    /// Altura con la pestaña más alta; dimensiona la superficie de la barra.
    pub fn max_height(theme: &Theme) -> f32 {
        let tallest = view_clock::height(theme)
            .max(view_alarms::max_height(theme))
            .max(view_stopwatch::max_height(theme))
            .max(view_timer::height(theme));

        shell_overhead() + tallest
    }

    fn height(&self, theme: &Theme) -> f32 {
        shell_overhead() + self.view_height(theme)
    }

    fn view_height(&self, theme: &Theme) -> f32 {
        match self.active_tab {
            ClockTab::Clock => view_clock::height(theme),
            ClockTab::Alarms => view_alarms::height(&self.data.alarms, self.editor, theme),
            ClockTab::Stopwatch => view_stopwatch::height(&self.data.stopwatch, theme),
            ClockTab::Timer => view_timer::height(theme),
        }
    }

    fn view_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        Rect::new(bounds.x + PAD, bounds.y + PAD + TAB_H + TAB_GAP, bounds.width - PAD * 2.0, self.view_height(theme))
    }
}

impl Panel for ClockPanel<'_> {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.clock_panel_width, self.height(theme))
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let tab_bar = Rect::new(bounds.x + PAD, bounds.y + PAD, bounds.width - PAD * 2.0, TAB_H);
        draw_tab_bar(scene, tab_bar, self.active_tab, ctx);

        let area = self.view_area(bounds, ctx.theme);

        match self.active_tab {
            ClockTab::Clock => view_clock::draw(scene, area, ctx),
            ClockTab::Alarms => view_alarms::draw(scene, area, &self.data.alarms, self.editor, ctx),
            ClockTab::Stopwatch => view_stopwatch::draw(scene, area, &self.data.stopwatch, ctx),
            ClockTab::Timer => view_timer::draw(scene, area, &self.data.timer, ctx),
        }
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        let tab_bar = Rect::new(bounds.x + PAD, bounds.y + PAD, bounds.width - PAD * 2.0, TAB_H);

        for (tab, rect) in tab_segment_rects(tab_bar) {
            if rect.contains_point(point.x, point.y) {
                return Some(ClockAction::SelectTab(tab).interaction());
            }
        }

        let area = self.view_area(bounds, theme);

        match self.active_tab {
            ClockTab::Clock => None,
            ClockTab::Alarms => view_alarms::hit_test(point, area, &self.data.alarms, self.editor, theme),
            ClockTab::Stopwatch => view_stopwatch::hit_test(point, area, theme),
            ClockTab::Timer => view_timer::hit_test(point, area, theme),
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn shell_overhead() -> f32 {
    PAD + TAB_H + TAB_GAP + PAD
}

fn tab_segment_rects(bar: Rect) -> [(ClockTab, Rect); 4] {
    let inner = Rect::new(bar.x + TAB_INSET, bar.y + TAB_INSET, bar.width - TAB_INSET * 2.0, bar.height - TAB_INSET * 2.0);
    let count = ClockTab::ALL.len() as f32;
    let segment = (inner.width - TAB_SEGMENT_GAP * (count - 1.0)) / count;

    std::array::from_fn(|index| {
        let tab = ClockTab::ALL[index];
        let rect = Rect::new(inner.x + index as f32 * (segment + TAB_SEGMENT_GAP), inner.y, segment, inner.height);

        (tab, rect)
    })
}

fn draw_tab_bar(scene: &mut Scene, bar: Rect, active: ClockTab, ctx: &mut RenderCtx<'_>) {
    let container = RoundedRect::new(bar.x as f64, bar.y as f64, (bar.x + bar.width) as f64, (bar.y + bar.height) as f64, TAB_BAR_RADIUS);
    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_inset, None, &container);

    let text_size = ctx.theme.typography.size_base * TAB_TEXT_SCALE;

    for (tab, rect) in tab_segment_rects(bar) {
        let is_active = tab == active;
        let hovered = ctx.hovered_interaction == Some(ClockAction::SelectTab(tab).interaction());

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

        ctx.text
            .draw_centered(scene, tab.label(), rect, TextStyle::new(text_size, ctx.theme.typography.font_family, color));
    }
}
