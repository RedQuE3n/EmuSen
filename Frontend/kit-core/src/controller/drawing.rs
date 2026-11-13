//! A controller's drawing as data (EmuSen_Kit.md §7.62): the shell, the regions a host binds by name, the marks and
//! words printed on it and its sticks, read from a drawing file. The kit names no controller: a region is named by
//! whatever control the host's controller declares, and a file is chosen by the host.

use serde_json::Value;

use super::shape::{darken, Bounds, Shape};

/// A part of the shell, or a mark printed over the regions.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub width: f64,
}

/// Words printed on the shell or on a region.
#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub colour: String,
    pub angle: f64,
    pub bold: bool,
}

/// The side of the drawing a region's label stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

impl Side {
    fn parse(word: Option<&str>) -> Option<Side> {
        match word? {
            "left" => Some(Side::Left),
            "right" => Some(Side::Right),
            "top" => Some(Side::Top),
            "bottom" => Some(Side::Bottom),
            _ => None,
        }
    }
}

/// One button, direction or trigger: the control it stands for, its words, its shape and look.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    /// The control a host binds it by, as the controller declares it.
    pub control: String,
    /// What its label and a reader call it.
    pub name: String,
    pub shape: Shape,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub width: f64,
    /// A direction pad's arm: lit by a wash over it rather than a ring around it, as its neighbours touch it.
    pub arm: bool,
    pub caption: Option<Word>,
    pub mark: Option<Part>,
    /// Drawn under the shell, as a trigger seen past its edge.
    pub behind: bool,
    /// The side its label stands on, where the file says; else the side nearer its centre.
    pub side: Option<Side>,
}

/// A stick: its gate, its knob and how far the knob travels.
#[derive(Clone, Debug, PartialEq)]
pub struct Stick {
    pub cx: f64,
    pub cy: f64,
    pub knob: f64,
    pub travel: f64,
    pub gate: Shape,
    pub gate_fill: String,
    pub knob_fill: String,
    pub knob_stroke: String,
    /// The directions' controls, up, down, left and right, which move the knob while held.
    pub ways: Vec<String>,
}

/// A whole drawing, its layers in the order they are drawn: the regions behind, the shell, the gates, the regions, the
/// marks, the words, the knobs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drawing {
    /// What a reader calls the drawing ("SNES controller").
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub body: Vec<Part>,
    pub regions: Vec<Region>,
    pub marks: Vec<Part>,
    pub words: Vec<Word>,
    pub sticks: Vec<Stick>,
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn number(v: &Value, key: &str, or: f64) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(or)
}

fn at(v: &Value, i: usize) -> f64 {
    v.get(i).and_then(Value::as_f64).unwrap_or(0.0)
}

fn part(v: &Value, shape: Shape) -> Part {
    Part { shape, fill: text(v, "fill"), stroke: text(v, "stroke"), width: number(v, "width", 0.0) }
}

fn word(v: &Value) -> Word {
    let place = &v["at"];
    Word {
        text: text(v, "word").unwrap_or_default(),
        x: at(place, 0),
        y: at(place, 1),
        size: number(v, "size", 17.0),
        colour: text(v, "colour").unwrap_or_else(|| "#FFFFFF".into()),
        angle: number(v, "angle", 0.0),
        bold: v.get("bold").and_then(Value::as_bool).unwrap_or(true),
    }
}

/// A region's caption, centred on its shape.
fn caption(v: &Value, shape: &Shape) -> Option<Word> {
    let words = text(v, "caption")?;
    let (x, y) = shape.bounds().centre();
    Some(Word {
        text: words,
        x,
        y,
        size: number(v, "caption_size", number(v, "size", 30.0)),
        colour: text(v, "caption_colour").unwrap_or_else(|| "#FFFFFF".into()),
        angle: number(v, "caption_angle", 0.0),
        bold: true,
    })
}

fn region(v: &Value, shape: Shape) -> Result<Region, String> {
    let control = text(v, "region").ok_or_else(|| format!("a region with no control: {v}"))?;
    let mark = match v.get("mark") {
        Some(m) => Some(Part { shape: Shape::parse(m)?, fill: Some(text(v, "mark_colour").unwrap_or_else(|| "#FFFFFF".into())), stroke: None, width: 0.0 }),
        None => None,
    };
    Ok(Region {
        name: text(v, "name").unwrap_or_else(|| control.clone()),
        caption: caption(v, &shape),
        control,
        fill: text(v, "fill"),
        stroke: text(v, "stroke"),
        width: number(v, "width", 0.0),
        arm: v.get("arm").and_then(Value::as_bool).unwrap_or(false),
        mark,
        behind: v.get("behind").and_then(Value::as_bool).unwrap_or(false),
        side: Side::parse(v.get("side").and_then(Value::as_str)),
        shape,
    })
}

impl Drawing {
    /// A drawing file read: `{"name", "design": [w, h], "items": [...]}`, its items as EmuSen_Kit.md §7.62 lists them.
    pub fn parse(text: &str) -> Result<Drawing, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("a drawing that is not JSON: {e}"))?;
        let mut d = Drawing {
            name: v["name"].as_str().unwrap_or_default().to_owned(),
            width: at(&v["design"], 0),
            height: at(&v["design"], 1),
            ..Drawing::default()
        };
        if d.width <= 0.0 || d.height <= 0.0 {
            return Err("a drawing with no design size".into());
        }
        for item in v["items"].as_array().into_iter().flatten() {
            d.add(item)?;
        }
        Ok(d)
    }

    fn add(&mut self, v: &Value) -> Result<(), String> {
        if let Some(s) = v.get("part") {
            self.body.push(part(v, Shape::parse(s)?));
        } else if let Some(s) = v.get("mark").filter(|_| v.get("region").is_none()) {
            self.marks.push(part(v, Shape::parse(s)?));
        } else if v.get("word").is_some() {
            self.words.push(word(v));
        } else if let Some(a) = v.get("round") {
            let shape = Shape::Circle { cx: at(a, 0), cy: at(a, 1), r: at(a, 2) };
            let colour = text(v, "colour").unwrap_or_else(|| "#808080".into());
            let mut r = region(v, shape)?;
            r.stroke = Some(darken(&colour, 0.35));
            r.fill = Some(colour);
            r.width = 3.0;
            self.regions.push(r);
        } else if let Some(a) = v.get("dpad") {
            self.dpad(v, at(a, 0), at(a, 1), at(a, 2), at(a, 3))?;
        } else if let Some(a) = v.get("stick") {
            self.stick(v, at(a, 0), at(a, 1), at(a, 2), at(a, 3), at(a, 4))?;
        } else if let Some(s) = v.get("shape") {
            let shape = Shape::parse(s)?;
            self.regions.push(region(v, shape)?);
        } else {
            return Err(format!("an item of no known kind: {v}"));
        }
        Ok(())
    }

    /// The controls of a list in an item, up, down, left and right, with their names where the item gives them.
    fn four(v: &Value, key: &str) -> Vec<(String, String)> {
        let controls: Vec<String> = v[key].as_array().into_iter().flatten().filter_map(|c| c.as_str().map(str::to_owned)).collect();
        let names: Vec<String> = v["names"].as_array().into_iter().flatten().filter_map(|c| c.as_str().map(str::to_owned)).collect();
        controls.iter().enumerate().map(|(i, c)| (c.clone(), names.get(i).cloned().unwrap_or_else(|| c.clone()))).collect()
    }

    /// A plus-shaped direction pad: the cross on the shell, an arm region each way with an arrow, and the hub.
    fn dpad(&mut self, v: &Value, cx: f64, cy: f64, arm: f64, reach: f64) -> Result<(), String> {
        let w = arm / 2.0;
        let radius = number(v, "radius", 10.0);
        let fill = text(v, "fill").unwrap_or_else(|| "#303038".into());
        let arrow = text(v, "arrow").unwrap_or_else(|| "#5E5E69".into());
        let stroke = text(v, "stroke");
        let cross = Shape::Union(vec![
            Shape::Box { x: cx - reach, y: cy - w, w: 2.0 * reach, h: arm, r: radius, ends: None },
            Shape::Box { x: cx - w, y: cy - reach, w: arm, h: 2.0 * reach, r: radius, ends: None },
        ]);
        self.body.push(Part { shape: cross, fill: Some(fill.clone()), width: if stroke.is_some() { 3.0 } else { 0.0 }, stroke });
        let ways = [(0.0, -1.0, "up"), (0.0, 1.0, "down"), (-1.0, 0.0, "left"), (1.0, 0.0, "right")];
        for ((dx, dy, end), (control, name)) in ways.iter().zip(Drawing::four(v, "controls")) {
            let (x0, y0) = (if *dx == 0.0 { cx - w } else if *dx < 0.0 { cx - reach } else { cx + w }, if *dy == 0.0 { cy - w } else if *dy < 0.0 { cy - reach } else { cy + w });
            let (bw, bh) = (if *dx == 0.0 { arm } else { reach - w }, if *dy == 0.0 { arm } else { reach - w });
            let mid = (reach + w) / 2.0;
            let mark = Shape::Arrow { cx: cx + dx * mid, cy: cy + dy * mid, size: arm * 0.36, dx: *dx, dy: *dy };
            let shape = Shape::Box { x: x0, y: y0, w: bw, h: bh, r: radius, ends: Some((*end).into()) };
            self.regions.push(Region {
                control,
                name,
                shape,
                fill: None,
                stroke: None,
                width: 0.0,
                arm: true,
                caption: None,
                mark: Some(Part { shape: mark, fill: Some(arrow.clone()), stroke: None, width: 0.0 }),
                behind: false,
                side: Side::parse(v.get("side").and_then(Value::as_str)),
            });
        }
        self.marks.push(Part { shape: Shape::Circle { cx, cy, r: arm * 0.2 }, fill: Some(darken(&fill, 0.25)), stroke: None, width: 0.0 });
        Ok(())
    }

    /// A stick: its gate, its four directions as bands of a ring around the gate each with an arrow, and its click.
    fn stick(&mut self, v: &Value, sx: f64, sy: f64, gate: f64, knob: f64, travel: f64) -> Result<(), String> {
        let octagonal = v.get("octagonal").and_then(Value::as_bool).unwrap_or(false);
        let gate_shape = if octagonal { Shape::Octagon { cx: sx, cy: sy, r: gate } } else { Shape::Circle { cx: sx, cy: sy, r: gate } };
        let ways = Drawing::four(v, "controls");
        let band = text(v, "band").unwrap_or_else(|| "#6E6E78".into());
        let arrow = text(v, "arrow").unwrap_or_else(|| "#C4C4CC".into());
        let (inner, outer) = (gate + 8.0, gate + 28.0);
        let arcs = [(-132.0, -48.0), (48.0, 132.0), (138.0, 222.0), (-42.0, 42.0)];
        for ((from, to), (control, name)) in arcs.iter().zip(ways.iter().cloned()) {
            let mid = f64::midpoint(*from, *to).to_radians();
            let r = f64::midpoint(inner, outer);
            let sign = |x: f64| if x.abs() < 1e-3 { 0.0 } else { x.signum() };
            let mark = Shape::Arrow { cx: sx + r * mid.cos(), cy: sy + r * mid.sin(), size: 15.0, dx: sign(mid.cos()), dy: sign(mid.sin()) };
            self.regions.push(Region {
                control,
                name,
                shape: Shape::ArcBand { cx: sx, cy: sy, inner, outer, from: *from, to: *to },
                fill: Some(band.clone()),
                stroke: None,
                width: 0.0,
                arm: false,
                caption: None,
                mark: Some(Part { shape: mark, fill: Some(arrow.clone()), stroke: None, width: 0.0 }),
                behind: false,
                side: Side::parse(v.get("side").and_then(Value::as_str)),
            });
        }
        if let Some(click) = text(v, "click") {
            let name = text(v, "click_name").unwrap_or_else(|| click.clone());
            let shape = Shape::Circle { cx: sx, cy: sy, r: knob };
            self.regions.push(Region { control: click, name, shape, fill: None, stroke: None, width: 0.0, arm: false, caption: None, mark: None, behind: false, side: Side::parse(v.get("side").and_then(Value::as_str)) });
        }
        self.sticks.push(Stick {
            cx: sx,
            cy: sy,
            knob,
            travel,
            gate: gate_shape,
            gate_fill: text(v, "gate_fill").unwrap_or_else(|| "#4A4A53".into()),
            knob_fill: text(v, "knob_fill").unwrap_or_else(|| "#A8A8B1".into()),
            knob_stroke: text(v, "edge").unwrap_or_else(|| "#45454E".into()),
            ways: ways.into_iter().map(|(c, _)| c).collect(),
        });
        Ok(())
    }

    /// The region standing for a control.
    pub fn region(&self, control: &str) -> Option<&Region> {
        self.regions.iter().find(|r| r.control == control)
    }

    /// The box of everything drawn.
    pub fn bounds(&self) -> Bounds {
        Bounds { x: 0.0, y: 0.0, w: self.width, h: self.height }
    }
}
