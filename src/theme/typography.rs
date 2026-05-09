//! Configuración tipográfica.

#[derive(Debug, Clone)]
pub struct Typography {
    /// Familia para texto general (números, etiquetas).
    pub font_family: String,
    /// Familia para íconos (Nerd Font). Los glyphs de Nerd Fonts viven en
    /// Private Use Area, donde fontique no hace fallback automático, así
    /// que hay que pedírsela explícitamente.
    pub icon_font_family: String,
    pub size_base: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            // Nombre típico; ajustar según `fc-list | grep -i nerd`.
            icon_font_family: "Symbols Nerd Font".to_string(),
            size_base: 10.0,
        }
    }
}
