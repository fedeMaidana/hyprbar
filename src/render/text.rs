// ─── < Imports > ────────────────────────────────────────────────────

use std::hash::{Hash, Hasher};

use hashbrown::{Equivalent, HashMap};
use parley::{
    FontContext, Layout, LayoutContext, StyleProperty,
    style::{FontFamily, FontWeight},
};
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Brush, Color, Fill};

// ─── < Constants > ────────────────────────────────────────────────────

/// Cuando la caché llega acá, se vacía entera: simple y suficiente para
/// una barra cuyos textos vivos por frame son unas pocas decenas.
const MAX_CACHED_LAYOUTS: usize = 512;

/// Peso tipográfico por defecto (regular).
const DEFAULT_WEIGHT: f32 = 400.0;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct TextStyle<'a> {
    pub size: f32,
    pub family: &'a str,
    pub color: Color,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LayoutKey {
    text: String,
    family: String,
    size_bits: u32,
    weight_bits: u32,
}

/// Versión prestada de `LayoutKey`: alcanza para buscar en la caché sin
/// alocar; la clave dueña recién se construye si hay que insertar.
struct LayoutKeyRef<'a> {
    text: &'a str,
    family: &'a str,
    size_bits: u32,
    weight_bits: u32,
}

pub struct TextEngine {
    font_cx: FontContext,
    layout_cx: LayoutContext<Brush>,
    /// Shaping cacheado por (texto, tamaño, familia). El color no participa:
    /// se aplica recién al dibujar, así un cambio de theme no invalida nada.
    cache: HashMap<LayoutKey, Layout<Brush>>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Hash for LayoutKeyRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Mismos campos y mismo orden que el derive de `LayoutKey`, para
        // que ambas formas de la clave caigan en el mismo bucket.
        self.text.hash(state);
        self.family.hash(state);
        self.size_bits.hash(state);
        self.weight_bits.hash(state);
    }
}

impl Equivalent<LayoutKey> for LayoutKeyRef<'_> {
    fn equivalent(&self, key: &LayoutKey) -> bool {
        self.size_bits == key.size_bits && self.weight_bits == key.weight_bits && self.text == key.text && self.family == key.family
    }
}

impl<'a> TextStyle<'a> {
    pub fn new(size: f32, family: &'a str, color: Color) -> Self {
        Self {
            size,
            family,
            color,
            weight: DEFAULT_WEIGHT,
        }
    }

    /// Mismo estilo con otro peso (400 normal, ~550 medium, 700 bold).
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            cache: HashMap::new(),
        }
    }

    pub fn measure(&mut self, text: &str, size: f32, family: &str) -> (f32, f32) {
        let layout = self.cached_layout(text, size, family, DEFAULT_WEIGHT);

        (layout.width(), layout.height())
    }

    pub fn draw_centered_v(&mut self, scene: &mut Scene, text: &str, x: f32, box_y: f32, box_height: f32, style: TextStyle<'_>) {
        let color = style.color;
        let layout = self.cached_layout(text, style.size, style.family, style.weight);
        let y = box_y + (box_height - layout.height()) / 2.0;

        draw_layout(scene, layout, x, y, color);
    }

    /// Draws text centered both ways inside `bounds`.
    pub fn draw_centered(&mut self, scene: &mut Scene, text: &str, bounds: crate::render::Rect, style: TextStyle<'_>) {
        let color = style.color;
        let layout = self.cached_layout(text, style.size, style.family, style.weight);

        let x = bounds.x + (bounds.width - layout.width()) / 2.0;
        let y = bounds.y + (bounds.height - layout.height()) / 2.0;

        draw_layout(scene, layout, x, y, color);
    }

    fn cached_layout(&mut self, text: &str, size: f32, family: &str, weight: f32) -> &Layout<Brush> {
        let Self { font_cx, layout_cx, cache } = self;

        let key = LayoutKeyRef {
            text,
            family,
            size_bits: size.to_bits(),
            weight_bits: weight.to_bits(),
        };

        if !cache.contains_key(&key) {
            if cache.len() >= MAX_CACHED_LAYOUTS {
                cache.clear();
            }

            let layout = build_layout(font_cx, layout_cx, text, size, family, weight);

            cache.insert(
                LayoutKey {
                    text: text.to_owned(),
                    family: family.to_owned(),
                    size_bits: size.to_bits(),
                    weight_bits: weight.to_bits(),
                },
                layout,
            );
        }

        cache.get(&key).expect("la clave se insertó recién arriba")
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn build_layout(
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<Brush>,
    text: &str,
    size: f32,
    family: &str,
    weight: f32,
) -> Layout<Brush> {
    let mut builder = layout_cx.ranged_builder(font_cx, text, 1.0, true);

    builder.push_default(StyleProperty::FontSize(size));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));

    let family = FontFamily::named(family);
    builder.push_default(StyleProperty::FontFamily(family));

    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    layout.align(parley::Alignment::Start, parley::AlignmentOptions::default());

    layout
}

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
