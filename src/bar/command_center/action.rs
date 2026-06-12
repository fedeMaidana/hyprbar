// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    VolumeSlider,
    BrightnessSlider,
    ToggleSinkMute,
    ToggleMicMute,
    MediaPlayPause,
    MediaPrevious,
    MediaNext,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl CommandAction {
    pub fn is_slider(self) -> bool {
        matches!(self, Self::VolumeSlider | Self::BrightnessSlider)
    }
}
