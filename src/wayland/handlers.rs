// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::compositor::CompositorHandler;
use smithay_client_toolkit::delegate_compositor;
use smithay_client_toolkit::delegate_layer;
use smithay_client_toolkit::delegate_output;
use smithay_client_toolkit::delegate_pointer;
use smithay_client_toolkit::delegate_registry;
use smithay_client_toolkit::delegate_seat;
use smithay_client_toolkit::delegate_shm;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::registry_handlers;
use smithay_client_toolkit::seat::Capability;
use smithay_client_toolkit::seat::SeatHandler;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::seat::pointer::{BTN_LEFT, CursorIcon, PointerEvent, PointerEventKind, PointerHandler, ThemeSpec};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure};
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::protocol::{wl_output, wl_pointer, wl_seat, wl_surface};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::{self, WpFractionalScaleManagerV1};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::{self, WpFractionalScaleV1};
use wayland_protocols::wp::viewporter::client::wp_viewport::{self, WpViewport};
use wayland_protocols::wp::viewporter::client::wp_viewporter::{self, WpViewporter};

use crate::app::AppState;
use crate::components::Point;

// ─── < Implementations > ────────────────────────────────────────────────────

impl CompositorHandler for AppState {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, surface: &wl_surface::WlSurface, new_factor: i32) {
        if surface != self.layer_surface().wl_surface() {
            return;
        }

        // With fractional scale + viewport the buffer stays at scale 1; the integer factor is irrelevant.
        if self.surface.fractional.is_some() {
            return;
        }

        let new_factor = new_factor.max(1);

        if new_factor == self.surface.scale {
            return;
        }

        log::info!("buffer scale: {} -> {new_factor}", self.surface.scale);

        self.surface.scale = new_factor;
        self.surface.pending_resize = true;
        self.needs_redraw = true;
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {}

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.wayland.output_state
    }

    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        log::warn!("superficie cerrada por el compositor (¿output apagado?); se recreará");
        self.surface.lost = true;
        self.surface.configured = false;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;

        if w > 0 && h > 0 && (w != self.surface.width || h != self.surface.height) {
            self.surface.width = w;
            self.surface.height = h;
            self.surface.pending_resize = true;
        }

        self.surface.configured = true;
        self.needs_redraw = true;
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.wayland.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}

    fn new_capability(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Pointer && self.pointer.themed_pointer.is_none() {
            let surface = self.create_cursor_surface(qh);

            match self
                .wayland
                .seat_state
                .get_pointer_with_theme(qh, &seat, self.wayland.shm_state.wl_shm(), surface, ThemeSpec::default())
            {
                Ok(pointer) => {
                    self.pointer.themed_pointer = Some(pointer);
                    log::info!("capacidad de puntero habilitada");
                }
                Err(error) => {
                    log::warn!("no se pudo crear el puntero con theme: {error}");
                }
            }
        }
    }

    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Pointer {
            if let Some(pointer) = self.pointer.themed_pointer.take() {
                pointer.pointer().release();
            }

            self.pointer.position = None;
            self.pointer.icon = CursorIcon::Default;

            log::info!("capacidad de puntero removida");
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
}

impl PointerHandler for AppState {
    fn pointer_frame(&mut self, conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            if &event.surface != self.layer_surface().wl_surface() {
                continue;
            }

            let (x, y) = event.position;
            let point = Point::new(x as f32, y as f32);

            match event.kind {
                PointerEventKind::Enter { .. } => {
                    self.handle_pointer_enter_or_motion(conn, point, true);
                }
                PointerEventKind::Motion { .. } => {
                    self.handle_pointer_enter_or_motion(conn, point, false);
                }
                PointerEventKind::Leave { .. } => {
                    self.handle_pointer_leave(conn);
                }
                PointerEventKind::Press { button, .. } => {
                    if button == BTN_LEFT {
                        self.pointer.position = Some(point);
                        self.handle_pointer_press();
                    }
                }
                PointerEventKind::Release { button, .. } => {
                    if button == BTN_LEFT {
                        self.handle_pointer_release();
                    }
                }
                PointerEventKind::Axis { vertical, .. } => {
                    self.pointer.position = Some(point);
                    self.handle_pointer_scroll(vertical.absolute);
                }
            }
        }
    }
}

impl ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.wayland.shm_state
    }
}

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.wayland.registry_state
    }

    registry_handlers![OutputState, SeatState];
}

// ─── < Fractional Scale + Viewport > ────────────────────────────────────────

impl Dispatch<WpFractionalScaleManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WpFractionalScaleManagerV1,
        _event: wp_fractional_scale_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpFractionalScaleV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Ignore stragglers from an already-destroyed object (surface recreated).
        if state.surface.fractional.as_ref() != Some(proxy) {
            return;
        }

        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event
            && state.surface.scale120 != Some(scale)
        {
            log::info!("fractional scale: {:?} -> {scale}/120", state.surface.scale120);

            state.surface.scale120 = Some(scale);
            state.surface.pending_resize = true;
            state.needs_redraw = true;
        }
    }
}

impl Dispatch<WpViewporter, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WpViewporter,
        _event: wp_viewporter::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpViewport, ()> for AppState {
    fn event(_state: &mut Self, _proxy: &WpViewport, _event: wp_viewport::Event, _data: &(), _conn: &Connection, _qh: &QueueHandle<Self>) {}
}

// ─── < SCTK Delegates > ────────────────────────────────────────────────────

delegate_compositor!(AppState);
delegate_output!(AppState);
delegate_layer!(AppState);
delegate_seat!(AppState);
delegate_pointer!(AppState);
delegate_shm!(AppState);
delegate_registry!(AppState);
