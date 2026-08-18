// ─── < Imports > ────────────────────────────────────────────────────

use calloop::channel::Sender;

use crate::bar::clock::ClockPill;
use crate::bar::command_center::CommandCenterPill;
use crate::bar::date::DatePill;
use crate::bar::layout::Bar;
use crate::bar::notifications::NotificationsPill;
use crate::bar::profile::ProfilePill;
use crate::bar::system::ArchLogoPill;
use crate::bar::weather::{WeatherConfig, WeatherPill};
use crate::bar::workspaces::WorkspacesPill;
use crate::components::Component;

// ─── < Constants > ────────────────────────────────────────────────────

const PROFILE_IMAGE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/profile.jpeg");

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn default_bar(redraw_signal: Sender<()>) -> Bar {
    Bar::new(
        vec![
            component(ArchLogoPill::new(redraw_signal.clone())),
            component(DatePill::new()),
            component(ClockPill::new()),
            component(WeatherPill::new(WeatherConfig::auto_detect(), redraw_signal.clone())),
        ],
        vec![
            component(CommandCenterPill::new(redraw_signal.clone())),
            component(WorkspacesPill::new(redraw_signal.clone())),
            component(NotificationsPill::new(redraw_signal)),
        ],
        vec![component(ProfilePill::from_path(PROFILE_IMAGE_PATH))],
    )
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn component<C>(component: C) -> Box<dyn Component>
where
    C: Component + 'static,
{
    Box::new(component)
}
