use hyprbar::components::{ComponentAction, ComponentTag, Interaction, InteractionOutcome};

const TAG: ComponentTag = ComponentTag::new("tag-a");
const OTHER_TAG: ComponentTag = ComponentTag::new("tag-b");

#[test]
fn component_action_round_trips_its_fields() {
    let action = ComponentAction::new(TAG, 7).with_value(-3);

    assert_eq!(action.owner(), TAG);
    assert_eq!(action.id(), 7);
    assert_eq!(action.value(), -3);
    assert!(!action.is_draggable());

    let draggable = ComponentAction::new(TAG, 1).draggable();

    assert!(draggable.is_draggable());
}

#[test]
fn actions_with_different_owner_id_or_value_are_distinct() {
    let base = ComponentAction::new(TAG, 1).with_value(10);

    assert_ne!(base, ComponentAction::new(OTHER_TAG, 1).with_value(10));
    assert_ne!(base, ComponentAction::new(TAG, 2).with_value(10));
    assert_ne!(base, ComponentAction::new(TAG, 1).with_value(11));
    assert_eq!(base, ComponentAction::new(TAG, 1).with_value(10));
}

#[test]
fn interaction_is_draggable_only_for_draggable_actions() {
    let click = Interaction::Action(ComponentAction::new(TAG, 0));
    let drag = Interaction::Action(ComponentAction::new(TAG, 0).draggable());

    assert!(!click.is_draggable());
    assert!(drag.is_draggable());
}

#[test]
fn outcome_constructors_set_exactly_one_flag() {
    let quiet = InteractionOutcome::quiet();
    assert!(!quiet.redraw && !quiet.close_dropdown && !quiet.toggle_theme);

    let redraw = InteractionOutcome::redraw();
    assert!(redraw.redraw && !redraw.close_dropdown && !redraw.toggle_theme);

    let close = InteractionOutcome::close_dropdown();
    assert!(!close.redraw && close.close_dropdown && !close.toggle_theme);

    let theme = InteractionOutcome::toggle_theme();
    assert!(!theme.redraw && !theme.close_dropdown && theme.toggle_theme);
}
