mod theme;
mod app;
mod components;

use iced_layershell::settings::Settings;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::Application;
use mimalloc::MiMalloc;
use app::Bar;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub fn main() -> iced_layershell::Result {
    let mut settings = Settings::default();

    // 2. CONFIGURACIÓN DE BARRA
    settings.layer_settings.layer = Layer::Top;
    settings.layer_settings.anchor = Anchor::Top | Anchor::Left | Anchor::Right;
    settings.layer_settings.exclusive_zone = 35;
    settings.layer_settings.size = Some((0, 35));

    Bar::run(settings)
}