// What the glow tests share: Eclipse's tokens as the host hands them, and the CSS rule a glow is held to (EmuSen_Kit.md §7.113).
#![allow(dead_code)]

use emusen_kit_core::tokens::Variant;

pub const ECLIPSE: &str = include_str!("../../../themes/eclipse/theme.json");

/// Eclipse's dark overrides: each top-level scalar `"name": value` entry of its `tokens.dark`, skipping nested objects.
pub fn eclipse() -> Vec<(String, String)> {
    let start = ECLIPSE.find("\"dark\": {").expect("Eclipse's dark tokens");
    let block = &ECLIPSE[start + ECLIPSE[start..].find('{').expect("dark brace")..];
    let mut out = Vec::new();
    let (mut depth, mut in_str, mut esc, mut entry_start) = (0usize, false, false, None);
    for (i, b) in block.bytes().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => {
                if depth == 1 && entry_start.is_none() {
                    entry_start = Some(i);
                }
                in_str = true;
            }
            b'{' => {
                if depth == 1 {
                    entry_start = None;
                }
                depth += 1;
            }
            b'}' => {
                if depth == 1 {
                    if let Some(s) = entry_start.take() {
                        if let Some((n, v)) = block[s..i].trim().split_once(':') {
                            out.push((n.trim().trim_matches('"').to_owned(), v.trim().trim_matches('"').to_owned()));
                        }
                    }
                    break;
                }
                depth -= 1;
            }
            b',' if depth == 1 => {
                if let Some(s) = entry_start.take() {
                    if let Some((n, v)) = block[s..i].trim().split_once(':') {
                        out.push((n.trim().trim_matches('"').to_owned(), v.trim().trim_matches('"').to_owned()));
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// The same, for the kit's `set_overrides`.
pub fn eclipse_dark() -> Vec<(Variant, String, String)> {
    eclipse().into_iter().map(|(n, v)| (Variant::Dark, n, v)).collect()
}

/// One of Eclipse's number tokens.
pub fn number(tokens: &[(String, String)], name: &str) -> f64 {
    tokens.iter().find(|(k, _)| k == name).and_then(|(_, v)| v.parse().ok()).unwrap_or_else(|| panic!("Eclipse's {name}"))
}

/// A glow's recipe as CSS writes it: a box (width, height, corner radius) and its box-shadow's spread, blur radius and alpha.
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub width: f64,
    pub height: f64,
    pub radius: f64,
    pub spread: f64,
    pub blur: f64,
    pub alpha: f64,
}

impl Recipe {
    /// The shadow's corner radius: CSS Backgrounds 3 §6.1.1 grows a corner by the spread, times 1 + (r − 1)³ where the
    /// radius is less than the spread, r the radius over the spread.
    pub fn grown_radius(&self) -> f64 {
        if self.spread <= 0.0 {
            return self.radius;
        }
        let r = self.radius / self.spread;
        let share = if r < 1.0 { 1.0 + (r - 1.0).powi(3) } else { 1.0 };
        self.radius + self.spread * share
    }

    /// Whether a point, the box's centre at the origin, is inside the grown shape.
    pub fn inside(&self, x: f64, y: f64) -> bool {
        let (hw, hh) = (self.width / 2.0 + self.spread, self.height / 2.0 + self.spread);
        let r = self.grown_radius().min(hw).min(hh);
        let (ax, ay) = (x.abs(), y.abs());
        if ax > hw || ay > hh {
            return false;
        }
        let (cx, cy) = (ax - (hw - r), ay - (hh - r));
        cx <= 0.0 || cy <= 0.0 || cx * cx + cy * cy <= r * r
    }

    /// The shadow's alpha at a point d past the box's top edge, at its middle, by CSS Backgrounds 3 §6.1.2: the grown shape
    /// blurred by a Gaussian of a standard deviation half the blur radius; outside the box only (§6.1.1). Integrated on a
    /// grid of a quarter unit over four standard deviations round the point.
    pub fn css_above(&self, d: f64) -> f64 {
        if d <= 0.0 {
            return 0.0;
        }
        let (px, py) = (0.0, -(self.height / 2.0 + d));
        if self.blur <= 0.0 {
            return if self.inside(px, py) { self.alpha } else { 0.0 };
        }
        let sigma = self.blur / 2.0;
        let step = 0.25;
        let reach = (4.0 * sigma / step).ceil() as i64;
        let mut sum = 0.0;
        for i in -reach..=reach {
            for j in -reach..=reach {
                let (dx, dy) = (i as f64 * step, j as f64 * step);
                if self.inside(px + dx, py + dy) {
                    sum += (-(dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
                }
            }
        }
        self.alpha * sum * step * step / (2.0 * std::f64::consts::PI * sigma * sigma)
    }
}

/// The alpha a glow is seen at over black, by the share of its colour a pixel holds on the colour's strongest channel.
pub fn share(seen: (u8, u8, u8), glow: (u8, u8, u8)) -> f64 {
    let pick = |c: (u8, u8, u8)| if glow.0 >= glow.1 && glow.0 >= glow.2 { c.0 } else if glow.1 >= glow.2 { c.1 } else { c.2 };
    f64::from(pick(seen)) / f64::from(pick(glow))
}

/// The glow scene's two measured parts: the lone ring's box and the disc's, each with the recipe its tokens give.
pub mod scene {
    use super::{number, share, Recipe};
    use emusen_kit_core::tokens::Colour;
    use emusen_qt_test::tree::Tree;
    use emusen_qt_test::Scene;

    pub const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/GlowScene.qml");
    /// The scene's window, a test input written in GlowScene.qml.
    pub const WINDOW: (i32, i32) = (1920, 1200);

    /// A named item's box in the window, from the tree.
    pub fn box_of(tree: &Tree, name: &str) -> (f64, f64, f64, f64) {
        let n = &tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene")) as usize];
        (n.x, n.y, n.w, n.h)
    }

    pub fn colour(scene: &mut Scene<'_>, name: &str) -> Colour {
        let text = scene.property("scene", name);
        Colour::parse(&text).unwrap_or_else(|| panic!("{name} is a colour: {text}"))
    }

    /// The ring's recipe under Eclipse: the ring's box grown by nothing, its corners the button's, the glow's tokens.
    pub fn ring(tokens: &[(String, String)], glow: Colour, b: (f64, f64, f64, f64)) -> Recipe {
        Recipe {
            width: b.2,
            height: b.3,
            radius: number(tokens, "buttonRadius"),
            spread: number(tokens, "glowSpread"),
            blur: number(tokens, "glowBlur"),
            alpha: f64::from(glow.a) / 255.0,
        }
    }

    /// The disc's recipe, the glow's (0) or the halo's (1).
    pub fn disc(tokens: &[(String, String)], halo: bool, glow: Colour, b: (f64, f64, f64, f64)) -> Recipe {
        let which = if halo { "Halo" } else { "Glow" };
        Recipe {
            width: b.2,
            height: b.3,
            radius: b.2 / 2.0,
            spread: number(tokens, &format!("corona{which}Spread")),
            blur: number(tokens, &format!("corona{which}Blur")),
            alpha: f64::from(glow.a) / 255.0,
        }
    }

    /// The distances measured past a box's top edge, each a pixel row's centre: past the ring's band, then at the spread
    /// and one, two and three standard deviations past it.
    pub fn distances(r: &Recipe, from: f64) -> Vec<f64> {
        let sigma = r.blur / 2.0;
        let mut out: Vec<f64> = [from, r.spread, r.spread + sigma, r.spread + 2.0 * sigma, r.spread + 3.0 * sigma]
            .iter()
            .map(|d| d.max(from).floor() + 0.5)
            .collect();
        out.dedup();
        out
    }

    /// The glow's alpha seen at each distance above the middle of a box's top edge, over the scene's black.
    pub fn seen(scene: &mut Scene<'_>, b: (f64, f64, f64, f64), glow: Colour, at: &[f64]) -> Vec<f64> {
        let centre = b.0 + (b.2 / 2.0).floor();
        let points: Vec<(f64, f64)> = at.iter().map(|d| (centre, b.1 - d - 0.5)).collect();
        scene.colours_at(&points).into_iter().map(|c| c.map_or(f64::NAN, |c| share((c.r, c.g, c.b), (glow.r, glow.g, glow.b)))).collect()
    }

    /// What is drawn inside the box of a corner radius: a 5 by 5 grid of pixels two in from each edge (a point is the pixel it
    /// rounds to), each inside the rounded box less a pixel, so its antialiased edge is not read.
    pub fn dark_inside(scene: &mut Scene<'_>, b: (f64, f64, f64, f64), radius: f64) -> Vec<(f64, f64, Colour)> {
        let shape = Recipe { width: b.2 - 2.0, height: b.3 - 2.0, radius: (radius - 1.0).max(0.0), spread: 0.0, blur: 0.0, alpha: 1.0 };
        let mut points = Vec::new();
        for i in 0..5 {
            for j in 0..5 {
                let (x, y) = (b.0 + 2.0 + (b.2 - 5.0) * f64::from(i) / 4.0, b.1 + 2.0 + (b.3 - 5.0) * f64::from(j) / 4.0);
                if shape.inside(x - b.0 - b.2 / 2.0, y - b.1 - b.3 / 2.0) {
                    points.push((x, y));
                }
            }
        }
        let seen = scene.colours_at(&points);
        points.into_iter().zip(seen).filter_map(|((x, y), c)| c.filter(|c| (c.r, c.g, c.b) != (0, 0, 0)).map(|c| (x, y, c))).collect()
    }

    /// The colour just past a glow's reach above the box: the scene's black where the glow has ended.
    pub fn past_reach(scene: &mut Scene<'_>, b: (f64, f64, f64, f64), r: &Recipe) -> Colour {
        let reach = (r.spread + 2.0 * r.blur).ceil() + 1.0;
        scene.colours_at(&[(b.0 + (b.2 / 2.0).floor(), b.1 - reach - 0.5)])[0].expect("a colour")
    }

    /// The visible items of a type or name under a named item.
    pub fn under(tree: &Tree, name: &str, what: &str) -> usize {
        let at = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene"));
        tree.under(at).into_iter().filter(|&i| {
            let n = &tree.nodes[i as usize];
            n.visible && (n.ty == what || n.name == what)
        }).count()
    }

    /// Every visible item of a type or name in the scene.
    pub fn anywhere(tree: &Tree, what: &str) -> usize {
        tree.nodes.iter().filter(|n| n.visible && (n.ty == what || n.name == what)).count()
    }
}
