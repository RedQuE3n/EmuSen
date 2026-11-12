//! `marks`: the built-in icons a theme does not supply, as polylines in a unit box, translated from LunaP's own
//! drawings (BadgeGlyphDrawing, StarRating, TextRowList, InfoLine, DeviceStatusBar; EmuSen_Qt.md §4.5.5). One colour,
//! the element's: a painter draws each part filled or stroked in it.

use std::f64::consts::PI;

/// A point in the unit box: x right, y down.
pub type Point = [f64; 2];

/// One drawn part: its rings (the first the outline, the rest holes, filled even-odd), closed or open, filled or stroked.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub rings: Vec<Vec<Point>>,
    pub closed: bool,
    pub fill: bool,
    /// The stroke's width as a fraction of the box's height; 0 draws no stroke.
    pub stroke: f64,
}

/// A built-in icon: its parts, its width over its height (the unit box is stretched to that shape), and the share of
/// its height down to the baseline it sits on beside words (the whole of it for a square mark).
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub aspect: f64,
    pub parts: Vec<Part>,
    pub baseline: f64,
}

/// An info icon's baseline: 0.85 of the em above it and 0.15 below.
const INFO_BASELINE: f64 = 0.85;

/// Every built-in icon the scene names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkId {
    /// The list's markers, 0.87 of the font size in a square.
    Star,
    Folder,
    Tick,
    /// gamelistinfo's icons, an em high with the baseline 0.85 down the box.
    InfoGamepad,
    InfoStar,
    InfoFunnel,
    InfoFolder,
    /// systemstatus's icons; the battery with 0 to 4 bars.
    Bluetooth,
    Wifi,
    Cellular,
    Battery(u8),
    /// The nine badges of THEMES.md's slots, and the link drawn over a linked folder.
    BadgeFavorite,
    BadgeCompleted,
    BadgeKidGame,
    BadgeBroken,
    BadgeController,
    BadgeAltEmulator,
    BadgeCollection,
    BadgeFolder,
    BadgeManual,
    FolderLink,
    /// The controller shapes the controller badge carries.
    ControllerUnknown,
    ControllerGamepad,
    ControllerNes,
    ControllerSnes,
    ControllerN64,
}

impl MarkId {
    /// Every mark, the battery once per level.
    pub const ALL: [MarkId; 30] = [
        MarkId::Star,
        MarkId::Folder,
        MarkId::Tick,
        MarkId::InfoGamepad,
        MarkId::InfoStar,
        MarkId::InfoFunnel,
        MarkId::InfoFolder,
        MarkId::Bluetooth,
        MarkId::Wifi,
        MarkId::Cellular,
        MarkId::Battery(0),
        MarkId::Battery(1),
        MarkId::Battery(2),
        MarkId::Battery(3),
        MarkId::Battery(4),
        MarkId::BadgeFavorite,
        MarkId::BadgeCompleted,
        MarkId::BadgeKidGame,
        MarkId::BadgeBroken,
        MarkId::BadgeController,
        MarkId::BadgeAltEmulator,
        MarkId::BadgeCollection,
        MarkId::BadgeFolder,
        MarkId::BadgeManual,
        MarkId::FolderLink,
        MarkId::ControllerUnknown,
        MarkId::ControllerGamepad,
        MarkId::ControllerNes,
        MarkId::ControllerSnes,
        MarkId::ControllerN64,
    ];

    /// The name a painter and a test know the mark by.
    pub fn name(self) -> String {
        match self {
            MarkId::Battery(bars) => format!("battery{bars}"),
            other => {
                let debug = format!("{other:?}");
                let mut out = String::new();
                for (i, c) in debug.chars().enumerate() {
                    if c.is_uppercase() && i > 0 {
                        out.push('-');
                    }
                    out.extend(c.to_lowercase());
                }
                out
            }
        }
    }

    pub fn mark(self) -> Mark {
        match self {
            MarkId::Star => square(vec![fill(vec![star_ring([0.5, 0.5], 0.5, 0.4)])]),
            MarkId::Folder => {
                square(vec![fill(vec![poly(&[(0.0, 0.15), (0.3, 0.15), (0.39, 0.28), (1.0, 0.28), (1.0, 0.9), (0.0, 0.9)])])])
            }
            MarkId::Tick => square(vec![line(poly(&[(0.08, 0.55), (0.38, 0.85), (0.94, 0.15)]), 1.0 / 6.0)]),
            MarkId::InfoGamepad => info(1.06, gamepad_info()),
            MarkId::InfoStar => info(0.92, vec![fill(vec![em(&star_points([0.46, -0.35], 0.48, 0.24))])]),
            MarkId::InfoFunnel => {
                info(0.78, vec![fill(vec![em(&[[0.0, -0.72], [0.78, -0.72], [0.47, -0.36], [0.47, 0.06], [0.31, -0.04], [0.31, -0.36]])])])
            }
            MarkId::InfoFolder => info(
                1.06,
                vec![
                    fill(vec![em(&[
                        [0.0, -0.78],
                        [0.36, -0.78],
                        [0.44, -0.68],
                        [0.86, -0.68],
                        [0.86, -0.46],
                        [0.26, -0.46],
                        [0.09, -0.08],
                        [0.0, -0.08],
                    ])]),
                    fill(vec![em(&[[0.3, -0.39], [1.06, -0.39], [0.82, 0.0], [0.05, 0.0]])]),
                ],
            ),
            MarkId::Bluetooth => square(bluetooth()),
            MarkId::Wifi => square(wifi()),
            MarkId::Cellular => square(
                (0..4)
                    .map(|i| {
                        let x = 0.14 + i as f64 * 0.214;
                        fill(vec![rect(x, 0.8 - 0.18 * (i + 1) as f64, 0.095, 0.18 * (i + 1) as f64)])
                    })
                    .collect(),
            ),
            MarkId::Battery(bars) => square(battery(bars.min(4))),
            MarkId::FolderLink => square(link()),
            MarkId::ControllerUnknown
            | MarkId::ControllerGamepad
            | MarkId::ControllerNes
            | MarkId::ControllerSnes
            | MarkId::ControllerN64 => square(controller(self)),
            badge => square(badge_parts(badge)),
        }
    }
}

fn square(parts: Vec<Part>) -> Mark {
    Mark { aspect: 1.0, parts, baseline: 1.0 }
}

// An info icon: em units across `width` and from 0.85 above the baseline to 0.15 below, to the unit box.
fn info(width: f64, parts: Vec<Part>) -> Mark {
    let parts = parts
        .into_iter()
        .map(|mut p| {
            for ring in &mut p.rings {
                for q in ring.iter_mut() {
                    *q = [q[0] / width, q[1] + INFO_BASELINE];
                }
            }
            p
        })
        .collect();
    Mark { aspect: width, parts, baseline: INFO_BASELINE }
}

fn em(points: &[Point]) -> Vec<Point> {
    points.to_vec()
}

fn fill(rings: Vec<Vec<Point>>) -> Part {
    Part { rings, closed: true, fill: true, stroke: 0.0 }
}

fn outline(ring: Vec<Point>, stroke: f64) -> Part {
    Part { rings: vec![ring], closed: true, fill: false, stroke }
}

fn line(points: Vec<Point>, stroke: f64) -> Part {
    Part { rings: vec![points], closed: false, fill: false, stroke }
}

fn poly(points: &[(f64, f64)]) -> Vec<Point> {
    points.iter().map(|&(x, y)| [x, y]).collect()
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Vec<Point> {
    vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

/// A rounded rectangle flattened, each corner in eight segments.
pub fn rounded(x: f64, y: f64, w: f64, h: f64, r: f64) -> Vec<Point> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut out = vec![];
    for (cx, cy, from) in [(x + w - r, y + r, -90.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0), (x + r, y + r, 180.0)] {
        out.extend(arc([cx, cy], r, r, from, from + 90.0, 8));
    }
    out
}

/// An arc of an ellipse as points, angles in degrees from the right, clockwise on a screen whose y runs down.
pub fn arc(c: Point, rx: f64, ry: f64, from: f64, to: f64, segments: usize) -> Vec<Point> {
    (0..=segments)
        .map(|i| {
            let a = (from + (to - from) * i as f64 / segments as f64).to_radians();
            [c[0] + rx * a.cos(), c[1] + ry * a.sin()]
        })
        .collect()
}

fn circle(c: Point, r: f64) -> Vec<Point> {
    let mut ring = arc(c, r, r, 0.0, 360.0, 32);
    ring.pop();
    ring
}

fn star_points(c: Point, outer: f64, inner: f64) -> Vec<Point> {
    (0..10)
        .map(|i| {
            let a = -PI / 2.0 + i as f64 * PI / 5.0;
            let r = if i % 2 == 0 { outer } else { inner };
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

// A five-pointed star in a circle of radius r, its inner radius `ratio` of the outer.
fn star_ring(c: Point, r: f64, ratio: f64) -> Vec<Point> {
    star_points(c, r, r * ratio)
}

fn gamepad_info() -> Vec<Part> {
    let body = rounded(0.0, -0.58, 1.06, 0.58, 0.29);
    vec![fill(vec![
        body,
        plus((0.2, 0.42, -0.33, -0.25), (0.27, 0.35, -0.4, -0.18)),
        circle([0.71, -0.35], 0.05),
        circle([0.81, -0.23], 0.05),
    ])]
}

// The outline of a plus: a horizontal bar (x0, x1, y0, y1) across a vertical one, as one ring.
fn plus(h: (f64, f64, f64, f64), v: (f64, f64, f64, f64)) -> Vec<Point> {
    let ((hx0, hx1, hy0, hy1), (vx0, vx1, vy0, vy1)) = (h, v);
    poly(&[
        (vx0, vy0),
        (vx1, vy0),
        (vx1, hy0),
        (hx1, hy0),
        (hx1, hy1),
        (vx1, hy1),
        (vx1, vy1),
        (vx0, vy1),
        (vx0, hy1),
        (hx0, hy1),
        (hx0, hy0),
        (vx0, hy0),
    ])
}

fn wifi() -> Vec<Part> {
    let (stroke, c) = (0.08, [0.5, 0.8]);
    let mut parts: Vec<Part> =
        [0.62, 0.43, 0.26].iter().map(|r| line(arc(c, r - stroke / 2.0, r - stroke / 2.0, -135.0, -45.0, 12), stroke)).collect();
    parts.push(fill(vec![circle(c, 0.05)]));
    parts
}

fn bluetooth() -> Vec<Part> {
    let stroke = 0.06;
    let (x, y, w, h) = (0.5 - 0.27 + stroke / 2.0, 0.5 - 0.385 + stroke / 2.0, 0.54 - stroke, 0.77 - stroke);
    let p = |fx: f64, fy: f64| [x + fx * w, y + fy * h];
    vec![line(vec![p(0.05, 0.25), p(0.95, 0.75), p(0.5, 1.0), p(0.5, 0.0), p(0.95, 0.25), p(0.05, 0.75)], stroke)]
}

fn battery(bars: u8) -> Vec<Part> {
    let mut parts = vec![fill(vec![rect(0.38, 0.0, 0.24, 0.06)]), outline(rounded(0.285, 0.145, 0.43, 0.67, 0.04), 0.05)];
    for i in 0..bars {
        parts.push(fill(vec![rect(0.36, 0.71 - i as f64 * 0.14, 0.28, 0.05)]));
    }
    parts
}

// A point of a badge drawing: offsets from the box's centre in multiples of its side.
fn b(x: f64, y: f64) -> Point {
    [0.5 + x, 0.5 + y]
}

fn bline(points: &[(f64, f64)], stroke: f64) -> Part {
    line(points.iter().map(|&(x, y)| b(x, y)).collect(), stroke)
}

const PEN: f64 = 0.075;
const THIN: f64 = 0.055;

fn badge_parts(kind: MarkId) -> Vec<Part> {
    let mut parts = vec![outline(rounded(0.07, 0.07, 0.86, 0.86, 0.16), PEN)];
    match kind {
        MarkId::BadgeFavorite => parts.push(fill(vec![star_ring([0.5, 0.5], 0.28, 0.4)])),
        MarkId::BadgeCompleted => parts.push(bline(&[(-0.2, 0.0), (-0.05, 0.16), (0.22, -0.16)], PEN)),
        MarkId::BadgeKidGame => {
            parts.push(outline(circle([0.5, 0.5], 0.25), THIN));
            parts.push(fill(vec![circle(b(-0.09, -0.06), 0.035)]));
            parts.push(fill(vec![circle(b(0.09, -0.06), 0.035)]));
            parts.push(line(arc(b(0.0, 0.02), 0.13, 0.13, 20.0, 160.0, 12), THIN));
        }
        MarkId::BadgeBroken => parts.push(bline(&[(0.04, -0.3), (-0.08, -0.08), (0.08, 0.02), (-0.05, 0.3)], PEN)),
        MarkId::BadgeAltEmulator => {
            parts.push(bline(&[(-0.22, -0.09), (0.2, -0.09)], THIN));
            parts.push(bline(&[(0.1, -0.18), (0.22, -0.09), (0.1, 0.0)], THIN));
            parts.push(bline(&[(0.22, 0.11), (-0.2, 0.11)], THIN));
            parts.push(bline(&[(-0.1, 0.02), (-0.22, 0.11), (-0.1, 0.2)], THIN));
        }
        MarkId::BadgeCollection => {
            parts.push(outline(rounded(0.35, 0.26, 0.36, 0.3, 0.04), THIN));
            parts.push(fill(vec![rounded(0.28, 0.42, 0.36, 0.3, 0.04)]));
        }
        MarkId::BadgeFolder => {
            parts.push(bline(&[(-0.26, 0.2), (-0.26, -0.2), (-0.08, -0.2), (-0.02, -0.12), (0.26, -0.12), (0.26, 0.2), (-0.26, 0.2)], THIN))
        }
        MarkId::BadgeManual => parts.push(bline(
            &[
                (0.0, -0.14),
                (-0.1, -0.2),
                (-0.26, -0.2),
                (-0.26, 0.18),
                (-0.1, 0.18),
                (0.0, 0.24),
                (0.1, 0.18),
                (0.26, 0.18),
                (0.26, -0.2),
                (0.1, -0.2),
                (0.0, -0.14),
                (0.0, 0.24),
            ],
            THIN,
        )),
        _ => {}
    }
    parts
}

// Two rounded links overlapping on the rising diagonal, turned a quarter of a half turn anticlockwise about the centre.
fn link() -> Vec<Part> {
    let turn = |ring: Vec<Point>| -> Vec<Point> {
        let (s, c) = (-PI / 4.0).sin_cos();
        ring.into_iter().map(|[x, y]| [0.5 + (x - 0.5) * c - (y - 0.5) * s, 0.5 + (x - 0.5) * s + (y - 0.5) * c]).collect()
    };
    vec![outline(turn(rounded(0.1, 0.36, 0.48, 0.28, 0.14)), PEN), outline(turn(rounded(0.42, 0.36, 0.48, 0.28, 0.14)), PEN)]
}

// A pad's directional cross: two bars of a third of its size.
fn cross(c: Point, size: f64) -> Part {
    let (half, arm) = (size / 2.0, size * 0.17);
    fill(vec![plus((c[0] - half, c[0] + half, c[1] - arm, c[1] + arm), (c[0] - arm, c[0] + arm, c[1] - half, c[1] + half))])
}

// An SVG-style elliptical arc from one point to another, flattened, clockwise and the short way.
fn arc_to(from: Point, to: Point, rx: f64, ry: f64) -> Vec<Point> {
    let (dx, dy) = ((from[0] - to[0]) / 2.0, (from[1] - to[1]) / 2.0);
    let lambda = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry);
    let (rx, ry) = if lambda > 1.0 { (rx * lambda.sqrt(), ry * lambda.sqrt()) } else { (rx, ry) };
    let num = (rx * rx * ry * ry - rx * rx * dy * dy - ry * ry * dx * dx).max(0.0);
    let den = rx * rx * dy * dy + ry * ry * dx * dx;
    // Small arc, clockwise sweep: the centre's sign is the negative of large-arc == sweep.
    let k = (num / den).sqrt();
    let (cxp, cyp) = (k * rx * dy / ry, -k * ry * dx / rx);
    let c = [cxp + (from[0] + to[0]) / 2.0, cyp + (from[1] + to[1]) / 2.0];
    let a0 = ((from[1] - c[1]) / ry).atan2((from[0] - c[0]) / rx);
    let mut a1 = ((to[1] - c[1]) / ry).atan2((to[0] - c[0]) / rx);
    while a1 < a0 {
        a1 += 2.0 * PI;
    }
    arc(c, rx, ry, a0.to_degrees(), a1.to_degrees(), 16)
}

fn controller(kind: MarkId) -> Vec<Part> {
    match kind {
        MarkId::ControllerNes => vec![
            outline(rounded(0.05, 0.31, 0.9, 0.38, 0.03), PEN),
            cross(b(-0.26, 0.0), 0.2),
            fill(vec![rect(0.41, 0.54, 0.07, 0.035)]),
            fill(vec![rect(0.52, 0.54, 0.07, 0.035)]),
            fill(vec![circle(b(0.2, 0.04), 0.05)]),
            fill(vec![circle(b(0.33, 0.04), 0.05)]),
        ],
        MarkId::ControllerSnes => {
            let mut parts = vec![outline(rounded(0.04, 0.3, 0.92, 0.4, 0.2), PEN), cross(b(-0.26, 0.0), 0.19)];
            for (x, y) in [(0.26, -0.09), (0.26, 0.09), (0.17, 0.0), (0.35, 0.0)] {
                parts.push(fill(vec![circle(b(x, y), 0.04)]));
            }
            parts.push(bline(&[(-0.07, 0.05), (-0.02, 0.0)], THIN));
            parts.push(bline(&[(0.02, 0.05), (0.07, 0.0)], THIN));
            parts
        }
        MarkId::ControllerN64 => {
            let body = [
                (-0.44, -0.18),
                (0.44, -0.18),
                (0.44, 0.02),
                (0.3, 0.38),
                (0.2, 0.38),
                (0.12, 0.08),
                (0.08, 0.08),
                (0.06, 0.4),
                (-0.06, 0.4),
                (-0.08, 0.08),
                (-0.12, 0.08),
                (-0.2, 0.38),
                (-0.3, 0.38),
                (-0.44, 0.02),
            ];
            vec![
                outline(body.iter().map(|&(x, y)| b(x, y)).collect(), THIN),
                cross(b(-0.27, -0.06), 0.15),
                fill(vec![circle(b(0.0, 0.02), 0.055)]),
                fill(vec![circle(b(0.24, -0.03), 0.045)]),
                fill(vec![circle(b(0.33, -0.1), 0.045)]),
            ]
        }
        _ => {
            let mut pad = vec![b(-0.3, -0.2), b(0.3, -0.2)];
            pad.extend(arc_to(b(0.3, -0.2), b(0.34, 0.3), 0.18, 0.26).into_iter().skip(1));
            pad.extend([b(0.12, 0.12), b(-0.12, 0.12), b(-0.34, 0.3)]);
            pad.extend(arc_to(b(-0.34, 0.3), b(-0.3, -0.2), 0.18, 0.26).into_iter().skip(1));
            pad.pop();
            let mut parts = vec![outline(pad, THIN)];
            if kind == MarkId::ControllerGamepad {
                parts.push(fill(vec![circle(b(-0.2, -0.03), 0.07)]));
                parts.push(fill(vec![circle(b(0.2, -0.03), 0.07)]));
            } else {
                // A question mark stroked: a hook over the stem and a dot, where LunaP sets the font's "?" glyph.
                let mut hook = arc(b(0.0, -0.11), 0.08, 0.07, 180.0, 405.0, 12);
                hook.push(b(0.0, 0.03));
                parts.push(line(hook, THIN));
                parts.push(fill(vec![circle(b(0.0, 0.1), 0.03)]));
            }
            parts
        }
    }
}
