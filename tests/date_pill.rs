use hyprbar::bar::date::{CalendarAction, DatePill};
use hyprbar::components::{Component, ComponentAction, ComponentTag, Interaction};

fn foreign_interaction() -> Interaction {
    Interaction::Action(ComponentAction::new(ComponentTag::new("otro"), 0))
}

#[test]
fn ignores_actions_from_other_components() {
    let mut pill = DatePill::new();

    assert_eq!(pill.handle_interaction(foreign_interaction()), None);
}

#[test]
fn navigation_reports_redraw() {
    let mut pill = DatePill::new();

    let outcome = pill
        .handle_interaction(CalendarAction::PrevMonth.interaction())
        .expect("acción propia");

    assert!(outcome.redraw);
    assert!(!outcome.close_dropdown && !outcome.toggle_theme);
}

#[test]
fn today_is_quiet_when_already_on_the_current_month() {
    let mut pill = DatePill::new();

    let outcome = pill.handle_interaction(CalendarAction::Today.interaction()).expect("acción propia");

    assert!(!outcome.redraw, "sin offset, volver a hoy no debería redibujar");
}

#[test]
fn today_redraws_after_navigating_away() {
    let mut pill = DatePill::new();

    let _ = pill.handle_interaction(CalendarAction::NextMonth.interaction());

    let outcome = pill.handle_interaction(CalendarAction::Today.interaction()).expect("acción propia");

    assert!(outcome.redraw);
}
