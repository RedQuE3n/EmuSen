//! The drawn world's shapes as a Qt Quick Shapes delegate reads them: one row per kit-core `Shape`, in the body's unit box (EmuSen_Kit.md §7.5).

use emusen_kit_core::tokens::Colour;
use emusen_kit_core::world::{self, Paint, Point, Shape, WorldBody, WorldRing, WorldShape};

/// The most gradient stops a row carries; a shorter gradient repeats its last stop, since QML cannot repeat a GradientStop.
pub const STOPS: usize = 4;

pub const CLEAR: Colour = Colour::rgba(0, 0, 0, 0);

/// A radial gradient as `RadialGradient` takes it: one centre, one radius, the focal point at the centre.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub centre: Point,
    pub radius: f64,
    pub stops: [(f64, Colour); STOPS],
}

/// One row: a closed outline repeats its first point, no fill or stroke is a clear colour and a width of nought.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeRow {
    pub points: Vec<Point>,
    pub fill: Colour,
    pub gradient: Option<Gradient>,
    pub stroke: Colour,
    pub stroke_width: f64,
    pub opacity: f64,
}

pub fn pad_stops(stops: &[(f64, Colour)]) -> [(f64, Colour); STOPS] {
    let last = stops.last().copied().unwrap_or((1.0, CLEAR));
    std::array::from_fn(|i| stops.get(i).copied().unwrap_or(last))
}

pub fn row(shape: &Shape) -> ShapeRow {
    let mut points = shape.points.clone();
    if shape.closed && points.len() > 1 && points.first() != points.last() {
        points.push(points[0]);
    }
    let (fill, gradient) = match &shape.fill {
        Some(Paint::Solid(colour)) => (*colour, None),
        Some(Paint::Radial { centre, radius, stops }) => {
            (CLEAR, Some(Gradient { centre: *centre, radius: *radius, stops: pad_stops(stops) }))
        }
        Some(Paint::Linear { stops, .. }) => (stops.first().map_or(CLEAR, |s| s.1), None),
        None => (CLEAR, None),
    };
    let (stroke, stroke_width) = shape.stroke.map_or((CLEAR, 0.0), |s| (s.colour, s.width));
    ShapeRow { points, fill, gradient, stroke, stroke_width, opacity: shape.opacity }
}

/// A body's rows, back to front, as kit-core draws it.
pub fn rows(body: &WorldBody, hue: f64, focused: bool) -> Vec<ShapeRow> {
    world::geometry(body, hue, focused).iter().map(row).collect()
}

/// Three `#RRGGBB` words as a palette; anything else is none, so the hue's own colours are drawn.
pub fn palette(words: &str) -> Option<[emusen_kit_core::tokens::Colour; 3]> {
    let c: Vec<_> = words.split_whitespace().map(emusen_kit_core::world::hex).collect();
    match c.as_slice() {
        [Some(a), Some(b), Some(d)] => Some([*a, *b, *d]),
        _ => None,
    }
}

/// A palette as the three `#RRGGBB` words `palette` reads, or empty for none.
pub fn palette_words(palette: Option<[emusen_kit_core::tokens::Colour; 3]>) -> String {
    palette.map(|p| p.iter().map(|c| format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)).collect::<Vec<_>>().join(" ")).unwrap_or_default()
}

pub fn body(shape: WorldShape, craters: f64, bands: f64, cap: f64, ring: WorldRing, moons: u8) -> WorldBody {
    WorldBody { shape, palette: None, craters, bands, cap, ring, moons }
}
