// ─── < Imports > ────────────────────────────────────────────────────

use parley::{FontContext, Layout, LayoutContext, StyleProperty, style::FontFamily};
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Brush, Color, Fill};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct TextStyle<'a> {
    pub size: f32,
    pub family: &'a str,
    pub color: Color,
}

pub struct TextEngine {
    font_cx: FontContext,
    layout_cx: LayoutContext<Brush>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl<'a> TextStyle<'a> {
    pub fn new(size: f32, family: &'a str, color: Color) -> Self {
        Self { size, family, color }
    }
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
        }
    }

    pub fn layout(&mut self, text: &str, size: f32, family: &str) -> Layout<Brush> {
        let mut builder = self.layout_cx.ranged_builder(&mut self.font_cx, text, 1.0, true);

        builder.push_default(StyleProperty::FontSize(size));

        let family = FontFamily::named(family);
        builder.push_default(StyleProperty::FontFamily(family));

        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout.align(parley::Alignment::Start, parley::AlignmentOptions::default());

        layout
    }

    pub fn measure(&mut self, text: &str, size: f32, family: &str) -> (f32, f32) {
        let layout = self.layout(text, size, family);

        (layout.width(), layout.height())
    }

    pub fn draw_centered_v(&mut self, scene: &mut Scene, text: &str, x: f32, box_y: f32, box_height: f32, style: TextStyle<'_>) {
        let layout = self.layout(text, style.size, style.family);
        let y = box_y + (box_height - layout.height()) / 2.0;

        draw_layout(scene, &layout, x, y, style.color);
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_layout(scene: &mut Scene, layout: &Layout<Brush>, x: f32, y: f32, color: Color) {
    let brush = Brush::Solid(color);

    for line in layout.lines() {
        for item in line.items() {
            let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let run = glyph_run.run();
            let font = run.font();
            let font_size = run.font_size();
            let synthesis = run.synthesis();

            let glyph_xform = synthesis.skew().map(|skew| Affine::skew(skew.to_radians().tan() as f64, 0.0));

            let mut x_pos = glyph_run.offset();
            let y_pos = glyph_run.baseline();

            scene
                .draw_glyphs(font)
                .brush(&brush)
                .transform(Affine::translate((x as f64, y as f64)))
                .glyph_transform(glyph_xform)
                .font_size(font_size)
                .normalized_coords(run.normalized_coords())
                .draw(
                    Fill::NonZero,
                    glyph_run.glyphs().map(|glyph| {
                        let gx = x_pos + glyph.x;
                        let gy = y_pos - glyph.y;
                        x_pos += glyph.advance;

                        vello::Glyph {
                            id: glyph.id,
                            x: gx,
                            y: gy,
                        }
                    }),
                );
        }
    }
}
