// ─── < Imports > ────────────────────────────────────────────────────

use vello::kurbo::BezPath;

use crate::render::Rect;

// ─── < Constants > ────────────────────────────────────────────────────

/// Cuánto se extiende la esquina sobre los lados respecto del radio
/// nominal (>1 = la curva arranca antes, estilo curvatura continua).
const CORNER_EXTENSION: f64 = 1.27;

/// Cuánto se acercan los puntos de control al vértice (0.55 ≈ círculo,
/// más alto = esquina más "squircle").
const CORNER_PULL: f64 = 0.86;

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Rectángulo con esquinas de curvatura continua (aproximación al
/// squircle de iOS/macOS): la transición borde→curva no tiene el salto
/// de curvatura de un arco circular, y eso se percibe más "caro".
pub fn squircle(bounds: Rect, radius: f64) -> BezPath {
    let x0 = bounds.x as f64;
    let y0 = bounds.y as f64;
    let x1 = (bounds.x + bounds.width) as f64;
    let y1 = (bounds.y + bounds.height) as f64;

    // La extensión se capea para que dos esquinas no se pisen entre sí.
    let max_span = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
    let span = (radius * CORNER_EXTENSION).clamp(0.0, max_span);
    let pull = span * CORNER_PULL;

    let mut path = BezPath::new();

    path.move_to((x0 + span, y0));

    // Lado superior y esquina superior derecha.
    path.line_to((x1 - span, y0));
    path.curve_to((x1 - span + pull, y0), (x1, y0 + span - pull), (x1, y0 + span));

    // Lado derecho y esquina inferior derecha.
    path.line_to((x1, y1 - span));
    path.curve_to((x1, y1 - span + pull), (x1 - span + pull, y1), (x1 - span, y1));

    // Lado inferior y esquina inferior izquierda.
    path.line_to((x0 + span, y1));
    path.curve_to((x0 + span - pull, y1), (x0, y1 - span + pull), (x0, y1 - span));

    // Lado izquierdo y esquina superior izquierda.
    path.line_to((x0, y0 + span));
    path.curve_to((x0, y0 + span - pull), (x0 + span - pull, y0), (x0 + span, y0));

    path.close_path();

    path
}

/// La misma forma, contraída `inset` px por lado (para bordes hairline).
pub fn squircle_inset(bounds: Rect, radius: f64, inset: f32) -> BezPath {
    let deflated = Rect::new(bounds.x + inset, bounds.y + inset, bounds.width - inset * 2.0, bounds.height - inset * 2.0);

    squircle(deflated, (radius - inset as f64).max(0.0))
}
