// ─── < Modules > ────────────────────────────────────────────────────

pub mod colors;
pub mod hyprcolor;
pub mod tokens;
pub mod typography;

// ─── < Imports > ────────────────────────────────────────────────────

pub use colors::Palette;
pub use tokens::Tokens;
pub use typography::Typography;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Theme {
    pub palette: Palette,
    pub tokens: Tokens,
    pub typography: Typography,
}

// ─── < Implementations > ────────────────────────────────────────────────────

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
