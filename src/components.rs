// ─── < Modules > ────────────────────────────────────────────────────

pub mod button_row;
pub mod component;
pub mod dropdown;
pub mod panel;
pub mod pill;
pub mod text_fit;

// ─── < Public API > ────────────────────────────────────────────────────

pub use button_row::evenly_spaced_rects;
pub use component::{Component, ComponentAction, ComponentTag, ConfirmRequest, Interaction, InteractionOutcome, Point, RenderCtx};
pub use dropdown::{Dropdown, DropdownFrame, DropdownId, DropdownItem};
pub use panel::{Panel, PanelHeader};
pub use pill::Pill;
pub use text_fit::{ELLIPSIS, truncate_to_width};
