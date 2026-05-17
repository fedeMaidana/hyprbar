use calloop::channel::Sender;

use crate::bar::arch_logo_pill::ArchLogoPill;
use crate::bar::clock_pill::ClockPill;
use crate::bar::command_center_pill::CommandCenterPill;
use crate::bar::date_pill::DatePill;
use crate::bar::layout::Bar;
use crate::bar::notifications_pill::NotificationsPill;
use crate::bar::profile::ProfilePill;
use crate::bar::weather::{WeatherConfig, WeatherPill};
use crate::bar::workspaces::WorkspacesPill;
use crate::components::Component;

pub fn default_bar(redraw_signal: Sender<()>) -> Bar {
    Bar::new(
        vec![
            Box::new(ArchLogoPill::new()) as Box<dyn Component>,
            Box::new(DatePill::new()),
            Box::new(ClockPill::new()),
            Box::new(WeatherPill::new(WeatherConfig::mar_del_plata())),
        ],
        vec![
            Box::new(CommandCenterPill::new()) as Box<dyn Component>,
            Box::new(WorkspacesPill::new(redraw_signal)),
            Box::new(NotificationsPill::new()),
        ],
        vec![Box::new(ProfilePill::from_path("assets/profile.jpeg"))],
    )
}
