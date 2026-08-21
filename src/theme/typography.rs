// ─── < Structs > ────────────────────────────────────────────────────

/// Las familias son literales del binario: `&'static str` evita allocs
/// en cada construcción de Theme y dice la verdad del tipo.
#[derive(Debug, Clone, Copy)]
pub struct Typography {
    pub font_family: &'static str,
    pub icon_font_family: &'static str,
    pub size_base: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "Inter",
            icon_font_family: "Symbols Nerd Font",
            size_base: 12.0,
        }
    }
}
