//! Componentes reutilizables.
//!
//! - `Component` es el trait base: medir + renderizar.
//! - `Pill` es el primitive visual (fondo redondeado con sombra).
//! - Los componentes específicos viven en `bar/`.

pub mod component;
pub mod pill;

pub use component::{Component, RenderCtx};
pub use pill::Pill;
