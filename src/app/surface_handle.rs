// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use raw_window_handle::DisplayHandle;
use raw_window_handle::HandleError;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use raw_window_handle::WaylandDisplayHandle;
use raw_window_handle::WaylandWindowHandle;
use raw_window_handle::WindowHandle;
use std::ptr::NonNull;
use wayland_client::{Connection, Proxy};

// ─── < Structs > ────────────────────────────────────────────────────

pub struct SurfaceHandle {
    display_ptr: NonNull<std::ffi::c_void>,
    surface_ptr: NonNull<std::ffi::c_void>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

// SAFETY: both pointers are opaque handles that this crate never
// dereferences; only wgpu consumes them, and it requires `Send + Sync`
// on the handle type. The underlying `wl_display`/`wl_surface` outlive
// the handle: `AppState::recreate_surface` drops the wgpu surface
// *before* destroying the Wayland objects it points at (see the
// ordering comment there), and `create_surface` is only called with a
// live surface from `runner.rs` and `render.rs`.
unsafe impl Send for SurfaceHandle {}
// SAFETY: see the `Send` impl above; the pointers are never read
// through by this crate, so sharing references across threads is sound.
unsafe impl Sync for SurfaceHandle {}

impl SurfaceHandle {
    pub fn new(conn: &Connection, surface: &wayland_client::protocol::wl_surface::WlSurface) -> Self {
        let display_ptr = NonNull::new(conn.backend().display_ptr() as *mut _).expect("display ptr no debería ser null");

        let surface_ptr = NonNull::new(surface.id().as_ptr() as *mut _).expect("surface ptr no debería ser null");

        Self { display_ptr, surface_ptr }
    }
}

impl HasDisplayHandle for SurfaceHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = WaylandDisplayHandle::new(self.display_ptr);

        // SAFETY: `borrow_raw` only asks that the handle stay valid for
        // the returned lifetime, which is tied to `&self`; the display
        // pointer lives as long as the Wayland `Connection` in `AppState`.
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(handle)) })
    }
}

impl HasWindowHandle for SurfaceHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = WaylandWindowHandle::new(self.surface_ptr);

        // SAFETY: same contract as `display_handle` — the wl_surface is
        // kept alive by `AppState` for at least as long as this handle
        // (the wgpu surface is dropped before the wl_surface it targets).
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Wayland(handle)) })
    }
}
