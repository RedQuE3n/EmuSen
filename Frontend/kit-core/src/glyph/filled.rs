//! The filled set: each control one solid shape with its letter, symbol or mark cut out, as ES-DE's help draws its
//! buttons (EmuSen_BigPicture.md §40.1, EmuSen_Kit.md §7.28). The holes of a shape never overlap one another or its
//! letter, since the shape is filled odd-even; a stroked symbol is cut as the band between two outlines.

use super::{at, circle, closed, direction, face_letter, plus, rect, rounded, shoulder_label, sides, Glyph, Letter, Mark, PadFamily, Pair, C};
use crate::input::PadControl;
use crate::world::Point;

/// The disc's radius in the box: 22 pixels beside a 26-pixel label, as ES-DE's help bar measured (§40.1).
const R: f64 = 0.425;
/// Half the width of a stroke cut out of a shape.
const CUT: f64 = 0.0425;
/// A face letter cut out of its disc, and a shoulder's two letters out of its body, at these shares of the glyph type.
const FACE_SCALE: f64 = 0.8;
const SHOULDER_SCALE: f64 = 0.65;

/// The filled glyph of a control in a family already lettered (`super::lettered`).
pub(super) fn glyph(family: PadFamily, control: PadControl) -> Glyph {
    match control {
        PadControl::South | PadControl::East | PadControl::West | PadControl::North => face(family, control),
        PadControl::Up | PadControl::Down | PadControl::Left | PadControl::Right => dpad(control),
        PadControl::LeftShoulder | PadControl::RightShoulder | PadControl::LeftTrigger | PadControl::RightTrigger => {
            shoulder(family, control)
        }
        PadControl::Start | PadControl::Select => middle(family, control == PadControl::Start),
        PadControl::LeftStick | PadControl::RightStick => body(disc(), ring(circle(C, 0.27 + CUT), circle(C, 0.27 - CUT))),
        PadControl::Guide => body(disc(), vec![circle(C, 0.16)]),
    }
}

fn disc() -> Vec<Point> {
    circle(C, R)
}

fn body(points: Vec<Point>, holes: Vec<Vec<Point>>) -> Glyph {
    Glyph { marks: vec![Mark { points, filled: true, stroke: 0.0, cut: false, holes }], letter: None }
}

fn lettered_body(points: Vec<Point>, text: &'static str, scale: f64) -> Glyph {
    Glyph {
        marks: vec![Mark { points, filled: true, stroke: 0.0, cut: true, holes: Vec::new() }],
        letter: Some(Letter { text, scale, bold: true }),
    }
}

/// A stroked outline cut out: the outer and the inner edge of its band.
fn ring(outer: Vec<Point>, inner: Vec<Point>) -> Vec<Vec<Point>> {
    vec![outer, inner]
}

/// Points turned about the centre by `angle` radians.
fn turned(points: Vec<Point>, angle: f64) -> Vec<Point> {
    let (sin, cos) = angle.sin_cos();
    points.into_iter().map(|[x, y]| [C[0] + (x - C[0]) * cos - (y - C[1]) * sin, C[1] + (x - C[0]) * sin + (y - C[1]) * cos]).collect()
}

/// A triangle grown (`d` > 0) or shrunk by `d` along every edge's normal: scaled about its incentre.
fn offset_triangle(t: [Point; 3], d: f64) -> Vec<Point> {
    let side = |a: Point, b: Point| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    let (a, b, c) = (side(t[1], t[2]), side(t[0], t[2]), side(t[0], t[1]));
    let perimeter = a + b + c;
    let centre = [(a * t[0][0] + b * t[1][0] + c * t[2][0]) / perimeter, (a * t[0][1] + b * t[1][1] + c * t[2][1]) / perimeter];
    let area = ((t[1][0] - t[0][0]) * (t[2][1] - t[0][1]) - (t[2][0] - t[0][0]) * (t[1][1] - t[0][1])).abs() / 2.0;
    let k = (2.0 * area / perimeter + d) / (2.0 * area / perimeter);
    closed(t.iter().map(|p| [centre[0] + (p[0] - centre[0]) * k, centre[1] + (p[1] - centre[1]) * k]).collect())
}

/// Bars side by side about the centre, `count` of them `pitch` apart, each `length` long and a cut's width thick.
fn bars(vertical: bool, count: i32, length: f64, pitch: f64) -> Vec<Vec<Point>> {
    (0..count)
        .map(|i| {
            let off = (f64::from(i) - f64::from(count - 1) / 2.0) * pitch;
            if vertical {
                rect(C[0] + off - CUT, C[1] - length / 2.0, 2.0 * CUT, length)
            } else {
                rect(C[0] - length / 2.0, C[1] + off - CUT, length, 2.0 * CUT)
            }
        })
        .collect()
}

fn face(family: PadFamily, control: PadControl) -> Glyph {
    if family != PadFamily::PlayStation {
        return lettered_body(disc(), face_letter(family, control).unwrap_or("?"), FACE_SCALE);
    }
    let r = 0.19;
    let holes = match control {
        PadControl::South => vec![turned(plus(CUT, r * std::f64::consts::SQRT_2), std::f64::consts::FRAC_PI_4)],
        PadControl::East => ring(circle(C, r + CUT), circle(C, r - CUT)),
        PadControl::West => {
            let h = 0.9 * r;
            ring(rect(C[0] - h - CUT, C[1] - h - CUT, 2.0 * (h + CUT), 2.0 * (h + CUT)), rect(C[0] - h + CUT, C[1] - h + CUT, 2.0 * (h - CUT), 2.0 * (h - CUT)))
        }
        _ => {
            let t = [at(0.0, -1.05 * r), at(r, 0.7 * r), at(-r, 0.7 * r)];
            ring(offset_triangle(t, CUT), offset_triangle(t, -CUT))
        }
    };
    body(disc(), holes)
}

const DPAD_HALF: f64 = 0.16;
const DPAD_REACH: f64 = 0.46;

/// A solid plus with an arrow cut into the tip of the arm the press moves along.
fn dpad(control: PadControl) -> Glyph {
    body(plus(DPAD_HALF, DPAD_REACH), vec![arrow(control)])
}

/// The arrow cut into the tip of a direction's arm.
fn arrow(control: PadControl) -> Vec<Point> {
    let (dx, dy) = direction(control);
    let (tip, base, wide) = (DPAD_REACH - 0.05, DPAD_REACH - 0.2, 0.1);
    closed(vec![at(dx * tip, dy * tip), at(dx * base - dy * wide, dy * base + dx * wide), at(dx * base + dy * wide, dy * base - dx * wide)])
}

/// A pair as one glyph: the solid plus with an arrow cut into both tips of the axis, or the two bodies side by side.
pub(super) fn paired(pair: Pair) -> Glyph {
    match pair {
        Pair::Axis(a, b) => body(plus(DPAD_HALF, DPAD_REACH), vec![arrow(a), arrow(b)]),
        Pair::Pad => body(plus(DPAD_HALF, DPAD_REACH), [PadControl::Up, PadControl::Down, PadControl::Left, PadControl::Right].map(arrow).to_vec()),
        Pair::Sides { trigger } => {
            Glyph { marks: sides(trigger).into_iter().map(|points| Mark { points, filled: true, stroke: 0.0, cut: false, holes: Vec::new() }).collect(), letter: None }
        }
    }
}

/// A shoulder's body rounded at the top, a trigger's taller and rounded at the bottom, its family's name cut out.
fn shoulder(family: PadFamily, control: PadControl) -> Glyph {
    let trigger = matches!(control, PadControl::LeftTrigger | PadControl::RightTrigger);
    let (w, h, r) = (0.9, if trigger { 0.78 } else { 0.6 }, 0.2);
    let (top, bottom) = if trigger { (0.05, r) } else { (r, 0.05) };
    lettered_body(rounded(C[0] - w / 2.0, C[1] - h / 2.0, w, h, top, bottom), shoulder_label(family, control), SHOULDER_SCALE)
}

/// The middle buttons: a disc for the families whose marks sit in a circle, a pill for PlayStation, each mark cut out.
fn middle(family: PadFamily, start: bool) -> Glyph {
    match family {
        PadFamily::Nintendo => {
            let (arm, bar) = (0.22, 0.075);
            let mark = if start { plus(bar, arm) } else { rect(C[0] - arm, C[1] - bar, 2.0 * arm, 2.0 * bar) };
            body(disc(), vec![mark])
        }
        PadFamily::PlayStation => {
            let pill = rounded(C[0] - 0.46, C[1] - 0.27, 0.92, 0.54, 0.27, 0.27);
            let holes = if start { bars(false, 3, 0.34, 0.12) } else { bars(true, 3, 0.24, 0.13) };
            body(pill, holes)
        }
        _ if start => body(disc(), bars(false, 3, 0.4, 0.13)),
        _ => body(disc(), two_squares()),
    }
}

/// The view mark: a square outlined behind a solid one, the band of the back one stopping short of the front.
fn two_squares() -> Vec<Vec<Point>> {
    let (x0, y0, x1, y1, t) = (-0.22, -0.2, 0.06, 0.06, 0.07);
    let (fx, fy, gap) = (-0.04, -0.04, 0.035);
    let back = closed(vec![
        at(x0, y0),
        at(x1, y0),
        at(x1, fy - gap),
        at(x1 - t, fy - gap),
        at(x1 - t, y0 + t),
        at(x0 + t, y0 + t),
        at(x0 + t, y1 - t),
        at(fx - gap, y1 - t),
        at(fx - gap, y1),
        at(x0, y1),
    ]);
    vec![back, rect(C[0] + fx, C[1] + fy, 0.28, 0.22)]
}
