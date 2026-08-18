// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use hyprbar::app::App;

// ─── < Entry Point > ────────────────────────────────────────────────────

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("hyprbar arrancando");

    App::run()
}
