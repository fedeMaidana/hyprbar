pub mod colors;
pub mod hyprcolor;
pub mod tokens;
pub mod typography;

pub use colors::Palette;
pub use tokens::Tokens;
pub use typography::Typography;

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
