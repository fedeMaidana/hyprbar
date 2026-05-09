//! Text rendering vía parley (shaping/layout) + vello (glyph drawing).

use parley::{FontContext, Layout, LayoutContext, StyleProperty, style::FontFamily};
use vello::{
    Scene,
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

pub struct TextEngine {
    font_cx: FontContext,
    layout_cx: LayoutContext<Brush>,
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
        }
    }

    pub fn layout(&mut self, text: &str, size: f32, family: &str) -> Layout<Brush> {
        let mut builder = self
            .layout_cx
            .ranged_builder(&mut self.font_cx, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(size));
        // En parley 0.9 ya no hay FontStack — solo una FontFamily a la vez.
        // Si la family no está instalada, el sistema de fallback de fontique
        // (que usa fontconfig en Linux) elige una sans-serif compatible.
        let fam = FontFamily::named(family);
        builder.push_default(StyleProperty::FontFamily(fam));
        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout.align(
            parley::Alignment::Start,
            parley::AlignmentOptions::default(),
        );
        layout
    }

    pub fn measure(&mut self, text: &str, size: f32, family: &str) -> (f32, f32) {
        let layout = self.layout(text, size, family);
        (layout.width(), layout.height())
    }

    pub fn draw(
        &mut self,
        scene: &mut Scene,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        family: &str,
        color: Color,
    ) {
        let layout = self.layout(text, size, family);
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
                let glyph_xform = synthesis
                    .skew()
                    .map(|skew| Affine::skew(skew.to_radians().tan() as f64, 0.0));

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
                        glyph_run.glyphs().map(|g| {
                            let gx = x_pos + g.x;
                            let gy = y_pos - g.y;
                            x_pos += g.advance;
                            vello::Glyph {
                                id: g.id as u32,
                                x: gx,
                                y: gy,
                            }
                        }),
                    );
            }
        }
    }

    /// Dibuja `text` centrado verticalmente dentro de la franja vertical
    /// `[box_y, box_y + box_height]` usando el centro matemático del
    /// bounding box que devuelve parley.
    pub fn draw_centered_v(
        &mut self,
        scene: &mut Scene,
        text: &str,
        x: f32,
        box_y: f32,
        box_height: f32,
        size: f32,
        family: &str,
        color: Color,
    ) {
        let (_, th) = self.measure(text, size, family);
        let y = box_y + (box_height - th) / 2.0;
        self.draw(scene, text, x, y, size, family, color);
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}
