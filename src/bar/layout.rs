//! Layout de la barra: tres secciones (left/center/right).

use calloop::channel::Sender;
use vello::Scene;

use crate::bar::arch_logo_pill::ArchLogoPill;
use crate::bar::clock_pill::ClockPill;
use crate::bar::command_center_pill::CommandCenterPill;
use crate::bar::date_pill::DatePill;
use crate::bar::notifications_pill::NotificationsPill;
use crate::bar::profile_pill::ProfilePill;
use crate::bar::weather_pill::{WeatherConfig, WeatherPill};
use crate::bar::workspaces_pill::WorkspacesPill;
use crate::components::{Component, RenderCtx};
use crate::render::Rect;
use crate::theme::Theme;

pub struct Bar {
    left: Vec<Box<dyn Component>>,
    center: Vec<Box<dyn Component>>,
    right: Vec<Box<dyn Component>>,
}

impl Bar {
    /// `redraw_signal` lo usan los componentes que tienen estado externo
    /// (workspaces, en el futuro notifications) para forzar un redraw inmediato
    /// cuando llega data nueva, sin esperar al timer global.
    pub fn new(redraw_signal: Sender<()>) -> Self {
        Self {
            left: vec![
                Box::new(ArchLogoPill::new()),
                Box::new(DatePill::new()),
                Box::new(ClockPill::new()),
                Box::new(WeatherPill::new(WeatherConfig::mar_del_plata())),
            ],
            center: vec![
                Box::new(CommandCenterPill::new()),
                Box::new(WorkspacesPill::new(redraw_signal)),
                Box::new(NotificationsPill::new()),
            ],
            right: vec![Box::new(ProfilePill::from_path("assets/profile.jpeg"))],
        }
    }

    pub fn render(
        &mut self,
        scene: &mut Scene,
        surface: Rect,
        theme: &Theme,
        ctx: &mut RenderCtx<'_>,
    ) {
        let pad_x = theme.tokens.bar_margin_x;
        let pad_top = theme.tokens.bar_margin_top;
        let gap = theme.tokens.pill_gap;

        let mut x = surface.x + pad_x;
        for comp in &mut self.left {
            let (w, h) = comp.measure(ctx);
            let bounds = Rect::new(x, surface.y + pad_top, w, h);
            comp.render(scene, bounds, ctx);
            x += w + gap;
        }

        if !self.center.is_empty() {
            let mut widths = Vec::with_capacity(self.center.len());
            let mut total = 0.0;
            for comp in &mut self.center {
                let (w, h) = comp.measure(ctx);
                widths.push((w, h));
                total += w;
            }
            total += gap * (self.center.len() as f32 - 1.0).max(0.0);
            let mut x = surface.x + (surface.width - total) / 2.0;
            for (comp, (w, h)) in self.center.iter_mut().zip(widths.iter().copied()) {
                let bounds = Rect::new(x, surface.y + pad_top, w, h);
                comp.render(scene, bounds, ctx);
                x += w + gap;
            }
        }

        let mut x = surface.x + surface.width - pad_x;
        for comp in self.right.iter_mut().rev() {
            let (w, h) = comp.measure(ctx);
            x -= w;
            let bounds = Rect::new(x, surface.y + pad_top, w, h);
            comp.render(scene, bounds, ctx);
            x -= gap;
        }
    }
}