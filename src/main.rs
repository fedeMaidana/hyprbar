// ─── < MODULES > ───────────────────────────────────────────────────────
mod theme;
mod app;
mod components;
pub mod ui;

// ─── < IMPORTS > ───────────────────────────────────────────────────────
use iced_layershell::settings::Settings;
use iced_layershell::reexport::{ Layer, Anchor };
use iced_layershell::Application;
use mimalloc::MiMalloc;

use crate::app::Bar;
use crate::theme::palette;

#[ global_allocator ]
static GLOBAL: MiMalloc = MiMalloc;

// ─── < MAIN FUNCTION > ───────────────────────────────────────────────────────
pub fn main() -> iced_layershell::Result {
    let mut settings = Settings::default();

    settings.layer_settings.layer = Layer::Top;
    settings.layer_settings.anchor = Anchor::Top | Anchor::Left | Anchor::Right;
    settings.layer_settings.exclusive_zone = palette::dim::MODULE_SPACING as i32;
    settings.layer_settings.size = Some( ( 0, palette::dim::BAR_HEIGHT ) );

    Bar::run( settings )
}