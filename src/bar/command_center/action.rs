// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    VolumeSlider,
    BrightnessSlider,
    ToggleSinkMute,
    ToggleMicMute,
    ToggleWifi,
    ToggleBluetooth,
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
