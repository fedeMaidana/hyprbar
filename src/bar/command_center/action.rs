// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    VolumeSlider,
    ToggleSinkMute,
    ToggleMicMute,
    ToggleWifi,
    ToggleTheme,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl CommandAction {
    pub fn is_slider(self) -> bool {
        matches!(self, Self::VolumeSlider)
    }
}
