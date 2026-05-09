//! Sistema de diseño centralizado.
//!
//! Todos los componentes deben consumir tokens de acá, nunca hardcodear.
//! Cambiar el tema = cambiar este módulo.

pub mod colors;
pub mod tokens;
pub mod typography;

pub use colors::Palette;
pub use tokens::Tokens;
pub use typography::Typography;

/// Tema completo. Singleton conceptual — se construye una vez y se pasa por referencia.
#[derive(Debug, Clone)]
pub struct Theme {
    pub palette: Palette,
    pub tokens: Tokens,
    pub typography: Typography,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            palette: Palette::dark(),
            tokens: Tokens::default(),
            typography: Typography::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
