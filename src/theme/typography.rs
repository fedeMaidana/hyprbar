// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Typography {
    pub font_family: String,
    pub icon_font_family: String,
    pub size_base: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            icon_font_family: "Symbols Nerd Font".to_string(),
            size_base: 11.0,
        }
    }
}
