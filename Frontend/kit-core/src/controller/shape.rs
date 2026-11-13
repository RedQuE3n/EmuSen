//! A drawing's shapes as its file writes them, turned into SVG path data for Qt's `PathSvg` and into a box for placing
//! labels (EmuSen_Kit.md §7.62). Every shape is in the drawing's own design units, y down, angles in degrees clockwise.

use serde_json::Value;

/// A box: left, top, width, height.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Bounds {
    pub fn centre(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    fn of_points(points: &[(f64, f64)]) -> Bounds {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for &(x, y) in points {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        if points.is_empty() { Bounds::default() } else { Bounds { x: x0, y: y0, w: x1 - x0, h: y1 - y0 } }
    }

    fn union(self, other: Bounds) -> Bounds {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        Bounds { x: x0, y: y0, w: (self.x + self.w).max(other.x + other.w) - x0, h: (self.y + self.h).max(other.y + other.h) - y0 }
    }
}

/// One shape of a drawing file.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Circle { cx: f64, cy: f64, r: f64 },
    /// A rectangle with corners rounded by `r`; `ends` rounds only the corners on one side ("up", "down", "left", "right").
    Box { x: f64, y: f64, w: f64, h: f64, r: f64, ends: Option<String> },
    /// A rounded bar centred on a point, `length` along the angle and `thickness` across.
    Pill { cx: f64, cy: f64, length: f64, thickness: f64, angle: f64 },
    /// SVG path data in absolute commands (M, L, H, V, C, Q, A, Z).
    Path(String),
    /// The band of a ring between two angles.
    ArcBand { cx: f64, cy: f64, inner: f64, outer: f64, from: f64, to: f64 },
    Octagon { cx: f64, cy: f64, r: f64 },
    /// A small solid triangle pointing along (dx, dy).
    Arrow { cx: f64, cy: f64, size: f64, dx: f64, dy: f64 },
    /// Several shapes filled as one: drawn stroke first, then fill, so the seams between them do not show.
    Union(Vec<Shape>),
    /// A shape turned about a point.
    Turned { shape: Box<Shape>, angle: f64, cx: f64, cy: f64 },
    /// An outline: the outer shape with the inner cut out of it, filled even-odd.
    Ring(Box<Shape>, Box<Shape>),
}

fn num(v: &Value, i: usize) -> f64 {
    v.get(i).and_then(Value::as_f64).unwrap_or(0.0)
}

/// A number written with at most three decimals, so the path's text is short and stable.
pub(crate) fn n(x: f64) -> String {
    let r = (x * 1000.0).round() / 1000.0;
    if r == r.trunc() { format!("{}", r as i64) } else { format!("{r}") }
}

/// A point turned by `angle` degrees about (cx, cy).
fn turn(p: (f64, f64), angle: f64, cx: f64, cy: f64) -> (f64, f64) {
    let (s, c) = angle.to_radians().sin_cos();
    let (dx, dy) = (p.0 - cx, p.1 - cy);
    (cx + dx * c - dy * s, cy + dx * s + dy * c)
}

impl Shape {
    /// A shape from its file form; an unknown form is an error naming it.
    pub fn parse(v: &Value) -> Result<Shape, String> {
        let shape = if let Some(a) = v.get("circle") {
            Shape::Circle { cx: num(a, 0), cy: num(a, 1), r: num(a, 2) }
        } else if let Some(a) = v.get("box") {
            Shape::Box { x: num(a, 0), y: num(a, 1), w: num(a, 2), h: num(a, 3), r: num(a, 4), ends: v["ends"].as_str().map(str::to_owned) }
        } else if let Some(a) = v.get("pill") {
            Shape::Pill { cx: num(a, 0), cy: num(a, 1), length: num(a, 2), thickness: num(a, 3), angle: num(a, 4) }
        } else if let Some(p) = v.get("path").and_then(Value::as_str) {
            Shape::Path(p.to_owned())
        } else if let Some(a) = v.get("arcband") {
            Shape::ArcBand { cx: num(a, 0), cy: num(a, 1), inner: num(a, 2), outer: num(a, 3), from: num(a, 4), to: num(a, 5) }
        } else if let Some(a) = v.get("octagon") {
            Shape::Octagon { cx: num(a, 0), cy: num(a, 1), r: num(a, 2) }
        } else if let Some(a) = v.get("arrow") {
            Shape::Arrow { cx: num(a, 0), cy: num(a, 1), size: num(a, 2), dx: num(a, 3), dy: num(a, 4) }
        } else if let Some(pair) = v.get("ring").and_then(Value::as_array).filter(|p| p.len() == 2) {
            Shape::Ring(Box::new(Shape::parse(&pair[0])?), Box::new(Shape::parse(&pair[1])?))
        } else if let Some(list) = v.get("union").and_then(Value::as_array) {
            Shape::Union(list.iter().map(Shape::parse).collect::<Result<_, _>>()?)
        } else {
            return Err(format!("a shape of no known form: {v}"));
        };
        Ok(match v.get("turn") {
            Some(t) => Shape::Turned { shape: Box::new(shape), angle: num(t, 0), cx: num(t, 1), cy: num(t, 2) },
            None => shape,
        })
    }

    /// The shape's outline points, before any turn: corners and the extremes of its curves, enough for its box.
    fn points(&self) -> Vec<(f64, f64)> {
        match self {
            Shape::Circle { cx, cy, r } => vec![(cx - r, cy - r), (cx + r, cy + r)],
            Shape::Box { x, y, w, h, .. } => vec![(*x, *y), (x + w, *y), (*x, y + h), (x + w, y + h)],
            Shape::Pill { cx, cy, length, thickness, angle } => {
                let (l, t) = (length / 2.0, thickness / 2.0);
                [(-l, -t), (l, -t), (-l, t), (l, t)].iter().map(|&(dx, dy)| turn((cx + dx, cy + dy), *angle, *cx, *cy)).collect()
            }
            Shape::Path(d) => path_points(d),
            Shape::ArcBand { cx, cy, outer, from, to, inner } => {
                let mut pts = Vec::new();
                let mut a = *from;
                while a <= *to {
                    let (s, c) = a.to_radians().sin_cos();
                    pts.push((cx + outer * c, cy + outer * s));
                    pts.push((cx + inner * c, cy + inner * s));
                    a += 5.0;
                }
                pts
            }
            Shape::Octagon { cx, cy, r } => (0..8)
                .map(|i| {
                    let (s, c) = (22.5 + 45.0 * f64::from(i)).to_radians().sin_cos();
                    (cx + r * c, cy + r * s)
                })
                .collect(),
            Shape::Arrow { .. } => arrow_points(self).to_vec(),
            Shape::Union(parts) => parts.iter().flat_map(Shape::outline).collect(),
            Shape::Turned { shape, angle, cx, cy } => shape.points().into_iter().map(|p| turn(p, *angle, *cx, *cy)).collect(),
            Shape::Ring(outer, _) => outer.points(),
        }
    }

    fn outline(&self) -> Vec<(f64, f64)> {
        self.points()
    }

    /// The shape's box.
    pub fn bounds(&self) -> Bounds {
        match self {
            Shape::Union(parts) => parts.iter().map(Shape::bounds).reduce(Bounds::union).unwrap_or_default(),
            _ => Bounds::of_points(&self.points()),
        }
    }

    /// The shape's parts as path data, one for each shape a union joins.
    pub fn subpaths(&self) -> Vec<String> {
        match self {
            Shape::Union(parts) => parts.iter().flat_map(Shape::subpaths).collect(),
            Shape::Turned { shape, angle, cx, cy } => shape.turned(*angle, *cx, *cy).subpaths(),
            _ => vec![self.svg()],
        }
    }

    /// The shape turned about a point, as a shape of its own form where it has one.
    fn turned(&self, angle: f64, cx: f64, cy: f64) -> Shape {
        match self {
            Shape::Box { x, y, w, h, r, ends: None } => {
                let corners = [(*x, *y), (x + w, *y), (x + w, y + h), (*x, y + h)].map(|p| turn(p, angle, cx, cy));
                Shape::Path(rounded_polygon(&corners, &[*r; 4]))
            }
            Shape::Pill { cx: px, cy: py, length, thickness, angle: own } => {
                let (x, y) = turn((*px, *py), angle, cx, cy);
                Shape::Pill { cx: x, cy: y, length: *length, thickness: *thickness, angle: own + angle }
            }
            Shape::Circle { cx: x, cy: y, r } => {
                let (x, y) = turn((*x, *y), angle, cx, cy);
                Shape::Circle { cx: x, cy: y, r: *r }
            }
            Shape::Union(parts) => Shape::Union(parts.iter().map(|p| p.turned(angle, cx, cy)).collect()),
            other => {
                let pts: Vec<(f64, f64)> = other.points().into_iter().map(|p| turn(p, angle, cx, cy)).collect();
                Shape::Path(polygon(&pts))
            }
        }
    }

    /// The shape as one path's data.
    pub fn svg(&self) -> String {
        match self {
            Shape::Circle { cx, cy, r } => format!(
                "M {} {} A {r} {r} 0 1 1 {} {} A {r} {r} 0 1 1 {} {} Z",
                n(cx - r),
                n(*cy),
                n(cx + r),
                n(*cy),
                n(cx - r),
                n(*cy),
                r = n(*r)
            ),
            Shape::Box { x, y, w, h, r, ends } => {
                let corners = [(*x, *y), (x + w, *y), (x + w, y + h), (*x, y + h)];
                let r = r.min(w / 2.0).min(h / 2.0);
                // Corners: top-left, top-right, bottom-right, bottom-left; an end rounds its side's two.
                let radii = match ends.as_deref() {
                    Some("up") => [r, r, 0.0, 0.0],
                    Some("down") => [0.0, 0.0, r, r],
                    Some("left") => [r, 0.0, 0.0, r],
                    Some("right") => [0.0, r, r, 0.0],
                    _ => [r; 4],
                };
                rounded_polygon(&corners, &radii)
            }
            Shape::Pill { cx, cy, length, thickness, angle } => {
                let t = thickness / 2.0;
                let l = (length / 2.0 - t).max(0.0);
                let p = |dx: f64, dy: f64| turn((cx + dx, cy + dy), *angle, *cx, *cy);
                let (a, b, c, d) = (p(-l, -t), p(l, -t), p(l, t), p(-l, t));
                format!(
                    "M {} {} L {} {} A {t} {t} 0 0 1 {} {} L {} {} A {t} {t} 0 0 1 {} {} Z",
                    n(a.0),
                    n(a.1),
                    n(b.0),
                    n(b.1),
                    n(c.0),
                    n(c.1),
                    n(d.0),
                    n(d.1),
                    n(a.0),
                    n(a.1),
                    t = n(t)
                )
            }
            Shape::Path(d) => d.clone(),
            Shape::ArcBand { cx, cy, inner, outer, from, to } => {
                let p = |r: f64, deg: f64| {
                    let (s, c) = deg.to_radians().sin_cos();
                    (cx + r * c, cy + r * s)
                };
                let (a, b, c, d) = (p(*outer, *from), p(*outer, *to), p(*inner, *to), p(*inner, *from));
                let large = i32::from(to - from > 180.0);
                format!(
                    "M {} {} A {o} {o} 0 {large} 1 {} {} L {} {} A {i} {i} 0 {large} 0 {} {} Z",
                    n(a.0),
                    n(a.1),
                    n(b.0),
                    n(b.1),
                    n(c.0),
                    n(c.1),
                    n(d.0),
                    n(d.1),
                    o = n(*outer),
                    i = n(*inner)
                )
            }
            Shape::Octagon { .. } | Shape::Arrow { .. } => polygon(&self.points_exact()),
            Shape::Union(_) | Shape::Turned { .. } => self.subpaths().join(" "),
            Shape::Ring(outer, inner) => format!("{} {}", outer.svg(), inner.svg()),
        }
    }

    fn points_exact(&self) -> Vec<(f64, f64)> {
        match self {
            Shape::Arrow { .. } => arrow_points(self).to_vec(),
            _ => self.points(),
        }
    }
}

fn arrow_points(shape: &Shape) -> [(f64, f64); 3] {
    let Shape::Arrow { cx, cy, size, dx, dy } = *shape else { return [(0.0, 0.0); 3] };
    let h = size / 2.0;
    [(cx + dx * h, cy + dy * h), (cx - dx * h + dy * h, cy - dy * h - dx * h), (cx - dx * h - dy * h, cy - dy * h + dx * h)]
}

fn polygon(points: &[(f64, f64)]) -> String {
    let mut out = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        out.push_str(if i == 0 { "M " } else { " L " });
        out.push_str(&format!("{} {}", n(*x), n(*y)));
    }
    out.push_str(" Z");
    out
}

/// A closed four-cornered outline with each corner rounded by its own radius, as a circular arc.
fn rounded_polygon(corners: &[(f64, f64); 4], radii: &[f64; 4]) -> String {
    let towards = |a: (f64, f64), b: (f64, f64), d: f64| {
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt().max(1e-9);
        (a.0 + (b.0 - a.0) * d / len, a.1 + (b.1 - a.1) * d / len)
    };
    let mut out = String::new();
    for i in 0..4 {
        let (prev, here, next) = (corners[(i + 3) % 4], corners[i], corners[(i + 1) % 4]);
        let r = radii[i];
        let start = towards(here, prev, r);
        let end = towards(here, next, r);
        out.push_str(if i == 0 { "M " } else { " L " });
        out.push_str(&format!("{} {}", n(start.0), n(start.1)));
        if r > 0.0 {
            out.push_str(&format!(" A {} {} 0 0 1 {} {}", n(r), n(r), n(end.0), n(end.1)));
        }
    }
    out.push_str(" Z");
    out
}

/// The points a path's absolute commands pass through or pull towards: its box, near enough to place a label by.
fn path_points(d: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let mut numbers: Vec<f64> = Vec::new();
    let mut command = ' ';
    let mut current = (0.0, 0.0);
    let flush = |command: char, numbers: &mut Vec<f64>, points: &mut Vec<(f64, f64)>, current: &mut (f64, f64)| {
        let take = |numbers: &[f64], step: usize, f: &mut dyn FnMut(&[f64])| {
            for chunk in numbers.chunks(step).filter(|c| c.len() == step) {
                f(chunk);
            }
        };
        match command {
            'M' | 'L' | 'C' | 'Q' | 'S' | 'T' => take(numbers, 2, &mut |c| {
                points.push((c[0], c[1]));
                *current = (c[0], c[1]);
            }),
            'H' => take(numbers, 1, &mut |c| {
                *current = (c[0], current.1);
                points.push(*current);
            }),
            'V' => take(numbers, 1, &mut |c| {
                *current = (current.0, c[0]);
                points.push(*current);
            }),
            'A' => take(numbers, 7, &mut |c| {
                points.push((c[5], c[6]));
                *current = (c[5], c[6]);
            }),
            _ => {}
        }
        numbers.clear();
    };
    let mut word = String::new();
    for ch in d.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_alphabetic() && ch != 'e' && ch != 'E' {
            if let Ok(x) = word.parse() {
                numbers.push(x);
            }
            word.clear();
            flush(command, &mut numbers, &mut points, &mut current);
            command = ch;
        } else if ch == ',' || ch.is_whitespace() || (ch == '-' && !word.is_empty() && !word.ends_with(['e', 'E'])) {
            if let Ok(x) = word.parse() {
                numbers.push(x);
            }
            word.clear();
            if ch == '-' {
                word.push(ch);
            }
        } else {
            word.push(ch);
        }
    }
    flush(command, &mut numbers, &mut points, &mut current);
    points
}

/// A colour darkened by a share, keeping its alpha: `#RRGGBB` or `#AARRGGBB` in, the same form out.
pub fn darken(colour: &str, by: f64) -> String {
    let hex = colour.trim_start_matches('#');
    let (alpha, rgb) = if hex.len() == 8 { (&hex[..2], &hex[2..]) } else { ("", hex) };
    let channel = |i: usize| u8::from_str_radix(rgb.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    let d = |c: u8| (f64::from(c) * (1.0 - by)) as u8;
    format!("#{alpha}{:02X}{:02X}{:02X}", d(channel(0)), d(channel(2)), d(channel(4)))
}

/// A colour with an alpha of its own, `#AARRGGBB`.
pub fn with_alpha(colour: &str, alpha: u8) -> String {
    let hex = colour.trim_start_matches('#');
    let rgb = if hex.len() == 8 { &hex[2..] } else { hex };
    format!("#{alpha:02X}{rgb}")
}
