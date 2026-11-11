//! An approved mockup as numbers, `mockup 1` (EmuSen_Qt.md §8.2, §7.1): every named element's box, colour and words,
//! written by hand from the canvas once, and a shown state's tree held to it within two design units, token colours
//! exactly. The format and its reasons are in EmuSen_Qt.md §7.1, step 2. *[own, because the format is EmuSen's own]*

use crate::tree::{colour, Node};
use emusen_kit_core::tokens::{self, Colour, ColourToken, Variant};

/// The tolerance on every edge, in design units.
pub const UNITS: f64 = 2.0;
/// The tolerance on a stated alpha, in levels of 255.
pub const ALPHA_LEVELS: f64 = 3.0;

/// One edge or size of an element's box a mockup states; the ones it leaves out are free.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    X,
    Y,
    W,
    H,
    Right,
    Bottom,
}

/// One named element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element {
    pub name: String,
    pub edges: Vec<(Edge, f64)>,
    /// A colour token the element paints, by its name in kit-core (`Muted`, `SurfaceRaised`).
    pub token: Option<String>,
    /// A colour that is no token, as `#RRGGBB`, compared within a level a channel.
    pub ink: Option<Colour>,
    /// The paint's alpha with the item's opacity, from 0 to 1.
    pub alpha: Option<f64>,
    /// The words, exactly.
    pub text: Option<String>,
    /// Words that are the canvas's example data: the element must show words, which ones is the library's.
    pub data: bool,
}

/// A canvas: its name and size in design units, and its elements in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mockup {
    pub canvas: String,
    pub width: f64,
    pub height: f64,
    pub elements: Vec<Element>,
}

fn unquote(v: &str) -> String {
    v.trim_matches('"').replace("\\\"", "\"")
}

/// Splits `key=value` pairs, a quoted value keeping its spaces.
fn pairs(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = line.trim();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].trim().to_owned();
        let after = &rest[eq + 1..];
        let (value, next) = if let Some(quoted) = after.strip_prefix('"') {
            let end = quoted.find('"').unwrap_or(quoted.len());
            (format!("\"{}\"", &quoted[..end]), quoted.get(end + 1..).unwrap_or(""))
        } else {
            let end = after.find(' ').unwrap_or(after.len());
            (after[..end].to_owned(), &after[end..])
        };
        out.push((key, value));
        rest = next.trim_start();
    }
    out
}

fn rgb(text: &str) -> Option<Colour> {
    let hex = text.strip_prefix('#').filter(|h| h.len() == 6)?;
    colour(&format!("#FF{hex}"))
}

/// Reads a mockup; a line it does not understand is an error naming it.
pub fn parse(text: &str) -> Result<Mockup, String> {
    let mut m = Mockup::default();
    let mut versioned = false;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
        match word {
            "mockup" if rest == "1" => versioned = true,
            "canvas" => {
                let f: Vec<&str> = rest.split_whitespace().collect();
                let num = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
                match (f.first(), num(1), num(2)) {
                    (Some(name), Some(w), Some(h)) => (m.canvas, m.width, m.height) = ((*name).to_owned(), w, h),
                    _ => return Err(format!("line {}: canvas NAME WIDTH HEIGHT", n + 1)),
                }
            }
            "source" | "note" => {}
            "element" => {
                // A name with a space is quoted.
                let (name, attrs) = match rest.strip_prefix('"').and_then(|q| q.split_once('"')) {
                    Some((name, attrs)) => (name, attrs),
                    None => rest.split_once(' ').unwrap_or((rest, "")),
                };
                let mut e = Element { name: name.to_owned(), ..Element::default() };
                for (key, value) in pairs(attrs) {
                    let number = || value.parse::<f64>().map_err(|_| format!("line {}: {key}={value} is no number", n + 1));
                    match key.as_str() {
                        "x" => e.edges.push((Edge::X, number()?)),
                        "y" => e.edges.push((Edge::Y, number()?)),
                        "w" => e.edges.push((Edge::W, number()?)),
                        "h" => e.edges.push((Edge::H, number()?)),
                        "right" => e.edges.push((Edge::Right, number()?)),
                        "bottom" => e.edges.push((Edge::Bottom, number()?)),
                        "token" => e.token = Some(value),
                        "ink" => e.ink = Some(rgb(&value).ok_or_else(|| format!("line {}: ink={value} is not #RRGGBB", n + 1))?),
                        "alpha" => e.alpha = Some(number()?),
                        "text" => e.text = Some(unquote(&value)),
                        "words" if value == "data" => e.data = true,
                        _ => return Err(format!("line {}: unknown {key}={value}", n + 1)),
                    }
                }
                if e.token.is_some() && token(&e.token.clone().unwrap_or_default()).is_none() {
                    return Err(format!("line {}: no token {}", n + 1, e.token.unwrap_or_default()));
                }
                m.elements.push(e);
            }
            _ => return Err(format!("line {}: {line}", n + 1)),
        }
    }
    if !versioned {
        return Err("no `mockup 1` line".into());
    }
    Ok(m)
}

/// A token by its name in kit-core.
pub fn token(name: &str) -> Option<ColourToken> {
    ColourToken::ALL.into_iter().find(|t| format!("{t:?}").eq_ignore_ascii_case(name))
}

/// One way a shown state differs from its mockup: the element, the kind of difference (an edge's name, `not shown`,
/// `paints`, `alpha`, `says`, `shows no words`) and the words of its line.
#[derive(Clone, Debug, PartialEq)]
pub struct Difference {
    pub element: String,
    pub kind: &'static str,
    pub words: String,
}

impl Difference {
    fn new(e: &Element, kind: &'static str, words: String) -> Difference {
        Difference { element: e.name.clone(), kind, words }
    }
}

impl std::fmt::Display for Difference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.element, self.words)
    }
}

impl Edge {
    /// The edge's name as a mockup's difference names it.
    pub fn name(self) -> &'static str {
        match self {
            Edge::X => "X",
            Edge::Y => "Y",
            Edge::W => "W",
            Edge::H => "H",
            Edge::Right => "Right",
            Edge::Bottom => "Bottom",
        }
    }
}

/// The state's items (a tree's nodes) held to the mockup: each element found by its objectName, its box in design units (the scene's
/// pixels over `unit`) within `UNITS` of every stated edge, its paints holding its token's colour or its ink, and its words.
pub fn compare(m: &Mockup, nodes: &[Node], unit: f64, variant: Variant) -> Vec<Difference> {
    compare_in(m, nodes, unit, &|t| tokens::colour(variant, t))
}

/// As `compare`, each token's colour as `palette` gives it: a theme's, its overrides over the kit's.
pub fn compare_in(m: &Mockup, nodes: &[Node], unit: f64, palette: &dyn Fn(ColourToken) -> Colour) -> Vec<Difference> {
    let mut faults = Vec::new();
    for e in &m.elements {
        let Some(i) = nodes.iter().position(|n| n.name == e.name && n.visible) else {
            faults.push(Difference::new(e, "not shown", "not shown".to_owned()));
            continue;
        };
        let n = &nodes[i];
        let (x, y, w, h) = (n.x / unit, n.y / unit, n.w / unit, n.h / unit);
        for &(edge, want) in &e.edges {
            let got = match edge {
                Edge::X => x,
                Edge::Y => y,
                Edge::W => w,
                Edge::H => h,
                Edge::Right => x + w,
                Edge::Bottom => y + h,
            };
            if (got - want).abs() > UNITS {
                faults.push(Difference::new(e, edge.name(), format!("{edge:?} {got:.1}, the mockup's {want}")));
            }
        }
        let mut paints: Vec<Colour> = n.colors.iter().filter_map(|c| colour(c)).collect();
        if let Some(ink) = n.ink.as_deref().and_then(colour) {
            paints.push(ink);
        }
        let same =
            |a: Colour, b: Colour, levels: u8| a.r.abs_diff(b.r) <= levels && a.g.abs_diff(b.g) <= levels && a.b.abs_diff(b.b) <= levels;
        let wanted = e.token.as_deref().and_then(token).map(|t| (palette(t), 0u8)).or(e.ink.map(|c| (c, 1u8)));
        if let Some((want, levels)) = wanted {
            match paints.iter().find(|p| same(**p, want, levels)) {
                None => faults.push(Difference::new(
                    e,
                    "paints",
                    format!("paints {:?}, not {}", n.colors, e.token.clone().unwrap_or_else(|| format!("{want:?}"))),
                )),
                Some(p) => {
                    if let Some(alpha) = e.alpha {
                        let got = f64::from(p.a) * n.opacity;
                        if (got - alpha * 255.0).abs() > ALPHA_LEVELS {
                            faults.push(Difference::new(e, "alpha", format!("alpha {:.3}, the mockup's {alpha}", got / 255.0)));
                        }
                    }
                }
            }
        }
        let words = n.text.clone().unwrap_or_default();
        if let Some(text) = &e.text {
            if &words != text {
                faults.push(Difference::new(e, "says", format!("says {words:?}, the mockup {text:?}")));
            }
        } else if e.data && words.trim().is_empty() {
            faults.push(Difference::new(e, "shows no words", "shows no words".to_owned()));
        }
    }
    faults
}
