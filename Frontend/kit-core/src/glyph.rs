//! A pad's buttons drawn, one set per family, as plain marks in a unit box (EmuSen_Kit.md §7.12): rings, discs, lines,
//! outlines and the words a family prints, never a vendor's artwork. The set is LunaP's outlined set (LunaP.md §103.1),
//! whose owner this module is from stage 0 on.

use crate::input::PadControl;
use crate::world::Point;

mod filled;

pub use filled::DISC_SHARE as FILLED_DISC_SHARE;

/// What a pad prints on its buttons; `Generic` is a pad whose printing is not known, drawn by position as the RetroPad names its buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PadFamily {
    Generic,
    Xbox,
    PlayStation,
    Nintendo,
}

impl PadFamily {
    pub const ALL: [PadFamily; 4] = [PadFamily::Generic, PadFamily::Xbox, PadFamily::PlayStation, PadFamily::Nintendo];

    /// The family's name, as a setting that chooses one stores it.
    pub const fn name(self) -> &'static str {
        match self {
            PadFamily::Generic => "Generic",
            PadFamily::Xbox => "Xbox",
            PadFamily::PlayStation => "PlayStation",
            PadFamily::Nintendo => "Nintendo",
        }
    }

    /// The family a stored name chooses; none for any other word (Automatic).
    pub fn named(name: &str) -> Option<PadFamily> {
        PadFamily::ALL.into_iter().find(|f| f.name() == name)
    }
}

/// One mark in the unit box: a polyline, filled or stroked (`stroke` a share of the box, 0 for none); `cut` when the
/// glyph's letter is cut out of it rather than drawn over it; `holes`, closed polygons cut out of a filled mark, filled
/// with it odd-even, so a hole inside a hole is filled again (a ring is two).
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub points: Vec<Point>,
    pub filled: bool,
    pub stroke: f64,
    pub cut: bool,
    pub holes: Vec<Vec<Point>>,
}

/// The kit's two sets: LunaP's outlined set (§7.12), and the filled set, each control a solid shape with its letter,
/// symbol or mark cut out, as ES-DE's help draws its buttons (EmuSen_BigPicture.md §40.1, EmuSen_Kit.md §7.28).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlyphStyle {
    Outlined,
    Filled,
}

impl GlyphStyle {
    pub const ALL: [GlyphStyle; 2] = [GlyphStyle::Outlined, GlyphStyle::Filled];
}

/// The words a family prints on a control, centred, in the interface's glyph type at `scale` of its size, `bold` where
/// cut out of a filled shape, so a hole reads at 22 pixels as ES-DE's heavy letters do (§7.28).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Letter {
    pub text: &'static str,
    pub scale: f64,
    pub bold: bool,
}

/// A control's glyph: its marks, and the letter drawn over them or cut out of the one marked `cut`.
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub marks: Vec<Mark>,
    pub letter: Option<Letter>,
}

const PEN: f64 = 0.075;
const THIN: f64 = 0.06;
const C: Point = [0.5, 0.5];
const SEGMENTS: usize = 48;
/// A shoulder's two letters beside a face button's one.
const LABEL_SCALE: f64 = 0.8;

fn at(dx: f64, dy: f64) -> Point {
    [C[0] + dx, C[1] + dy]
}

fn arc(points: &mut Vec<Point>, centre: Point, r: f64, from: f64, to: f64, steps: usize) {
    for i in 0..=steps {
        let a = from + (to - from) * i as f64 / steps as f64;
        points.push([centre[0] + r * a.cos(), centre[1] + r * a.sin()]);
    }
}

fn circle(centre: Point, r: f64) -> Vec<Point> {
    let mut points = Vec::with_capacity(SEGMENTS + 1);
    arc(&mut points, centre, r, 0.0, std::f64::consts::TAU * (SEGMENTS - 1) as f64 / SEGMENTS as f64, SEGMENTS - 1);
    closed(points)
}

fn closed(mut points: Vec<Point>) -> Vec<Point> {
    if let Some(&first) = points.first() {
        points.push(first);
    }
    points
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Vec<Point> {
    closed(vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]])
}

/// A box with its top corners rounded by `top` and its bottom ones by `bottom`.
fn rounded(x: f64, y: f64, w: f64, h: f64, top: f64, bottom: f64) -> Vec<Point> {
    use std::f64::consts::{FRAC_PI_2, PI};
    let mut points = Vec::new();
    arc(&mut points, [x + w - top, y + top], top, -FRAC_PI_2, 0.0, 8);
    arc(&mut points, [x + w - bottom, y + h - bottom], bottom, 0.0, FRAC_PI_2, 8);
    arc(&mut points, [x + bottom, y + h - bottom], bottom, FRAC_PI_2, PI, 8);
    arc(&mut points, [x + top, y + top], top, PI, PI + FRAC_PI_2, 8);
    closed(points)
}

fn stroked(points: Vec<Point>, stroke: f64) -> Mark {
    Mark { points, filled: false, stroke, cut: false, holes: Vec::new() }
}

fn solid(points: Vec<Point>) -> Mark {
    Mark { points, filled: true, stroke: 0.0, cut: false, holes: Vec::new() }
}

/// A plus about the centre, its arms `half` either side of the axes and reaching `reach` from the centre.
fn plus(half: f64, reach: f64) -> Vec<Point> {
    let corners = [
        (-half, -reach),
        (half, -reach),
        (half, -half),
        (reach, -half),
        (reach, half),
        (half, half),
        (half, reach),
        (-half, reach),
        (-half, half),
        (-reach, half),
        (-reach, -half),
        (-half, -half),
    ];
    closed(corners.iter().map(|&(x, y)| at(x, y)).collect())
}

fn line(a: Point, b: Point) -> Mark {
    stroked(vec![a, b], THIN)
}

/// The letter a family prints on a face button at a position; none for the families that print shapes or nothing.
pub fn face_letter(family: PadFamily, control: PadControl) -> Option<&'static str> {
    match (family, control) {
        (PadFamily::Xbox, PadControl::South) | (PadFamily::Nintendo, PadControl::East) => Some("A"),
        (PadFamily::Xbox, PadControl::East) | (PadFamily::Nintendo, PadControl::South) => Some("B"),
        (PadFamily::Xbox, PadControl::West) | (PadFamily::Nintendo, PadControl::North) => Some("X"),
        (PadFamily::Xbox, PadControl::North) | (PadFamily::Nintendo, PadControl::West) => Some("Y"),
        _ => None,
    }
}

/// The words a family prints on its shoulders and triggers.
pub fn shoulder_label(family: PadFamily, control: PadControl) -> &'static str {
    match (family, control) {
        (PadFamily::Xbox, PadControl::LeftShoulder) => "LB",
        (PadFamily::Xbox, PadControl::RightShoulder) => "RB",
        (PadFamily::Xbox, PadControl::LeftTrigger) => "LT",
        (PadFamily::Xbox, PadControl::RightTrigger) => "RT",
        (PadFamily::Nintendo, PadControl::LeftShoulder) => "L",
        (PadFamily::Nintendo, PadControl::RightShoulder) => "R",
        (PadFamily::Nintendo, PadControl::LeftTrigger) => "ZL",
        (PadFamily::Nintendo, PadControl::RightTrigger) => "ZR",
        (_, PadControl::LeftShoulder) => "L1",
        (_, PadControl::RightShoulder) => "R1",
        (_, PadControl::LeftTrigger) => "L2",
        _ => "R2",
    }
}

/// What a screen reader hears: the printed name where the family prints one, else the control's place.
pub fn name(family: PadFamily, control: PadControl) -> &'static str {
    match control {
        PadControl::South | PadControl::East | PadControl::West | PadControl::North => match family {
            PadFamily::PlayStation => match control {
                PadControl::South => "Cross",
                PadControl::East => "Circle",
                PadControl::West => "Square",
                _ => "Triangle",
            },
            PadFamily::Generic => match control {
                PadControl::South => "South button",
                PadControl::East => "East button",
                PadControl::West => "West button",
                _ => "North button",
            },
            _ => face_letter(family, control).unwrap_or_default(),
        },
        PadControl::LeftShoulder | PadControl::RightShoulder | PadControl::LeftTrigger | PadControl::RightTrigger => {
            shoulder_label(family, control)
        }
        PadControl::Up => "D-pad up",
        PadControl::Down => "D-pad down",
        PadControl::Left => "D-pad left",
        PadControl::Right => "D-pad right",
        PadControl::Start => match family {
            PadFamily::Xbox => "Menu",
            PadFamily::PlayStation => "Options",
            PadFamily::Nintendo => "Plus",
            PadFamily::Generic => "Start",
        },
        PadControl::Select => match family {
            PadFamily::Xbox => "View",
            PadFamily::PlayStation => "Create",
            PadFamily::Nintendo => "Minus",
            PadFamily::Generic => "Select",
        },
        PadControl::LeftStick => {
            if family == PadFamily::PlayStation {
                "L3"
            } else {
                "Left stick click"
            }
        }
        PadControl::RightStick => {
            if family == PadFamily::PlayStation {
                "R3"
            } else {
                "Right stick click"
            }
        }
        PadControl::Guide => "Guide",
    }
}

/// The family whose printing a style letters: the filled set has no positional drawing, so a pad whose printing is not
/// known is lettered as an Xbox pad, ES-DE's default type (EmuSen_BigPicture.md §40.1).
pub fn lettered(family: PadFamily, style: GlyphStyle) -> PadFamily {
    if style == GlyphStyle::Filled && family == PadFamily::Generic {
        PadFamily::Xbox
    } else {
        family
    }
}

/// The glyph of a control in a family and a style.
pub fn styled(family: PadFamily, control: PadControl, style: GlyphStyle) -> Glyph {
    match style {
        GlyphStyle::Outlined => glyph(family, control),
        GlyphStyle::Filled => filled::glyph(lettered(family, style), control),
    }
}

/// The glyph of a control in a family in the outlined set (LunaP.md §103.1's table).
pub fn glyph(family: PadFamily, control: PadControl) -> Glyph {
    match control {
        PadControl::South | PadControl::East | PadControl::West | PadControl::North => face(family, control),
        PadControl::Up | PadControl::Down | PadControl::Left | PadControl::Right => dpad(control),
        PadControl::LeftShoulder | PadControl::RightShoulder | PadControl::LeftTrigger | PadControl::RightTrigger => {
            shoulder(family, control)
        }
        PadControl::Start | PadControl::Select => middle(family, control == PadControl::Start),
        PadControl::LeftStick | PadControl::RightStick => {
            Glyph { marks: vec![stroked(circle(C, 0.44), THIN), solid(circle(C, 0.3))], letter: None }
        }
        PadControl::Guide => Glyph { marks: vec![stroked(circle(C, 0.42), PEN), solid(circle(C, 0.16))], letter: None },
    }
}

fn direction(control: PadControl) -> (f64, f64) {
    match control {
        PadControl::North | PadControl::Up => (0.0, -1.0),
        PadControl::South | PadControl::Down => (0.0, 1.0),
        PadControl::West | PadControl::Left => (-1.0, 0.0),
        _ => (1.0, 0.0),
    }
}

fn face(family: PadFamily, control: PadControl) -> Glyph {
    let letter = face_letter(family, control).map(|text| Letter { text, scale: 1.0, bold: false });
    match family {
        // Four dots in a diamond, the one meant filled: a position, for a pad whose printing is not known.
        PadFamily::Generic => {
            let marks = [PadControl::North, PadControl::East, PadControl::South, PadControl::West]
                .iter()
                .map(|&place| {
                    let (dx, dy) = direction(place);
                    let dot = circle(at(dx * 0.29, dy * 0.29), 0.14);
                    if place == control { solid(dot) } else { stroked(dot, THIN) }
                })
                .collect();
            Glyph { marks, letter: None }
        }
        PadFamily::PlayStation => {
            let r = 0.2;
            let mut marks = vec![stroked(circle(C, 0.44), PEN)];
            match control {
                PadControl::South => {
                    marks.push(line(at(-r, -r), at(r, r)));
                    marks.push(line(at(-r, r), at(r, -r)));
                }
                PadControl::East => marks.push(stroked(circle(C, r), THIN)),
                PadControl::West => marks.push(stroked(rect(C[0] - 0.9 * r, C[1] - 0.9 * r, 1.8 * r, 1.8 * r), THIN)),
                _ => marks.push(stroked(closed(vec![at(0.0, -1.05 * r), at(r, 0.7 * r), at(-r, 0.7 * r)]), THIN)),
            }
            Glyph { marks, letter: None }
        }
        // The letter cut out of a disc, where the other lettered family rings it: the two tell apart at a glance.
        PadFamily::Nintendo => Glyph { marks: vec![Mark { cut: true, ..solid(circle(C, 0.46)) }], letter },
        PadFamily::Xbox => Glyph { marks: vec![stroked(circle(C, 0.44), PEN)], letter },
    }
}

const DPAD_ARM: f64 = 0.28;
const DPAD_REACH: f64 = 0.46;

/// A plus with its arms outlined and the arm of the direction filled.
fn dpad(control: PadControl) -> Glyph {
    Glyph { marks: vec![stroked(plus(DPAD_ARM / 2.0, DPAD_REACH), THIN), solid(dpad_arm(control))], letter: None }
}

/// The filled inside of the d-pad's arm along a direction.
fn dpad_arm(control: PadControl) -> Vec<Point> {
    let inset = 0.07;
    let a = DPAD_ARM / 2.0;
    let (half, length) = (a - inset, DPAD_REACH - a - inset);
    match control {
        PadControl::Up => rect(C[0] - half, C[1] - DPAD_REACH + inset, 2.0 * half, length),
        PadControl::Down => rect(C[0] - half, C[1] + a, 2.0 * half, length),
        PadControl::Left => rect(C[0] - DPAD_REACH + inset, C[1] - half, length, 2.0 * half),
        _ => rect(C[0] + a, C[1] - half, length, 2.0 * half),
    }
}

/// Two controls a help bar names as one action, drawn as one glyph (EmuSen_Kit.md §7.54).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pair {
    /// A d-pad axis: both of its directions.
    Axis(PadControl, PadControl),
    /// The two shoulders, or the two triggers.
    Sides { trigger: bool },
    /// The whole d-pad, named by Up with Left: its four directions as one.
    Pad,
}

fn pair_of(a: PadControl, b: PadControl) -> Option<Pair> {
    use PadControl::{Down, Left, LeftShoulder, LeftTrigger, Right, RightShoulder, RightTrigger, Up};
    match (a, b) {
        (Left, Right) | (Right, Left) => Some(Pair::Axis(Left, Right)),
        (Up, Down) | (Down, Up) => Some(Pair::Axis(Up, Down)),
        (LeftShoulder, RightShoulder) | (RightShoulder, LeftShoulder) => Some(Pair::Sides { trigger: false }),
        (LeftTrigger, RightTrigger) | (RightTrigger, LeftTrigger) => Some(Pair::Sides { trigger: true }),
        (Up, Left) | (Left, Up) => Some(Pair::Pad),
        _ => None,
    }
}

/// One glyph for two controls pressed for one action: a d-pad axis is the plus with both of its directions marked, Up
/// with Left the whole d-pad with all four marked, and
/// the two shoulders or the two triggers are their two bodies side by side, unlettered, the same in every family. None
/// for any other two, which a bar draws as two glyphs (EmuSen_Kit.md §7.54).
pub fn paired(a: PadControl, b: PadControl, style: GlyphStyle) -> Option<Glyph> {
    let pair = pair_of(a, b)?;
    Some(match style {
        GlyphStyle::Filled => filled::paired(pair),
        GlyphStyle::Outlined => match pair {
            Pair::Axis(a, b) => Glyph {
                marks: vec![stroked(plus(DPAD_ARM / 2.0, DPAD_REACH), THIN), solid(dpad_arm(a)), solid(dpad_arm(b))],
                letter: None,
            },
            Pair::Sides { trigger } => Glyph { marks: sides(trigger).into_iter().map(|s| stroked(s, THIN)).collect(), letter: None },
            Pair::Pad => Glyph {
                marks: std::iter::once(stroked(plus(DPAD_ARM / 2.0, DPAD_REACH), THIN))
                    .chain([PadControl::Up, PadControl::Down, PadControl::Left, PadControl::Right].map(|c| solid(dpad_arm(c))))
                    .collect(),
                letter: None,
            },
        },
    })
}

/// The two bodies of a pair of shoulders or triggers, left and right, each the shape of one at half its width.
fn sides(trigger: bool) -> [Vec<Point>; 2] {
    let (w, h, r, gap) = (0.43, if trigger { 0.78 } else { 0.56 }, 0.15, 0.06);
    let (top, bottom) = if trigger { (0.04, r) } else { (r, 0.04) };
    let y = C[1] - h / 2.0;
    [rounded(C[0] - gap / 2.0 - w, y, w, h, top, bottom), rounded(C[0] + gap / 2.0, y, w, h, top, bottom)]
}

/// A shoulder is a bar rounded at the top, a trigger taller and rounded at the bottom; each carries its family's name
/// for it, or, with no printing known, the half on its own side filled.
fn shoulder(family: PadFamily, control: PadControl) -> Glyph {
    let trigger = matches!(control, PadControl::LeftTrigger | PadControl::RightTrigger);
    let (w, h, r) = (0.94, if trigger { 0.8 } else { 0.56 }, 0.2);
    let (x, y) = (C[0] - w / 2.0, C[1] - h / 2.0);
    let (top, bottom) = if trigger { (0.05, r) } else { (r, 0.05) };
    let mut marks = vec![stroked(rounded(x, y, w, h, top, bottom), THIN)];
    if family == PadFamily::Generic {
        let left = matches!(control, PadControl::LeftShoulder | PadControl::LeftTrigger);
        let inset = 0.12;
        marks.push(solid(rect(if left { x + inset } else { C[0] }, y + inset, w / 2.0 - inset, h - 2.0 * inset)));
        return Glyph { marks, letter: None };
    }
    Glyph { marks, letter: Some(Letter { text: shoulder_label(family, control), scale: LABEL_SCALE, bold: false }) }
}

/// The two middle buttons: a ring for the families that print a mark in a circle, a pill for the others, each with its own mark.
fn middle(family: PadFamily, start: bool) -> Glyph {
    let mut marks = Vec::new();
    match family {
        PadFamily::Xbox => {
            marks.push(stroked(circle(C, 0.44), PEN));
            if start {
                for i in -1..=1 {
                    let y = f64::from(i) * 0.13;
                    marks.push(line(at(-0.18, y), at(0.18, y)));
                }
            } else {
                marks.push(stroked(rect(C[0] - 0.2, C[1] - 0.16, 0.24, 0.2), THIN));
                marks.push(stroked(rect(C[0] - 0.04, C[1] - 0.04, 0.24, 0.2), THIN));
            }
        }
        PadFamily::Nintendo => {
            marks.push(stroked(circle(C, 0.44), PEN));
            marks.push(line(at(-0.2, 0.0), at(0.2, 0.0)));
            if start {
                marks.push(line(at(0.0, -0.2), at(0.0, 0.2)));
            }
        }
        PadFamily::PlayStation | PadFamily::Generic => {
            marks.push(stroked(rounded(C[0] - 0.46, C[1] - 0.26, 0.92, 0.52, 0.26, 0.26), PEN));
            if family == PadFamily::PlayStation {
                for i in -1..=1 {
                    let i = f64::from(i);
                    if start {
                        marks.push(line(at(-0.16, i * 0.09), at(0.16, i * 0.09)));
                    } else {
                        marks.push(line(at(i * 0.12, -0.1), at(i * 0.12, 0.1)));
                    }
                }
            } else if start {
                marks.push(solid(closed(vec![at(-0.1, -0.13), at(0.14, 0.0), at(-0.1, 0.13)])));
            } else {
                marks.push(solid(rect(C[0] - 0.1, C[1] - 0.1, 0.2, 0.2)));
            }
        }
    }
    Glyph { marks, letter: None }
}
