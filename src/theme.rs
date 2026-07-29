// ─── < Modules > ────────────────────────────────────────────────────

pub mod colors;
pub mod hyprcolor;
pub mod mode;
pub mod tokens;
pub mod typography;

// ─── < Imports > ────────────────────────────────────────────────────

pub use colors::Palette;
pub use mode::ThemeMode;
pub use tokens::Tokens;
pub use typography::Typography;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub palette: Palette,
    pub tokens: Tokens,
    pub typography: Typography,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Theme {
    pub fn dark() -> Self {
        Self::of(ThemeMode::Dark)
    }

    pub fn light() -> Self {
        Self::of(ThemeMode::Light)
    }

    pub fn of(mode: ThemeMode) -> Self {
        let palette = match mode {
            ThemeMode::Dark => Palette::dark(),
            ThemeMode::Light => Palette::light(),
        };

        Self {
            mode,
            palette,
            tokens: Tokens::default(),
            typography: Typography::default(),
        }
    }

    /// Builds the theme for the mode persisted by the user.
    pub fn preferred() -> Self {
        Self::of(mode::load_preferred())
    }

    pub fn toggled(&self) -> Self {
        Self::of(self.mode.toggled())
    }

    pub fn refresh_dynamic_colors(&mut self) {
        self.palette.refresh_from_hyprcolor();
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
