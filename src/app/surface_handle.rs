use std::ptr::NonNull;

use anyhow::Result;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use wayland_client::{Connection, Proxy};

pub struct SurfaceHandle {
    display_ptr: NonNull<std::ffi::c_void>,
    surface_ptr: NonNull<std::ffi::c_void>,
}

unsafe impl Send for SurfaceHandle {}
unsafe impl Sync for SurfaceHandle {}

impl SurfaceHandle {
    pub fn new(
        conn: &Connection,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
    ) -> Self {
        let display_ptr = NonNull::new(conn.backend().display_ptr() as *mut _)
            .expect("display ptr no debería ser null");

        let surface_ptr =
            NonNull::new(surface.id().as_ptr() as *mut _).expect("surface ptr no debería ser null");

        Self {
            display_ptr,
            surface_ptr,
        }
    }
}

impl HasDisplayHandle for SurfaceHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = WaylandDisplayHandle::new(self.display_ptr);

        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(handle)) })
    }
}

impl HasWindowHandle for SurfaceHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = WaylandWindowHandle::new(self.surface_ptr);

        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Wayland(handle)) })
    }
}
