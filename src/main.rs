use anyhow::Result;

mod app;
mod bar;
mod components;
mod hyprland_ipc;
mod render;
mod theme;
mod wayland;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("hyprbar starting");
    app::App::run()?;
    Ok(())
}
