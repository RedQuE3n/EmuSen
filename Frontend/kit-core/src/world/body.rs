use super::shape::{Paint, Point, Shape, Stroke, circle, ellipse, quadratic};
use crate::tokens::Colour;

/// The kind of body a world is drawn as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldShape {
    /// A lit disc with optional craters, bands, a polar cap, a ring and moonlets.
    Planet,
    /// A cluster of three tall shards.
    Shards,
    /// A cut gem: a crown and a pavilion of facets.
    Gem,
}

/// The size of a planet's ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum WorldRing {
    #[default]
    None,
    /// Thin and faint, close to the disc.
    Subtle,
    /// Wide, bold and bright.
    Big,
}

/// A world's body as plain data: its shape, a planet's three colours (or none, derived from the hue) and its features (LunaP §222.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBody {
    pub shape: WorldShape,
    /// A planet's light, middle and rim colours; none derives the three from the hue.
    pub palette: Option<[Colour; 3]>,
    /// Three craters in the rim colour, at this opacity.
    pub craters: f64,
    /// Two bands across the disc in the light colour, at this opacity.
    pub bands: f64,
    /// A pale polar cap, at this opacity.
    pub cap: f64,
    pub ring: WorldRing,
    /// How many moonlets, 0 to 2: the first at the upper right, the second, smaller, at the lower left.
    pub moons: u8,
}

impl WorldBody {
    /// A planet with no palette and no feature, drawn in its hue.
    pub const PLAIN: WorldBody = WorldBody::new(WorldShape::Planet);

    pub const fn new(shape: WorldShape) -> WorldBody {
        WorldBody { shape, palette: None, craters: 0.0, bands: 0.0, cap: 0.0, ring: WorldRing::None, moons: 0 }
    }
}

/// The box a shape is drawn in, in its own units: 100 by 100 for a planet, 100 by 160 for shards and a gem.
pub const fn unit_box(shape: WorldShape) -> (f64, f64) {
    (100.0, 100.0 * tallness(shape))
}

/// The box's height over its width.
pub const fn tallness(shape: WorldShape) -> f64 {
    match shape {
        WorldShape::Planet => 1.0,
        _ => 1.6,
    }
}

/// The size a shape is drawn at on a picker's tile, at rest or focused, in units (LunaP §222.2).
pub const fn size_of(shape: WorldShape, focused: bool) -> (f64, f64) {
    match (shape, focused) {
        (WorldShape::Planet, false) => (58.0, 58.0),
        (WorldShape::Planet, true) => (84.0, 84.0),
        (_, false) => (42.0, 68.0),
        (_, true) => (60.0, 96.0),
    }
}

/// A planet's three colours, the palette's or, without one, its hue's.
pub fn colours(body: &WorldBody, hue: f64) -> [Colour; 3] {
    body.palette.unwrap_or([Colour::hsl(hue, 0.85, 0.8, 1.0), Colour::hsl(hue, 0.65, 0.52, 1.0), Colour::hsl(hue, 0.6, 0.22, 1.0)])
}

/// The four facet inks of shards and a gem, at rest translucent and focused opaque.
pub fn facet_inks(hue: f64, focused: bool) -> [Colour; 4] {
    if focused {
        [
            Colour::hsl(hue, 0.8, 0.84, 1.0),
            Colour::hsl(hue, 0.7, 0.64, 1.0),
            Colour::hsl(hue, 0.95, 0.9, 1.0),
            Colour::hsl(hue, 0.6, 0.46, 1.0),
        ]
    } else {
        [
            Colour::hsl(hue, 0.5, 0.74, 0.62),
            Colour::hsl(hue, 0.45, 0.58, 0.55),
            Colour::hsl(hue, 0.6, 0.82, 0.68),
            Colour::hsl(hue, 0.38, 0.42, 0.55),
        ]
    }
}

/// The glow round a body, as a drop shadow of its shape: colour, blur and opacity.
pub fn glow(hue: f64, focused: bool) -> (Colour, f64, f64) {
    (Colour::hsl(hue, 0.9, 0.7, 1.0), if focused { 24.0 } else { 9.0 }, if focused { 0.75 } else { 0.3 })
}

const CAP_INK: Colour = Colour::hex("#FFF4F6");
const HIGHLIGHT: Colour = Colour::rgba(255, 255, 255, 128);

const SHARD_FACETS: [(&[Point], usize); 6] = [
    (&[[14.0, 74.0], [26.0, 62.0], [30.0, 156.0], [20.0, 156.0]], 1),
    (&[[26.0, 62.0], [37.0, 82.0], [38.0, 156.0], [30.0, 156.0]], 2),
    (&[[64.0, 84.0], [76.0, 52.0], [74.0, 156.0], [63.0, 156.0]], 1),
    (&[[76.0, 52.0], [88.0, 70.0], [82.0, 156.0], [74.0, 156.0]], 4),
    (&[[50.0, 4.0], [37.0, 44.0], [40.0, 156.0], [50.0, 156.0]], 3),
    (&[[50.0, 4.0], [50.0, 156.0], [60.0, 156.0], [63.0, 40.0]], 4),
];

const GEM_FACETS: [(&[Point], usize); 6] = [
    (&[[6.0, 72.0], [30.0, 48.0], [40.0, 72.0]], 1),
    (&[[30.0, 48.0], [70.0, 48.0], [60.0, 72.0], [40.0, 72.0]], 3),
    (&[[70.0, 48.0], [94.0, 72.0], [60.0, 72.0]], 2),
    (&[[6.0, 72.0], [40.0, 72.0], [50.0, 156.0]], 2),
    (&[[40.0, 72.0], [60.0, 72.0], [50.0, 156.0]], 3),
    (&[[60.0, 72.0], [94.0, 72.0], [50.0, 156.0]], 4),
];

const SHARD_SPARKS: [(Point, Point, f64); 2] = [([44.0, 34.0], [43.0, 104.0], 3.0), ([22.0, 84.0], [23.0, 108.0], 2.0)];
const GEM_SPARKS: [(Point, Point, f64); 2] = [([36.0, 55.0], [54.0, 55.0], 3.0), ([34.0, 82.0], [42.0, 108.0], 2.0)];

const CURVE_SEGMENTS: usize = 24;

/// The body's drawing in its unit box, back to front: the shapes a scene graph fills and strokes (LunaP §222.2).
pub fn geometry(body: &WorldBody, hue: f64, focused: bool) -> Vec<Shape> {
    match body.shape {
        WorldShape::Planet => planet(body, hue, focused),
        WorldShape::Shards => facets(&SHARD_FACETS, &SHARD_SPARKS, hue, focused),
        WorldShape::Gem => facets(&GEM_FACETS, &GEM_SPARKS, hue, focused),
    }
}

fn planet(body: &WorldBody, hue: f64, focused: bool) -> Vec<Shape> {
    let [light, middle, rim] = colours(body, hue);
    let mut shapes = Vec::with_capacity(12);
    let ring_ink = if focused { Colour::rgba(255, 255, 255, 242) } else { Colour::rgba(255, 245, 252, 179) };
    let ring = match body.ring {
        WorldRing::Big => Some((64.0, 13.0, 0.75, 2.4)),
        WorldRing::Subtle => Some((54.0, 10.0, 0.45, 1.4)),
        WorldRing::None => None,
    };
    if let Some((rx, ry, opacity, width)) = ring {
        let stroke = Stroke { colour: ring_ink, width };
        shapes.push(Shape::stroked(ellipse(50.0, 52.0, rx, ry, -16.0), true, stroke).with_opacity(opacity));
    }
    shapes.push(Shape::filled(
        circle(50.0, 50.0, 44.0),
        Paint::Radial {
            centre: [6.0 + 0.36 * 88.0, 6.0 + 0.32 * 88.0],
            radius: 0.74 * 88.0,
            stops: vec![(0.0, light), (0.48, middle), (1.0, rim)],
        },
    ));
    if body.craters > 0.0 {
        for (cx, cy, r) in [(63.0, 40.0, 7.0), (40.0, 64.0, 5.0), (67.0, 66.0, 3.5)] {
            shapes.push(Shape::filled(circle(cx, cy, r), Paint::Solid(rim)).with_opacity(body.craters));
        }
    }
    if body.bands > 0.0 {
        let high = quadratic([9.0, 44.0], [50.0, 33.0], [91.0, 44.0], CURVE_SEGMENTS);
        let low = quadratic([12.0, 62.0], [50.0, 53.0], [88.0, 62.0], CURVE_SEGMENTS);
        shapes.push(Shape::stroked(high, false, Stroke { colour: light, width: 4.0 }).with_opacity(body.bands));
        shapes.push(Shape::stroked(low, false, Stroke { colour: light, width: 3.0 }).with_opacity(body.bands));
    }
    if body.cap > 0.0 {
        let mut cap = quadratic([33.0, 13.0], [50.0, 4.0], [67.0, 13.0], CURVE_SEGMENTS);
        cap.extend(quadratic([67.0, 13.0], [50.0, 20.0], [33.0, 13.0], CURVE_SEGMENTS).into_iter().skip(1));
        shapes.push(Shape::filled(cap, Paint::Solid(CAP_INK)).with_opacity(body.cap));
    }
    if let Some((rx, ry, opacity, width)) = ring {
        let front = super::shape::arc([50.0, 52.0], (rx, ry), -16.0, 0.0, 180.0, 32);
        shapes.push(Shape::stroked(front, false, Stroke { colour: ring_ink, width }).with_opacity(opacity));
    }
    shapes.push(Shape::filled(ellipse(35.0, 29.0, 10.0, 6.0, -28.0), Paint::Solid(HIGHLIGHT)));
    let moon_edge = Stroke { colour: light, width: 1.0 };
    if body.moons >= 1 {
        shapes.push(Shape::filled(circle(93.0, 14.0, 5.5), Paint::Solid(middle)).with_stroke(moon_edge));
    }
    if body.moons >= 2 {
        shapes.push(Shape::filled(circle(5.0, 84.0, 3.8), Paint::Solid(middle)).with_stroke(moon_edge));
    }
    shapes
}

fn facets(polygons: &[(&[Point], usize)], sparks: &[(Point, Point, f64)], hue: f64, focused: bool) -> Vec<Shape> {
    let inks = facet_inks(hue, focused);
    let edge = Stroke { colour: if focused { Colour::rgba(255, 255, 255, 242) } else { Colour::rgba(255, 245, 252, 140) }, width: 1.2 };
    let spark = if focused { Colour::rgba(255, 255, 255, 242) } else { Colour::rgba(255, 255, 255, 115) };
    let mut shapes: Vec<Shape> =
        polygons.iter().map(|(points, ink)| Shape::filled(points.to_vec(), Paint::Solid(inks[ink - 1])).with_stroke(edge)).collect();
    for (from, to, width) in sparks {
        shapes.push(Shape::stroked(vec![*from, *to], false, Stroke { colour: spark, width: *width }));
    }
    shapes
}
