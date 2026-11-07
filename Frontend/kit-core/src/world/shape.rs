use crate::tokens::Colour;

/// A point in a drawing's own units.
pub type Point = [f64; 2];

/// A box: x, y, width, height.
pub type Box2 = [f64; 4];

/// What fills a shape: one colour, or a gradient the scene graph shades.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(Colour),
    Radial { centre: Point, radius: f64, stops: Vec<(f64, Colour)> },
    Linear { from: Point, to: Point, stops: Vec<(f64, Colour)> },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub colour: Colour,
    pub width: f64,
}

/// One drawn shape: its outline as vertices, closed or open, a fill, a stroke and an opacity over both.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub points: Vec<Point>,
    pub closed: bool,
    pub fill: Option<Paint>,
    pub stroke: Option<Stroke>,
    pub opacity: f64,
}

impl Shape {
    pub fn filled(points: Vec<Point>, paint: Paint) -> Shape {
        Shape { points, closed: true, fill: Some(paint), stroke: None, opacity: 1.0 }
    }

    pub fn stroked(points: Vec<Point>, closed: bool, stroke: Stroke) -> Shape {
        Shape { points, closed, fill: None, stroke: Some(stroke), opacity: 1.0 }
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Shape {
        self.stroke = Some(stroke);
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Shape {
        self.opacity = opacity;
        self
    }

    /// The smallest box round every vertex, ignoring stroke width.
    pub fn bounds(&self) -> Box2 {
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for p in &self.points {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]]
    }
}

/// How many segments a full ellipse is flattened into.
pub const ELLIPSE_SEGMENTS: usize = 64;

fn rotate(p: Point, about: Point, degrees: f64) -> Point {
    let (s, c) = degrees.to_radians().sin_cos();
    let (dx, dy) = (p[0] - about[0], p[1] - about[1]);
    [about[0] + dx * c - dy * s, about[1] + dx * s + dy * c]
}

/// An arc of an ellipse as an open polyline: angles in degrees, 0 at the right, growing clockwise on a screen whose y runs down; the whole is turned about the centre.
pub fn arc(centre: Point, radii: (f64, f64), rotation: f64, from: f64, to: f64, segments: usize) -> Vec<Point> {
    let n = segments.max(1);
    (0..=n)
        .map(|i| {
            let a = (from + (to - from) * i as f64 / n as f64).to_radians();
            rotate([centre[0] + radii.0 * a.cos(), centre[1] + radii.1 * a.sin()], centre, rotation)
        })
        .collect()
}

/// A whole ellipse, turned about its centre, as a closed polygon.
pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, rotation: f64) -> Vec<Point> {
    let mut points = arc([cx, cy], (rx, ry), rotation, 0.0, 360.0, ELLIPSE_SEGMENTS);
    points.pop();
    points
}

pub fn circle(cx: f64, cy: f64, r: f64) -> Vec<Point> {
    ellipse(cx, cy, r, r, 0.0)
}

/// A quadratic curve from `p0` through control `c` to `p1`, flattened.
pub fn quadratic(p0: Point, c: Point, p1: Point, segments: usize) -> Vec<Point> {
    let n = segments.max(1);
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            [u * u * p0[0] + 2.0 * u * t * c[0] + t * t * p1[0], u * u * p0[1] + 2.0 * u * t * c[1] + t * t * p1[1]]
        })
        .collect()
}

/// Every point scaled from the origin and moved, so a drawing in its own box lands in a host's.
pub fn transform(points: &[Point], scale_x: f64, scale_y: f64, dx: f64, dy: f64) -> Vec<Point> {
    points.iter().map(|p| [p[0] * scale_x + dx, p[1] * scale_y + dy]).collect()
}

/// A shape moved into a box: its unit box of `unit_w` by `unit_h` stretched to the box.
pub fn into_box(shape: &Shape, unit_w: f64, unit_h: f64, target: Box2) -> Shape {
    let (sx, sy) = (target[2] / unit_w, target[3] / unit_h);
    let points = transform(&shape.points, sx, sy, target[0], target[1]);
    let fill = shape.fill.as_ref().map(|paint| match paint {
        Paint::Solid(c) => Paint::Solid(*c),
        Paint::Radial { centre, radius, stops } => Paint::Radial {
            centre: [centre[0] * sx + target[0], centre[1] * sy + target[1]],
            radius: radius * sx.min(sy),
            stops: stops.clone(),
        },
        Paint::Linear { from, to, stops } => Paint::Linear {
            from: [from[0] * sx + target[0], from[1] * sy + target[1]],
            to: [to[0] * sx + target[0], to[1] * sy + target[1]],
            stops: stops.clone(),
        },
    });
    let stroke = shape.stroke.map(|s| Stroke { colour: s.colour, width: s.width * sx.min(sy) });
    Shape { points, closed: shape.closed, fill, stroke, opacity: shape.opacity }
}
