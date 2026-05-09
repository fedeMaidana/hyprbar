//! La barra en sí: organiza componentes en secciones (left/center/right)
//! y dibuja todo en orden.

pub mod arch_logo_pill;
pub mod date_pill;
pub mod clock_pill;
pub mod weather_pill;
pub mod command_center_pill;
pub mod workspaces_pill;
pub mod notifications_pill;
pub mod profile_pill;
pub mod layout;

pub use layout::Bar;
