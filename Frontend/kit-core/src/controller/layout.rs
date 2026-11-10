//! A drawing laid out in a box for the controls a host declares (EmuSen_Kit.md §7.62): which regions are shown and
//! walked, where the drawing stands and at what scale, where each label stands beside it with its leader, and what
//! is painted in which colour as the focus and the held controls change.

use super::drawing::{Drawing, Side, Word};
use super::shape::{darken, n, with_alpha, Bounds, Shape};

/// The space between a column of labels and the drawing, and between two labels.
pub const LEAD: f64 = 24.0;
pub const LABEL_GAP: f64 = 2.0;

/// One thing painted: a path in the drawing's units with its fill and stroke.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub path: String,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub width: f64,
}

/// The drawing in its box: the drawing's place and scale, each shown control's label box (in the order the host
/// gave the controls) and side, the leaders as one path in the box's units, and whether everything fitted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Laid {
    pub drawing: Bounds,
    pub scale: f64,
    pub labels: Vec<Bounds>,
    pub sides: Vec<Side>,
    pub anchors: Vec<(f64, f64)>,
    /// Where each leader meets its label.
    pub ends: Vec<(f64, f64)>,
    pub leaders: String,
    pub fits: bool,
}

/// The regions a host's controls name, in the host's order; a control the drawing lacks is none.
pub fn shown(drawing: &Drawing, controls: &[String]) -> Vec<Option<usize>> {
    controls.iter().map(|c| drawing.regions.iter().position(|r| &r.control == c)).collect()
}

/// Whether a drawing has a region for every control a host declares, and so can stand for that controller.
pub fn covers(drawing: &Drawing, controls: &[String]) -> bool {
    shown(drawing, controls).iter().all(Option::is_some)
}

fn side_of(drawing: &Drawing, region: usize) -> Side {
    let r = &drawing.regions[region];
    r.side.unwrap_or_else(|| if r.shape.bounds().centre().0 < drawing.width / 2.0 { Side::Left } else { Side::Right })
}

/// Places boxes of `sizes` along a line of `length`, each as near its `wanted` start as the others let it, `gap` apart;
/// false when they cannot all fit.
fn pack(wanted: &[f64], sizes: &[f64], length: f64, gap: f64) -> (Vec<f64>, bool) {
    let mut order: Vec<usize> = (0..wanted.len()).collect();
    order.sort_by(|a, b| wanted[*a].total_cmp(&wanted[*b]));
    let total: f64 = sizes.iter().sum::<f64>() + gap * (sizes.len().saturating_sub(1)) as f64;
    let mut at = vec![0.0; wanted.len()];
    let mut next = 0.0_f64;
    for &i in &order {
        at[i] = wanted[i].max(next);
        next = at[i] + sizes[i] + gap;
    }
    let mut limit = length;
    for &i in order.iter().rev() {
        at[i] = at[i].min(limit - sizes[i]).max(0.0);
        limit = at[i] - gap;
    }
    (at, total <= length + 0.5)
}

/// The drawing laid out in a box of `width` × `height` for the host's controls, each label `widths[i]` wide (0 where
/// not measured yet) and `label_height` high.
pub fn lay_out(drawing: &Drawing, controls: &[String], widths: &[f64], label_height: f64, width: f64, height: f64) -> Laid {
    if drawing.width <= 0.0 || drawing.height <= 0.0 {
        // No drawing yet: nothing has a place.
        return Laid { labels: vec![Default::default(); controls.len()], sides: vec![Side::Bottom; controls.len()], ..Laid::default() };
    }
    let regions = shown(drawing, controls);
    let sides: Vec<Side> = regions.iter().map(|r| r.map_or(Side::Bottom, |r| side_of(drawing, r))).collect();
    let w = |i: usize| widths.get(i).copied().unwrap_or(0.0);
    let column = |side: Side| (0..controls.len()).filter(|&i| sides[i] == side).map(w).fold(0.0, f64::max);
    let any = |side: Side| sides.iter().any(|s| *s == side);
    let (left, right) = (column(Side::Left), column(Side::Right));
    let band = |side: Side| if any(side) { label_height + LEAD } else { 0.0 };
    let x0 = if left > 0.0 { left + LEAD } else { 0.0 };
    let x1 = width - if right > 0.0 { right + LEAD } else { 0.0 };
    let (y0, y1) = (band(Side::Top), height - band(Side::Bottom));
    let scale = ((x1 - x0) / drawing.width).min((y1 - y0) / drawing.height).max(0.0);
    let (dw, dh) = (drawing.width * scale, drawing.height * scale);
    let place = Bounds { x: x0 + (x1 - x0 - dw) / 2.0, y: y0 + (y1 - y0 - dh) / 2.0, w: dw, h: dh };
    let anchors: Vec<(f64, f64)> = regions
        .iter()
        .map(|r| {
            let (x, y) = r.map_or((drawing.width / 2.0, drawing.height), |r| drawing.regions[r].shape.bounds().centre());
            (place.x + x * scale, place.y + y * scale)
        })
        .collect();
    let mut labels = vec![Bounds::default(); controls.len()];
    // A label not measured yet has no place beside the drawing.
    let mut fits = scale > 0.0 && (0..controls.len()).all(|i| w(i) > 0.0);
    for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
        let members: Vec<usize> = (0..controls.len()).filter(|&i| sides[i] == side).collect();
        if members.is_empty() {
            continue;
        }
        let across = matches!(side, Side::Top | Side::Bottom);
        let sizes: Vec<f64> = members.iter().map(|&i| if across { w(i) } else { label_height }).collect();
        let wanted: Vec<f64> = members.iter().zip(&sizes).map(|(&i, s)| if across { anchors[i].0 - s / 2.0 } else { anchors[i].1 - s / 2.0 }).collect();
        // The columns stand between the rows of labels above and below, which span the whole width.
        let (from, to) = if across { (0.0, width) } else { (if any(Side::Top) { label_height + LABEL_GAP } else { 0.0 }, height - if any(Side::Bottom) { label_height + LABEL_GAP } else { 0.0 }) };
        let wanted: Vec<f64> = wanted.iter().map(|w| w - from).collect();
        let (at, fitted) = pack(&wanted, &sizes, to - from, LABEL_GAP);
        let at: Vec<f64> = at.iter().map(|a| a + from).collect();
        fits &= fitted;
        for (k, &i) in members.iter().enumerate() {
            labels[i] = match side {
                Side::Left => Bounds { x: left - w(i), y: at[k], w: w(i), h: label_height },
                Side::Right => Bounds { x: width - right, y: at[k], w: w(i), h: label_height },
                Side::Top => Bounds { x: at[k], y: 0.0, w: w(i), h: label_height },
                Side::Bottom => Bounds { x: at[k], y: height - label_height, w: w(i), h: label_height },
            };
        }
    }
    let ends: Vec<(f64, f64)> = labels
        .iter()
        .zip(&sides)
        .map(|(label, side)| match side {
            Side::Left => (label.x + label.w, label.y + label.h / 2.0),
            Side::Right => (label.x, label.y + label.h / 2.0),
            Side::Top => (label.x + label.w / 2.0, label.y + label.h),
            Side::Bottom => (label.x + label.w / 2.0, label.y),
        })
        .collect();
    let mut laid = Laid { drawing: place, scale, labels, sides, anchors, ends, leaders: String::new(), fits };
    laid.leaders = (0..controls.len()).filter(|&i| regions[i].is_some()).map(|i| laid.leader(i)).collect::<Vec<_>>().join(" ");
    laid
}

impl Laid {
    /// One control's leader, from its region to its label, as path data in the box's units; empty for none.
    pub fn leader(&self, i: usize) -> String {
        match (self.anchors.get(i), self.ends.get(i)) {
            (Some((ax, ay)), Some((ex, ey))) => format!("M {} {} L {} {}", n(*ax), n(*ay), n(*ex), n(*ey)),
            _ => String::new(),
        }
    }
}

/// The parts of a shape painted with a stroke and a fill: a union strokes every part first, then fills them, so its
/// seams do not show.
fn painted(out: &mut Vec<Paint>, shape: &Shape, fill: Option<String>, stroke: Option<String>, width: f64) {
    let parts = shape.subpaths();
    if parts.len() > 1 && stroke.is_some() && width > 0.0 {
        out.extend(parts.iter().map(|p| Paint { path: p.clone(), fill: None, stroke: stroke.clone(), width }));
        out.extend(parts.into_iter().map(|p| Paint { path: p, fill: fill.clone(), stroke: None, width: 0.0 }));
    } else {
        out.extend(parts.into_iter().map(|p| Paint { path: p, fill: fill.clone(), stroke: stroke.clone(), width }));
    }
}

/// What a region looks like now: whether it is the one focused and whether it is held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lit {
    pub focused: bool,
    pub held: bool,
}

/// Everything painted, in order, with the regions lit as `lit` says (by the drawing's region index) and `accent` the
/// colour of a focus and a press, as LunaP's ControllerDiagram drew them.
pub fn paints(drawing: &Drawing, lit: &dyn Fn(usize) -> Lit, knobs: &dyn Fn(usize) -> (f64, f64), accent: &str) -> Vec<Paint> {
    let mut out = Vec::new();
    let region = |out: &mut Vec<Paint>, i: usize| {
        let r = &drawing.regions[i];
        let Lit { focused, held } = lit(i);
        if focused && !r.arm {
            painted(out, &r.shape, None, Some(accent.to_owned()), 14.0);
        }
        if held && !r.arm {
            painted(out, &r.shape, None, Some(with_alpha(accent, 90)), 26.0);
        }
        let fill = if held { Some(accent.to_owned()) } else { r.fill.clone() };
        let stroke = r.stroke.clone().filter(|_| r.width > 0.0).map(|s| if held { darken(accent, 0.3) } else { s });
        if fill.is_some() || stroke.is_some() {
            painted(out, &r.shape, fill, stroke, r.width);
        }
        if focused && !held && r.arm {
            painted(out, &r.shape, Some(with_alpha(accent, 130)), None, 0.0);
        }
        if let Some(m) = &r.mark {
            painted(out, &m.shape, m.fill.clone(), m.stroke.clone(), m.width);
        }
    };
    for i in (0..drawing.regions.len()).filter(|&i| drawing.regions[i].behind) {
        region(&mut out, i);
    }
    for p in &drawing.body {
        painted(&mut out, &p.shape, p.fill.clone(), p.stroke.clone(), p.width);
    }
    for s in &drawing.sticks {
        painted(&mut out, &s.gate, Some(s.gate_fill.clone()), None, 0.0);
    }
    for i in (0..drawing.regions.len()).filter(|&i| !drawing.regions[i].behind) {
        region(&mut out, i);
    }
    for p in &drawing.marks {
        painted(&mut out, &p.shape, p.fill.clone(), p.stroke.clone(), p.width);
    }
    for (k, s) in drawing.sticks.iter().enumerate() {
        let (dx, dy) = knobs(k);
        let knob = Shape::Circle { cx: s.cx + dx * s.travel, cy: s.cy + dy * s.travel, r: s.knob };
        let held = s.ways.iter().any(|w| drawing.regions.iter().position(|r| &r.control == w).is_some_and(|i| lit(i).held));
        painted(&mut out, &knob, Some(if held { accent.to_owned() } else { s.knob_fill.clone() }), Some(s.knob_stroke.clone()), 3.0);
    }
    out
}

/// The words printed on the drawing: the shell's, then each region's caption.
pub fn words(drawing: &Drawing) -> Vec<Word> {
    let mut out: Vec<Word> = drawing.words.clone();
    out.extend(drawing.regions.iter().filter_map(|r| r.caption.clone()));
    out
}

/// The next control the walk focuses from `at` among `count`, by `by`, round from either end.
pub fn walk(at: usize, by: i32, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (at as i64 + i64::from(by)).rem_euclid(count as i64) as usize
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The drawing as one SVG document in its own units, for a picture of it: what `paints` paints, then the printed words,
/// each centred on its point and turned by its angle. The words are part of the picture, read by no one.
pub fn svg(drawing: &Drawing, painted: &[Paint]) -> String {
    let mut out = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#, w = n(drawing.width), h = n(drawing.height));
    for p in painted {
        let paint = |colour: Option<&str>, what: &str| match colour {
            None => format!(r#" {what}="none""#),
            Some(c) => match alpha_of(c).filter(|a| *a < 255) {
                Some(a) => format!(r#" {what}="{}" {what}-opacity="{}""#, svg_colour(c), n(f64::from(a) / 255.0)),
                None => format!(r#" {what}="{}""#, svg_colour(c)),
            },
        };
        out.push_str(&format!(
            r#"<path d="{}"{}{} stroke-width="{}" fill-rule="evenodd" stroke-linejoin="round" stroke-linecap="round"/>"#,
            p.path,
            paint(p.fill.as_deref(), "fill"),
            paint(p.stroke.as_deref(), "stroke"),
            n(p.width)
        ));
    }
    for w in words(drawing) {
        // The words' middle on the point: a capital stands about 0.35 of its size above the baseline.
        let baseline = w.y + w.size * 0.35;
        out.push_str(&format!(
            r#"<text x="{x}" y="{b}" font-family="sans-serif" font-size="{s}" font-weight="{weight}" fill="{c}" text-anchor="middle" transform="rotate({a} {x} {y})">{t}</text>"#,
            x = n(w.x),
            y = n(w.y),
            b = n(baseline),
            s = n(w.size),
            weight = if w.bold { "bold" } else { "normal" },
            c = svg_colour(&w.colour),
            a = n(w.angle),
            t = escaped(&w.text)
        ));
    }
    out.push_str("</svg>");
    out
}

/// A colour's RGB as SVG names it, `#RRGGBB`; its alpha, if any, goes in an opacity of its own.
fn svg_colour(colour: &str) -> String {
    let hex = colour.trim_start_matches('#');
    format!("#{}", if hex.len() == 8 { &hex[2..] } else { hex })
}

fn alpha_of(colour: &str) -> Option<u8> {
    let hex = colour.trim_start_matches('#');
    (hex.len() == 8).then(|| u8::from_str_radix(&hex[..2], 16).ok()).flatten()
}

/// A text as a `data:` URL's body: the characters a URL reserves written as their percent codes.
pub fn data_url(mime: &str, text: &str) -> String {
    let mut out = format!("data:{mime},");
    for ch in text.chars() {
        match ch {
            '#' | '%' | '<' | '>' | '"' | '\n' | '?' | '&' | ' ' => out.push_str(&format!("%{:02X}", ch as u32)),
            c => out.push(c),
        }
    }
    out
}
