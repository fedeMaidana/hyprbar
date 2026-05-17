use hyprbar::bar::Bar;
use hyprbar::bar::workspaces::WorkspaceId;
use hyprbar::components::{Component, Interaction, Point, RenderCtx};
use hyprbar::render::{Rect, TextEngine};
use hyprbar::theme::Theme;
use vello::Scene;

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
}

fn fixed_workspace(id: WorkspaceId, width: f32, height: f32) -> Box<dyn Component> {
    Box::new(FixedComponent::new(width, height, Interaction::Workspace(id)))
}

fn render_bar(bar: &mut Bar, theme: &Theme, text_engine: &mut TextEngine, surface: Rect) {
    let mut scene = Scene::new();

    let mut ctx = RenderCtx {
        theme,
        text: text_engine,
        hovered_interaction: None,
    };

    bar.render(&mut scene, surface, theme, &mut ctx);
}

#[test]
fn hit_test_returns_none_before_first_render() {
    let theme = Theme::default();

    let bar = Bar::new(vec![fixed_workspace(1, 20.0, 26.0)], vec![fixed_workspace(2, 40.0, 26.0)], vec![fixed_workspace(3, 20.0, 26.0)]);

    assert_eq!(bar.hit_test(Point::new(15.0, 10.0), &theme), None);
}

#[test]
fn hit_test_detects_left_center_and_right_sections_after_render() {
    let theme = Theme::default();
    let mut text_engine = TextEngine::new();

    let mut bar =
        Bar::new(vec![fixed_workspace(1, 20.0, 26.0)], vec![fixed_workspace(2, 40.0, 26.0)], vec![fixed_workspace(3, 20.0, 26.0)]);

    render_bar(&mut bar, &theme, &mut text_engine, Rect::new(0.0, 0.0, 300.0, 36.0));

    assert_eq!(bar.hit_test(Point::new(15.0, 10.0), &theme), Some(Interaction::Workspace(1)));

    assert_eq!(bar.hit_test(Point::new(150.0, 10.0), &theme), Some(Interaction::Workspace(2)));

    assert_eq!(bar.hit_test(Point::new(275.0, 10.0), &theme), Some(Interaction::Workspace(3)));

    assert_eq!(bar.hit_test(Point::new(100.0, 10.0), &theme), None);
}

#[test]
fn hit_test_preserves_right_section_visual_order() {
    let theme = Theme::default();
    let mut text_engine = TextEngine::new();

    let mut bar = Bar::new(vec![], vec![], vec![fixed_workspace(1, 20.0, 26.0), fixed_workspace(2, 30.0, 26.0)]);

    render_bar(&mut bar, &theme, &mut text_engine, Rect::new(0.0, 0.0, 300.0, 36.0));

    assert_eq!(bar.hit_test(Point::new(240.0, 10.0), &theme), Some(Interaction::Workspace(1)));

    assert_eq!(bar.hit_test(Point::new(265.0, 10.0), &theme), Some(Interaction::Workspace(2)));
}

#[test]
fn hides_center_section_when_it_would_overlap_left_and_right_sections() {
    let theme = Theme::default();
    let mut text_engine = TextEngine::new();

    let mut bar =
        Bar::new(vec![fixed_workspace(1, 120.0, 26.0)], vec![fixed_workspace(2, 120.0, 26.0)], vec![fixed_workspace(3, 120.0, 26.0)]);

    render_bar(&mut bar, &theme, &mut text_engine, Rect::new(0.0, 0.0, 300.0, 36.0));

    assert_eq!(bar.hit_test(Point::new(20.0, 10.0), &theme), Some(Interaction::Workspace(1)));

    assert_eq!(bar.hit_test(Point::new(180.0, 10.0), &theme), Some(Interaction::Workspace(3)));

    assert_eq!(bar.hit_test(Point::new(150.0, 10.0), &theme), None);
}
