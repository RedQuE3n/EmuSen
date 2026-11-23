//! The table of an Eclipse board and its check against a placed screen, for Home's and Choose a console's oracles (EmuSen_Qt.md §3.4.8).
#![allow(dead_code)]

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::QString;
use emusen_kit_core::tokens::Colour;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::Scene;
use emusen_theme_qt::host;
use std::collections::BTreeMap;

/// One row of the table.
pub struct Row {
    pub name: String,
    pub part: String,
    pub x: String,
    pub y: String,
    pub w: String,
    pub h: String,
    pub radius: String,
    pub colour: String,
    pub size: String,
    pub status: String,
    /// The type as the board's CSS gives it: "weight spacing case" for words, "glyph size weight" or "word size weight".
    pub kind: String,
}

pub fn rows(table: &str) -> (f64, Vec<Row>) {
    let height = table
        .lines()
        .find_map(|l| l.strip_prefix("# board_height: "))
        .and_then(|h| h.trim().parse::<f64>().ok())
        .expect("the table names its board's height");
    let rows = table
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("name\t") && !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            assert_eq!(c.len(), 14, "a row of 14 columns: {l}");
            let s = |i: usize| c[i].to_owned();
            Row { name: s(0), part: s(4), x: s(5), y: s(6), w: s(7), h: s(8), radius: s(9), colour: s(10), size: s(11), status: s(12), kind: s(13) }
        })
        .collect();
    (height, rows)
}

/// A box in canvas units.
#[derive(Clone, Copy, Debug)]
pub struct Boxed {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Boxed {
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn middle(&self) -> f64 {
        self.y + self.h / 2.0
    }
}

/// Equal to a millionth, the float precision of Qt's item-to-scene matrix (EmuSen_Qt.md §3.4.9, the tests' means).
pub fn same(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * b.abs().max(1.0)
}

/// An object of the theme's engine by its number in the tree, its QObject's address.
pub fn object(id: u64) -> Pin<&'static mut QObject> {
    // Safety: the id was read from the live tree, and the screen it names lives until the host loads again.
    unsafe { Pin::new_unchecked(&mut *(id as *mut QObject)) }
}

/// Every element of the built screen by name, in canvas units.
pub fn boxes(tree: &Tree, unit: f64) -> BTreeMap<String, Boxed> {
    let mut out = BTreeMap::new();
    // A name drawn and also held by a hidden item (the frame's own help bar, not shown) is the drawn one's.
    for n in tree.nodes.iter().filter(|n| !n.name.is_empty()) {
        if n.visible || !out.contains_key(&n.name) {
            out.insert(n.name.clone(), Boxed { x: n.x / unit, y: n.y / unit, w: n.w / unit, h: n.h / unit });
        }
    }
    out
}

/// A rule's place on one axis: what the element's edge must be, in canvas units; none for "-".
pub fn rule(spec: &str, k: f64, got: &BTreeMap<String, Boxed>, canvas_width: f64) -> Option<(char, f64)> {
    let of = |n: &str| *got.get(n).unwrap_or_else(|| panic!("{n} is drawn, a rule names it"));
    let parts: Vec<&str> = spec.split(':').collect();
    let num = |s: &str| s.parse::<f64>().unwrap_or_else(|_| panic!("{spec}: {s} is no number"));
    match parts.as_slice() {
        ["-"] => None,
        ["after", n, g] => Some(('x', of(n).right() + num(g) * k)),
        ["in", n, g] => Some(('x', of(n).x + num(g) * k)),
        ["right", g] => Some(('r', canvas_width - num(g) * k)),
        ["fit", n, g] => Some(('r', of(n).right() + num(g) * k)),
        ["inright", n, g] => Some(('r', of(n).right() - num(g) * k)),
        ["before", n, g] => Some(('r', of(n).x - num(g) * k)),
        ["below", n, g] => Some(('y', of(n).bottom() + num(g) * k)),
        ["top", n, g] => Some(('y', of(n).y + num(g) * k)),
        ["foot", n] => Some(('b', of(n).bottom())),
        ["up", n, g] => Some(('b', of(n).bottom() - num(g) * k)),
        ["above", n, g] => Some(('b', of(n).y - num(g) * k)),
        ["mid", g] if g.parse::<f64>().is_ok() => Some(('m', num(g) * k)),
        ["mid", n] => Some(('m', of(n).middle())),
        // A baseline is the words' own, which the loop reads from the tree.
        ["base", _] => None,
        [v] => Some(('=', num(v) * k)),
        _ => panic!("{spec}: no such rule"),
    }
}

/// The tokens of the theme's engine, as Qt writes each as text.
pub fn tokens(screen: u64) -> BTreeMap<String, String> {
    let tokens = host::kit_tokens(object(screen)).expect("the theme's tokens");
    Scene::constants(tokens.upcast_pin()).into_iter().collect()
}

/// A property of a placed screen's element, as Qt writes it as text, the element by its number in the tree (EmuSen_Qt.md §3.4.9).
pub fn read(tree: &Tree, name: &str, property: &str) -> String {
    // A Line's words are named as the Line is: the last of a name in the tree, the deepest, is the words themselves; the drawn
    // one where a hidden item of another screen holds the name too.
    let Some(i) = tree.nodes.iter().rposition(|n| n.name == name && n.visible).or_else(|| tree.nodes.iter().rposition(|n| n.name == name)) else { return String::new() };
    let id: u64 = tree.nodes[i as usize].id.parse().unwrap();
    emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(name), &QString::from(property)).to_string()
}

/// A QFont's weight, case, absolute letter spacing and sizes, from the text Qt writes for it (EmuSen_Qt.md §3.4.9).
pub fn font_of(text: &str) -> Option<(f64, f64, i32, i32, f64)> {
    let f: Vec<&str> = text.split(',').collect();
    let n = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
    Some((n(2)?, n(4)?, n(10)? as i32, n(11)? as i32, n(12)?))
}

/// What differs from a row's type: its words' weight, spacing and case, or its glyph's size and weight.
pub fn typed(tree: &Tree, row: &Row, k: f64, unit: f64, tokens: &BTreeMap<String, String>) -> Vec<String> {
    let parts: Vec<&str> = row.kind.split(' ').collect();
    let mut out = Vec::new();
    // The drawn one where a hidden item holds the name too (the frame's own help bar, not shown).
    let node = tree.nodes.iter().rposition(|n| n.name == row.name && n.visible).or_else(|| tree.nodes.iter().rposition(|n| n.name == row.name)).expect("the row is drawn");
    // The texts the row's type holds: a help line's entries' words, else the row's own words.
    let texts: Vec<usize> = if row.part == "HelpBar" {
        let mut inside = Vec::new();
        for (i, n) in tree.nodes.iter().enumerate() {
            let mut at = n.parent.clone();
            while let Some(p) = tree.nodes.iter().find(|p| p.id == at) {
                if p.id == tree.nodes[node].id {
                    if n.visible && n.text.as_deref().is_some_and(|t| !t.is_empty()) && n.ty.contains("Text") {
                        inside.push(i);
                    }
                    break;
                }
                at = p.parent.clone();
            }
        }
        inside
    } else {
        vec![node]
    };
    let prop = |i: usize, p: &str| {
        let id: u64 = tree.nodes[i].id.parse().unwrap();
        emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(tree.nodes[i].name.as_str()), &QString::from(p)).to_string()
    };
    match parts.as_slice() {
        [kind @ ("glyph" | "word"), size, weight, shape @ ..] => {
            // The glyph's own measures, in board pixels: the Glyph under the row, or the row itself.
            let glyph = std::iter::once(node)
                .chain(tree.nodes.iter().enumerate().filter(|(_, n)| n.parent == tree.nodes[node].id).map(|(i, _)| i))
                .find(|&i| tree.nodes[i].cls.contains("Glyph") || tree.nodes[i].ty.contains("Glyph"))
                .unwrap_or(node);
            let lead = prop(glyph, "lead") == "true";
            let (size_of, weight_of) = match (*kind, lead) {
                ("word", _) => (prop(glyph, "wordSize"), "400".to_owned()),
                (_, true) => (prop(glyph, "leadLetterSize"), prop(glyph, "leadWeight")),
                (_, false) => (prop(glyph, "letterSize"), "400".to_owned()),
            };
            if size_of.parse::<f64>().ok() != size.parse::<f64>().ok() {
                out.push(format!("glyph's {kind} at {size_of} where the board gives {size}"));
            }
            if weight_of != *weight {
                out.push(format!("glyph's weight {weight_of} where the board gives {weight}"));
            }
            // A named glyph's corner and its padding either side of the name, in board pixels, where the row gives them.
            for (p, v) in ["corner", "inset"].iter().zip(shape) {
                let (have, want) = (prop(glyph, p).parse::<f64>().unwrap_or(f64::NAN), v.parse::<f64>().unwrap() * k);
                if !same(have, want) {
                    out.push(format!("glyph's {p} {have} where the board gives {want}"));
                }
            }
        }
        [weight, em, case @ ("upper" | "none")] => {
            let want_weight: f64 = weight.parse().unwrap();
            let em: f64 = em.parse().unwrap();
            for &i in &texts {
                let Some((px, w, caps, spacing_type, spacing)) = font_of(&prop(i, "font")) else {
                    out.push(format!("has no font to read: {:?}", prop(i, "font")));
                    continue;
                };
                if w != want_weight {
                    out.push(format!("weight {w} where the board gives {weight}"));
                }
                // An absolute spacing in the Text's own pixels, the board's em of its size, held within Qt's 64th.
                let want = em * px;
                if (spacing - want).abs() > 1.0 / 64.0 || (em != 0.0 && spacing_type != 1) {
                    out.push(format!("letter spacing {spacing} (type {spacing_type}) where the board gives {want}"));
                }
                if (caps == 1) != (*case == "upper") {
                    out.push(format!("case {caps} where the board gives {case}"));
                }
            }
            if texts.is_empty() {
                out.push("has no words drawn".to_owned());
            }
        }
        ["field", turn, corner, _, _] => {
            let number = |p: &str| prop(node, p).parse::<f64>().unwrap_or(f64::NAN);
            let (turn, corner): (f64, f64) = (turn.parse().unwrap(), corner.parse().unwrap());
            if !same(number("rotation"), turn) {
                out.push(format!("turned {} where the board turns it {turn}", number("rotation")));
            }
            // The shape drawn: the ellipse its box holds, or the box at the board's corner in canvas units; the other hidden.
            let under = |name: &str| {
                let id = tree.nodes[node].id.clone();
                tree.nodes.iter().position(|n| {
                    n.name == name && {
                        let mut at = n.parent.clone();
                        let mut inside = false;
                        while let Some(p) = tree.nodes.iter().find(|p| p.id == at) {
                            if p.id == id {
                                inside = true;
                                break;
                            }
                            at = p.parent.clone();
                        }
                        inside
                    }
                })
            };
            let (shown, hidden) = if corner < 0.0 { ("backdropEllipse", "backdropBox") } else { ("backdropBox", "backdropEllipse") };
            match (under(shown), under(hidden)) {
                (Some(a), Some(b)) if tree.nodes[a].visible && !tree.nodes[b].visible => {
                    let n = |p: &str| prop(a, p).parse::<f64>().unwrap_or(f64::NAN);
                    let want = if corner < 0.0 { n("width") / 2.0 } else { corner * k };
                    if !same(n("radius"), want) {
                        out.push(format!("{shown}'s corner {} where the board gives {want}", n("radius")));
                    }
                }
                _ => out.push(format!("is not drawn as the board's {shown}")),
            }
        }
        ["glow", blur, spread, dx, dy] => {
            for (p, v) in [("blur", blur), ("spread", spread), ("offsetX", dx), ("offsetY", dy)] {
                let (have, want) = (prop(node, p).parse::<f64>().unwrap_or(f64::NAN), v.parse::<f64>().unwrap() * k);
                if !same(have, want) {
                    out.push(format!("{p} {have} where the board gives {want}"));
                }
            }
            // The Gaussian runs at the scene's pixels: a canvas unit's.
            let scale = prop(node, "pixelScale").parse::<f64>().unwrap_or(f64::NAN);
            if !same(scale, unit) {
                out.push(format!("blurred at {scale} pixels a unit where the scene draws {unit}"));
            }
        }
        ["ring", gap, thickness, blur, spread, band, glow, gap_colour, ring_colour] => {
            for (p, v) in [("gap", gap), ("thickness", thickness), ("glowBlur", blur), ("glowSpread", spread), ("bandRadius", band)] {
                let (have, want) = (prop(node, p).parse::<f64>().unwrap_or(f64::NAN), v.parse::<f64>().unwrap() * k);
                if !same(have, want) {
                    out.push(format!("{p} {have} where the board gives {want}"));
                }
            }
            if Colour::parse(&prop(node, "glowColor")) != Colour::parse(glow) {
                out.push(format!("glow colour {} where the board gives {glow}", prop(node, "glowColor")));
            }
            // The gap is filled in the ring's gap colour, the ring in the accent: the kit's tokens, which must be the board's.
            for (token, want) in [("ringGapColor", gap_colour), ("accent", ring_colour)] {
                if tokens.get(token).and_then(|v| Colour::parse(v)) != Colour::parse(want) {
                    out.push(format!("{token} {:?} where the board gives {want}", tokens.get(token)));
                }
            }
            // The gap and the band drawn round the tile by their widths, and the glow under them at the scene's pixels.
            let child = |name: &str| {
                let mut found = None;
                for (i, n) in tree.nodes.iter().enumerate() {
                    if n.name == name && n.parent == tree.nodes[node].id {
                        found = Some(i);
                    }
                }
                found
            };
            let box_of = |i: usize| (tree.nodes[i].x / unit, tree.nodes[i].y / unit, tree.nodes[i].w / unit, tree.nodes[i].h / unit);
            let ring = box_of(node);
            for (name, out_by) in [("focusGap", gap.parse::<f64>().unwrap()), ("focusBand", gap.parse::<f64>().unwrap() + thickness.parse::<f64>().unwrap())] {
                let Some(i) = child(name) else {
                    out.push(format!("draws no {name}"));
                    continue;
                };
                let (x, y, w, h) = box_of(i);
                let d = out_by * k;
                if !(same(x, ring.0 - d) && same(y, ring.1 - d) && same(w, ring.2 + 2.0 * d) && same(h, ring.3 + 2.0 * d)) || !tree.nodes[i].visible {
                    out.push(format!("{name} at {:?} where the board grows the tile {d} on each side", (x, y, w, h)));
                }
            }
            match child("focusGlow") {
                Some(i) => {
                    let scale = prop(i, "pixelScale").parse::<f64>().unwrap_or(f64::NAN);
                    if !same(scale, unit) {
                        out.push(format!("glow blurred at {scale} pixels a unit where the scene draws {unit}"));
                    }
                }
                None => out.push("draws no focusGlow".to_owned()),
            }
        }
        ["line", width, colour] => {
            // A box's line inside its edge (CSS's border-box): its width in board pixels and its colour.
            let (have, want) = (prop(node, "border.width").parse::<f64>().unwrap_or(f64::NAN), width.parse::<f64>().unwrap() * k);
            if !same(have, want) {
                out.push(format!("line {have} wide where the board gives {want}"));
            }
            if want > 0.0 && Colour::parse(&prop(node, "border.color")) != Colour::parse(colour) {
                out.push(format!("line coloured {} where the board gives {colour}", prop(node, "border.color")));
            }
        }
        ["corona", blur, spread, alpha] => {
            // A glow of the disc: its blur radius and spread in board pixels, its colour's strength as the board's rgba gives it.
            for (p, v) in [("blur", blur), ("spread", spread)] {
                let (have, want) = (prop(node, p).parse::<f64>().unwrap_or(f64::NAN), v.parse::<f64>().unwrap() * k);
                if !same(have, want) {
                    out.push(format!("{p} {have} where the board gives {want}"));
                }
            }
            let a = Colour::parse(&prop(node, "color")).map_or(f64::NAN, |c| f64::from(c.a) / 255.0);
            // Half a level of 255: the colour is held as 8 bits a channel.
            if (a - alpha.parse::<f64>().unwrap()).abs() > 0.5 / 255.0 {
                out.push(format!("strength {a} where the board gives {alpha}"));
            }
        }
        _ => panic!("{}: no such type {:?}", row.name, row.kind),
    }
    out
}

/// The colour an element paints: a pill's, a mark's, a field's, a disc's fill, a help line's words, else its own colour.
pub fn painted(tree: &Tree, name: &str) -> Option<Colour> {
    ["paint", "ink", "fill", "labelColor", "color"].iter().find_map(|p| Colour::parse(&read(tree, name, p))).or_else(|| {
        // A Shape's: its first path's, as the tree records each path's fill or stroke.
        tree.nodes.iter().rev().find(|n| n.name == name && n.visible).and_then(|n| n.colors.first()).and_then(|c| Colour::parse(c))
    })
}

/// Every built row held to the tree at size `at`, each difference pushed; `may_hide` allows a hidden tile past the window's edge.
pub fn held(
    tree: &Tree,
    rows: &[Row],
    k: f64,
    unit: f64,
    canvas_width: f64,
    tokens: &BTreeMap<String, String>,
    at: &str,
    may_hide: &dyn Fn(&str, f64, f64) -> bool,
    differences: &mut Vec<String>,
) {
    let mut got = boxes(tree, unit);
    let named = |n: &str| tree.nodes.iter().rev().find(|m| m.name == n && m.visible).or_else(|| tree.nodes.iter().rev().find(|m| m.name == n));
    // A "Line" row is held by Eclipse's Line itself, the box a board's CSS gives the line, not by its words within it, which a
    // board line stands on Chrome's baseline (themes/eclipse/parts/Line.qml boardLine): the outer visible item of the name.
    let outer = |n: &str| tree.nodes.iter().find(|m| m.name == n && m.visible && m.cls.starts_with("Line"));
    for row in rows.iter().filter(|r| r.part == "Line") {
        if let Some(line) = outer(&row.name) {
            got.insert(row.name.clone(), Boxed { x: line.x / unit, y: line.y / unit, w: line.w / unit, h: line.h / unit });
        }
    }
    // A ShapePath row is a path, no item of the tree: its screen's own test holds it.
    for row in rows.iter().filter(|r| r.part != "ShapePath") {
        if row.status != "built" {
            // Not drawn: no element of that name, or an empty place for a part still to be ruled on.
            for node in tree.nodes.iter().filter(|n| n.name == row.name || n.name.starts_with(&format!("{}.", row.name))) {
                let children = tree.nodes.iter().filter(|c| c.parent == node.id).count();
                if node.ty != "QQuickItem" || children > 0 {
                    differences.push(format!("{at}: {} is drawn ({}), the program has nothing for it", row.name, node.ty));
                }
            }
            continue;
        }
        let Some(node) = named(&row.name) else {
            differences.push(format!("{at}: {} is not drawn", row.name));
            continue;
        };
        let mut b = got[&row.name];
        // A turned element is held by its own box about its middle, in canvas units, as the board places it before it turns.
        if read(tree, &row.name, "rotation").parse::<f64>().is_ok_and(|r| r != 0.0) {
            let (w, h): (f64, f64) = (read(tree, &row.name, "width").parse().unwrap(), read(tree, &row.name, "height").parse().unwrap());
            b = Boxed { x: b.x + (b.w - w) / 2.0, y: b.y + (b.h - h) / 2.0, w, h };
        }
                if !node.visible {
            // A tile past the window's edge is not drawn; one that would fit must be.
            let x = rule(&row.x, k, &got, canvas_width).map(|r| r.1).unwrap_or(b.x);
            if !may_hide(&row.name, x, b.w) {
                differences.push(format!("{at}: {} is hidden", row.name));
            }
            continue;
        }
        let part_of = if row.part == "Line" { outer(&row.name).unwrap_or(node) } else { node };
        if row.part != "-" && !part_of.ty.contains(&row.part) && !part_of.cls.contains(&row.part) {
            differences.push(format!("{at}: {} is drawn by {} ({}), not {}", row.name, part_of.ty, part_of.cls, row.part));
        }
        for (spec, axis) in [(&row.x, 'x'), (&row.y, 'y'), (&row.w, 'w'), (&row.h, 'h')] {
            if let Some(n) = spec.strip_prefix("base:") {
                // The words' baseline in canvas units: the Text's own offset, in board pixels, through the board's measure.
                let base = |name: &str| read(tree, name, "baselineOffset").parse::<f64>().unwrap_or(f64::NAN) * k;
                let (want, have) = (got[n].y + base(n), b.y + base(&row.name));
                if !same(have, want) {
                    differences.push(format!("{at}: {} baseline {have} where the board gives {want} ({spec})", row.name));
                }
                continue;
            }
            let Some((kind, want)) = rule(spec, k, &got, canvas_width) else { continue };
            let (what, have) = match (kind, axis) {
                ('=', 'x') | ('x', _) => ("x", b.x),
                ('=', 'y') | ('y', _) => ("y", b.y),
                ('=', 'w') => ("width", b.w),
                ('=', 'h') => ("height", b.h),
                ('r', _) => ("right edge", b.right()),
                ('b', _) => ("bottom", b.bottom()),
                ('m', _) => ("middle", b.middle()),
                _ => unreachable!(),
            };
            if !same(have, want) {
                differences.push(format!("{at}: {} {what} {have} where the board gives {want} ({spec})", row.name));
            }
        }
        if let Ok(radius) = row.radius.parse::<f64>() {
            // A radius is in the element's own units, the canvas's.
            let have: f64 = read(tree, &row.name, "radius").parse().unwrap_or(f64::NAN);
            if !same(have, radius * k) {
                differences.push(format!("{at}: {} radius {have} where the board gives {}", row.name, radius * k));
            }
        }
        if let Some(opacity) = row.colour.strip_prefix("opacity:") {
            let want: f64 = opacity.parse().unwrap();
            if !same(node.opacity, want) {
                differences.push(format!("{at}: {} opacity {} where the table gives {want}", row.name, node.opacity));
            }
        } else if row.colour != "-" {
            let want = match row.colour.strip_prefix("Tokens.") {
                Some(t) => tokens.get(t).and_then(|v| Colour::parse(v)),
                None => Colour::parse(&row.colour),
            };
            let have = painted(tree, &row.name);
            if want.is_none() || have != want {
                differences.push(format!("{at}: {} paints {have:?} where the board gives {} ({want:?})", row.name, row.colour));
            }
        }
        if let Some(blur) = row.size.strip_prefix("blur:") {
            // A field's blur, the Gaussian's deviation in canvas units (Main's px through the board's measure), as the field holds it.
            let want = blur.parse::<f64>().unwrap() * k;
            let have = read(tree, &row.name, "deviation").parse::<f64>().unwrap_or(f64::NAN);
            if !same(have, want) {
                differences.push(format!("{at}: {} blur {have} where the table gives {want}", row.name));
            }
        }
        if let Ok(size) = row.size.parse::<f64>() {
            // The words' size in the Text's own units, board pixels, as the harness reads the font; a help line's are its entries'.
            let have = if row.part == "HelpBar" {
                let inside = |n: &emusen_qt_test::tree::Node| {
                    let mut at = n.parent.clone();
                    while let Some(p) = tree.nodes.iter().find(|p| p.id == at) {
                        if p.id == node.id {
                            return true;
                        }
                        at = p.parent.clone();
                    }
                    false
                };
                let sizes: Vec<f64> = tree.nodes.iter().filter(|n| n.visible && n.text.as_deref().is_some_and(|t| !t.is_empty()) && inside(n)).map(|n| n.size).collect();
                if sizes.iter().all(|s| *s == sizes[0]) { sizes.first().copied().unwrap_or(f64::NAN) } else { f64::NAN }
            } else {
                node.size
            };
            // The board's size exactly, in board pixels, the Text scaled by the board's measure (R-20261115-24).
            if have != size {
                differences.push(format!("{at}: {} type {have} where the board gives {size}", row.name));
            }
        }
    }

    // Each text's weight, letter spacing and case, and each glyph's letter size and weight, as the board's CSS gives them.
    for row in rows.iter().filter(|r| r.status == "built" && r.kind != "-" && r.part != "ShapePath") {
        for problem in typed(tree, row, k, unit, tokens) {
            differences.push(format!("{at}: {} {problem}", row.name));
        }
    }
}
