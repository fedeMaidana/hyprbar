//! Abstracciones sobre Smithay Client Toolkit.
//!
//! En esta arquitectura `AppState` (definido en `crate::app`) es el state
//! que dispatchea el EventQueue de Wayland. Este módulo provee:
//! - `init`: setup inicial (bind de globals, layer surface).
//! - `handlers`: impls de los traits delegate_* sobre AppState.
//! - `layer_surface`: configuración del layer-shell (anchor, capa, etc.).

pub mod handlers;
pub mod init;
pub mod layer_surface;

pub use init::{init, WaylandInit};
pub use layer_surface::LayerConfig;