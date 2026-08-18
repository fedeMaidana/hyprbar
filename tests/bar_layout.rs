use hyprbar::bar::Bar;
use hyprbar::components::{Component, ComponentAction, ComponentTag, Interaction, InteractionOutcome, Point, RenderCtx};
use hyprbar::render::{Rect, TextEngine};
use hyprbar::theme::Theme;
use vello::Scene;

const TEST_TAG: ComponentTag = ComponentTag::new("test");

struct FixedComponent {
    width: f32,
    height: f32,
    interaction: Interaction,
}

impl FixedComponent {
    fn new(width: f32, height: f32, interaction: Interaction) -> Self {
        Self {
            width,
            height,
            interaction,
        }
    }
}

impl Component for FixedComponent {
    fn measure(&mut self, _ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        (self.width, self.height)
    }

    fn render(&mut self, _scene: &mut Scene, _bounds: Rect, _ctx: &mut RenderCtx<'_>) {}

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(self.interaction)
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> Option<InteractionOutcome> {
        (interaction == self.interaction).then(InteractionOutcome::redraw)
    }
}

fn slot_interaction(id: i32) -> Interaction {
    Interaction::Action(ComponentAction::new(TEST_TAG, 0).with_value(id))
}

fn fixed_slot(id: i32, width: f32, height: f32) -> Box<dyn Component> {
    Box::new(FixedComponent::new(width, height, slot_interaction(id)))
}

fn render_bar(bar: &mut Bar, theme: &Theme, text_engine: &mut TextEngine, surface: Rect) {
    let mut scene = Scene::new();

    let mut ctx = RenderCtx {
        theme,
        text: text_engine,
        hovered_interaction: None,
        open_dropdown: None,
        dt: 0.0,
        animating: false,
    };

    bar.render(&mut scene, surface, theme, &mut ctx);
}

#[test]
fn hit_test_returns_none_before_first_render() {
    let theme = Theme::default();

    let bar = Bar::new(vec![fixed_slot(1, 20.0, 26.0)], vec![fixed_slot(2, 40.0, 26.0)], vec![fixed_slot(3, 20.0, 26.0)]);

    assert_eq!(bar.hit_test(Point::new(15.0, 10.0), &theme, None), None);
}

#[test]
fn hit_test_detects_left_center_and_right_sections_after_render() {
    let theme = Theme::default();
    let mut text_engine = TextEngine::new();

    let mut bar = Bar::new(vec![fixed_slot(1, 20.0, 26.0)], vec![fixed_slot(2, 40.0, 26.0)], vec![fixed_slot(3, 20.0, 26.0)]);

    render_bar(&mut bar, &theme, &mut text_engine, Rect::new(0.0, 0.0, 300.0, 36.0));

    assert_eq!(bar.hit_test(Point::new(15.0, 10.0), &theme, None), Some(slot_interaction(1)));

    assert_eq!(bar.hit_test(Point::new(150.0, 10.0), &theme, None), Some(slot_interaction(2)));

    assert_eq!(bar.hit_test(Point::new(275.0, 10.0), &theme, None), Some(slot_interaction(3)));

    assert_eq!(bar.hit_test(Point::new(100.0, 10.0), &theme, None), None);
}

#[test]
fn hit_test_preserves_right_section_visual_order() {
    let theme = Theme::default();
    let mut text_engine = TextEngine::new();

    let mut bar = Bar::new(vec![], vec![], vec![fixed_slot(1, 20.0, 26.0), fixed_slot(2, 30.0, 26.0)]);

    render_bar(&mut bar, &theme, &mut text_engine, Rect::new(0.0, 0.0, 300.0, 36.0));

    assert_eq!(bar.hit_test(Point::new(240.0, 10.0), &theme, None), Some(slot_interaction(1)));

    assert_eq!(bar.hit_test(Point::new(265.0, 10.0), &theme, None), Some(slot_interaction(2)));
}

#[test]
fn handle_interaction_routes_to_the_owning_component() {
    let mut bar = Bar::new(vec![fixed_slot(1, 20.0, 26.0)], vec![fixed_slot(2, 40.0, 26.0)], vec![fixed_slot(3, 20.0, 26.0)]);

    // La primera respuesta Some corta la búsqueda; los demás no opinan.
    assert_eq!(bar.handle_interaction(slot_interaction(2)), Some(InteractionOutcome::redraw()));

    let foreign = Interaction::Action(ComponentAction::new(ComponentTag::new("nadie"), 9));

    assert_eq!(bar.handle_interaction(foreign), None);
}

#[test]
fn hides_center_section_when_it_would_overlap_left_and_right_sections() {
    let theme = Theme::default();
    let mut text_engine = TextEngine::new();

    let mut bar = Bar::new(vec![fixed_slot(1, 120.0, 26.0)], vec![fixed_slot(2, 120.0, 26.0)], vec![fixed_slot(3, 120.0, 26.0)]);

    render_bar(&mut bar, &theme, &mut text_engine, Rect::new(0.0, 0.0, 300.0, 36.0));

    assert_eq!(bar.hit_test(Point::new(20.0, 10.0), &theme, None), Some(slot_interaction(1)));

    assert_eq!(bar.hit_test(Point::new(180.0, 10.0), &theme, None), Some(slot_interaction(3)));

    assert_eq!(bar.hit_test(Point::new(150.0, 10.0), &theme, None), None);
}
