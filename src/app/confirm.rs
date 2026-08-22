// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use smithay_client_toolkit::seat::pointer::CursorIcon;
use smithay_client_toolkit::shell::WaylandSurface;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect, Stroke};
use vello::peniko::{Color, Fill};
use wayland_client::Connection;

use crate::components::{ConfirmRequest, Interaction, Point, RenderCtx};
use crate::render::{Rect, RenderContext, TextStyle};
use crate::theme::Theme;
use crate::wayland::LayerConfig;

use super::state::AppState;
use super::surface::SurfaceState;
use super::surface_handle::SurfaceHandle;

// ─── < Constants > ────────────────────────────────────────────────────

/// Namespace de la capa: el blur de pantalla completa se activa en
/// Hyprland con `layerrule = blur, hyprbar-confirm`.
const CONFIRM_NAMESPACE: &str = "hyprbar-confirm";

/// Velo oscuro sobre la pantalla (el compositor le agrega el blur).
const DIM_ALPHA: f32 = 0.4;

const CONTAINER_WIDTH: f32 = 320.0;
const CONTAINER_RADIUS: f64 = 20.0;
const CONTAINER_PADDING: f32 = 24.0;

const GLYPH_SCALE: f32 = 2.6;
const GLYPH_H: f32 = 44.0;
const TITLE_SCALE: f32 = 1.15;
const TITLE_H: f32 = 26.0;
const SECTION_GAP: f32 = 14.0;

const BUTTON_H: f32 = 40.0;
const BUTTON_GAP: f32 = 10.0;
const BUTTON_RADIUS: f64 = 12.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct ConfirmOverlay {
    pub(crate) request: ConfirmRequest,
    /// Declarado antes que `surface`: al dropear, la superficie wgpu
    /// tiene que morir antes que la wl_surface a la que apunta.
    pub(crate) render_ctx: RenderContext,
    pub(crate) surface: SurfaceState,
    pub(crate) render_stale: bool,
    pub(crate) hovered: Option<ConfirmButton>,
}

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmButton {
    Cancel,
    Confirm,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    /// Abre el overlay modal a pantalla completa para confirmar `request`.
    pub(crate) fn open_confirm(&mut self, request: ConfirmRequest) {
        if self.confirm.is_some() {
            return;
        }

        let config = LayerConfig::fullscreen_overlay();
        let layer = self.wayland.create_layer(&self.qh, &config, CONFIRM_NAMESPACE);
        let (fractional, viewport) = self.wayland.fractional_objects(layer.wl_surface(), &self.qh);

        self.confirm = Some(ConfirmOverlay {
            request,
            render_ctx: RenderContext::new(),
            surface: SurfaceState::new(layer, fractional, viewport),
            render_stale: true,
            hovered: None,
        });

        self.needs_redraw = true;
    }

    /// Cierra el overlay destruyendo sus objetos en el orden correcto.
    pub(crate) fn close_confirm(&mut self) {
        let Some(mut overlay) = self.confirm.take() else {
            return;
        };

        overlay.render_ctx.drop_surface();

        if let Some(viewport) = overlay.surface.viewport.take() {
            viewport.destroy();
        }

        if let Some(fractional) = overlay.surface.fractional.take() {
            fractional.destroy();
        }

        self.needs_redraw = true;
    }

    /// Renderiza el diálogo si el overlay está vivo y configurado.
    pub(crate) fn render_confirm(&mut self) -> Result<()> {
        let Some(overlay) = &mut self.confirm else {
            return Ok(());
        };

        if !overlay.surface.configured || overlay.surface.lost {
            return Ok(());
        }

        if overlay.render_stale {
            let wl_surface = overlay.surface.layer.wl_surface().clone();
            let handle = SurfaceHandle::new(&self.conn, &wl_surface);

            overlay
                .render_ctx
                .create_surface(handle, overlay.surface.physical_width(), overlay.surface.physical_height())?;

            overlay.render_stale = false;
            overlay.surface.pending_resize = false;
        }

        if overlay.surface.pending_resize {
            overlay
                .render_ctx
                .resize(overlay.surface.physical_width(), overlay.surface.physical_height());

            overlay.surface.pending_resize = false;
        }

        // Sin fractional, el buffer escala entero como en la barra.
        if overlay.surface.fractional.is_none() && overlay.surface.applied_buffer_scale != overlay.surface.scale {
            overlay.surface.layer.wl_surface().set_buffer_scale(overlay.surface.scale);
            overlay.surface.applied_buffer_scale = overlay.surface.scale;
        }

        let surface_rect = Rect::new(0.0, 0.0, overlay.surface.width as f32, overlay.surface.height as f32);
        let scale = overlay.surface.effective_scale();

        let mut ctx = RenderCtx {
            theme: &self.theme,
            text: &mut self.text_engine,
            hovered_interaction: None,
            open_dropdown: None,
        };

        if (scale - 1.0).abs() < f64::EPSILON {
            draw_dialog(&mut overlay.render_ctx.scene, surface_rect, &overlay.request, overlay.hovered, &mut ctx);
        } else {
            let mut logical_scene = Scene::new();

            draw_dialog(&mut logical_scene, surface_rect, &overlay.request, overlay.hovered, &mut ctx);

            overlay.render_ctx.scene.append(&logical_scene, Some(Affine::scale(scale)));
        }

        if let Some(viewport) = &overlay.surface.viewport {
            viewport.set_destination(overlay.surface.width.max(1) as i32, overlay.surface.height.max(1) as i32);
        }

        overlay.render_ctx.render()
    }

    /// Hover sobre los botones del diálogo (y cursor acorde).
    pub(crate) fn confirm_pointer_motion(&mut self, conn: &Connection, point: Point, force_cursor_reload: bool) {
        let Some(overlay) = &mut self.confirm else {
            return;
        };

        let surface_rect = Rect::new(0.0, 0.0, overlay.surface.width as f32, overlay.surface.height as f32);
        let hovered = hit_button(point, surface_rect);

        if overlay.hovered != hovered {
            overlay.hovered = hovered;
            self.needs_redraw = true;
        }

        let icon = if hovered.is_some() {
            CursorIcon::Pointer
        } else {
            CursorIcon::Default
        };

        self.set_cursor_icon(conn, icon, force_cursor_reload);
    }

    pub(crate) fn confirm_pointer_leave(&mut self) {
        let Some(overlay) = &mut self.confirm else {
            return;
        };

        if overlay.hovered.take().is_some() {
            self.needs_redraw = true;
        }
    }

    /// Click en el overlay: confirmar despacha la acción, cancelar o
    /// clickear fuera del contenedor solo cierra.
    pub(crate) fn confirm_pointer_press(&mut self, point: Point) {
        let Some(overlay) = &self.confirm else {
            return;
        };

        let surface_rect = Rect::new(0.0, 0.0, overlay.surface.width as f32, overlay.surface.height as f32);

        match hit_button(point, surface_rect) {
            Some(ConfirmButton::Confirm) => {
                let action = overlay.request.action;

                self.close_confirm();

                if let Some(outcome) = self.bar.handle_interaction(Interaction::Action(action)) {
                    self.apply_outcome(outcome);
                }
            }
            Some(ConfirmButton::Cancel) => self.close_confirm(),
            None if !container_rect(surface_rect).contains_point(point.x, point.y) => self.close_confirm(),
            None => {}
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn container_rect(surface: Rect) -> Rect {
    let height = CONTAINER_PADDING + GLYPH_H + SECTION_GAP + TITLE_H + SECTION_GAP + BUTTON_H + CONTAINER_PADDING;

    Rect::new(surface.x + (surface.width - CONTAINER_WIDTH) / 2.0, surface.y + (surface.height - height) / 2.0, CONTAINER_WIDTH, height)
}

fn button_rects(container: Rect) -> [(ConfirmButton, Rect); 2] {
    let y = container.y + container.height - CONTAINER_PADDING - BUTTON_H;
    let width = (container.width - CONTAINER_PADDING * 2.0 - BUTTON_GAP) / 2.0;

    [
        (ConfirmButton::Cancel, Rect::new(container.x + CONTAINER_PADDING, y, width, BUTTON_H)),
        (ConfirmButton::Confirm, Rect::new(container.x + CONTAINER_PADDING + width + BUTTON_GAP, y, width, BUTTON_H)),
    ]
}

fn hit_button(point: Point, surface: Rect) -> Option<ConfirmButton> {
    button_rects(container_rect(surface))
        .into_iter()
        .find(|(_, rect)| rect.contains_point(point.x, point.y))
        .map(|(button, _)| button)
}

fn draw_dialog(scene: &mut Scene, surface: Rect, request: &ConfirmRequest, hovered: Option<ConfirmButton>, ctx: &mut RenderCtx<'_>) {
    // Velo sobre toda la pantalla; el blur lo agrega el compositor.
    let veil = vello::kurbo::Rect::new(
        surface.x as f64,
        surface.y as f64,
        (surface.x + surface.width) as f64,
        (surface.y + surface.height) as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, Color::BLACK.with_alpha(DIM_ALPHA), None, &veil);

    // Contenedor central.
    let container = container_rect(surface);

    let body = RoundedRect::new(
        container.x as f64,
        container.y as f64,
        (container.x + container.width) as f64,
        (container.y + container.height) as f64,
        CONTAINER_RADIUS,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_bg, None, &body);
    scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, ctx.theme.palette.panel_border, None, &body);

    let mut y = container.y + CONTAINER_PADDING;

    // Glifo grande de la acción.
    let glyph_size = ctx.theme.typography.size_base * GLYPH_SCALE;

    let glyph_color = if request.destructive {
        ctx.theme.palette.meter_critical
    } else {
        ctx.theme.palette.accent
    };

    ctx.text.draw_centered(
        scene,
        request.glyph,
        Rect::new(container.x, y, container.width, GLYPH_H),
        TextStyle::new(glyph_size, ctx.theme.typography.icon_font_family, glyph_color),
    );

    y += GLYPH_H + SECTION_GAP;

    // Título.
    let title_size = ctx.theme.typography.size_base * TITLE_SCALE;

    ctx.text.draw_centered(
        scene,
        request.title,
        Rect::new(container.x, y, container.width, TITLE_H),
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    // Botones.
    for (button, rect) in button_rects(container) {
        let is_hovered = hovered == Some(button);
        let (background, foreground) = button_colors(button, request.destructive, is_hovered, ctx.theme);

        let shape =
            RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, BUTTON_RADIUS);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &shape);

        let label = match button {
            ConfirmButton::Cancel => "Cancelar",
            ConfirmButton::Confirm => "Confirmar",
        };

        ctx.text.draw_centered(
            scene,
            label,
            rect,
            TextStyle::new(ctx.theme.typography.size_base, ctx.theme.typography.font_family, foreground),
        );
    }
}

fn button_colors(button: ConfirmButton, destructive: bool, hovered: bool, theme: &Theme) -> (Color, Color) {
    match (button, destructive, hovered) {
        (ConfirmButton::Cancel, _, false) => (theme.palette.control_bg, theme.palette.text_primary),
        (ConfirmButton::Cancel, _, true) => (theme.palette.control_hover_bg, theme.palette.text_primary),
        // El confirmar destructivo replica el botón de apagado del footer.
        (ConfirmButton::Confirm, true, false) => (theme.palette.control_bg, theme.palette.meter_critical),
        (ConfirmButton::Confirm, true, true) => (theme.palette.danger_bg, theme.palette.danger_text),
        (ConfirmButton::Confirm, false, false) => (theme.palette.control_bg, theme.palette.accent),
        (ConfirmButton::Confirm, false, true) => (theme.palette.accent, Color::WHITE),
    }
}
