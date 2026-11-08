//! The five fit sizes (EmuSen_Qt.md §4.8.5): each watched theme's system view at the SNES and its gamelist at the first
//! game, at the harness's `SIZES`, on the oracle's synthetic library, on the GPU tier. Held: pad reach, pointer reach, no
//! chrome, nothing logged, and every element's rectangle ÷ (W, H) equal within 0.001 across the sizes of one aspect
//! ratio (a theme lays itself out per aspect ratio). Each grab is written for the side-by-sides beside ES-DE's captures.

mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::{settings, until, Scratch};
use emusen_esde::loader::{self, Choices, ThemeSystem};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};

// The systems in the order ES-DE 3.4.1 shows them on the oracle's library (its capture of Art Book Next's carousel, 2026-11-08), the SNES first.
const ORDER: [(&str, &str); 5] =
    [("snes", "Super Nintendo"), ("n64", "Nintendo 64"), ("nes", "Nintendo Entertainment System"), ("gb", "Game Boy"), ("gbc", "Game Boy Color")];

// A box and whether the painter sized it by its laid-out words (a help bar, a status row, an auto-sized text).
type Boxes = BTreeMap<String, ((f64, f64, f64, f64), bool)>;

// The share of a measured extent that text layout may differ by between sizes: each run's advance is rounded to whole
// pixels and hinted at its own size, so a 14 px help bar is not exactly two thirds of a 21 px one (EmuSen_Qt.md §4.8.5).
const WORDS: f64 = 0.04;

// Every drawn element's box as a fraction of the window, keyed by its name and its place among elements of that name.
fn fractions(scene: &mut Scene<'_>, w: f64, h: f64) -> Boxes {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = Boxes::new();
    for n in scene.tree().nodes.iter().filter(|n| n.visible && n.name.starts_with("element-")) {
        let k = seen.entry(n.name.clone()).or_default();
        let worded = ["HelpElement", "StatusElement", "TextElement"].iter().any(|c| n.cls.contains(c));
        out.insert(format!("{}#{k}", n.name), ((n.x / w, n.y / h, n.w / w, n.h / h), worded));
        *k += 1;
    }
    out
}

fn ratio(w: i32, h: i32) -> (i32, i32) {
    let g = (1..=w.min(h)).rev().find(|d| w % d == 0 && h % d == 0).unwrap();
    (w / g, h / g)
}

// A step along a view's primary element, as the painter reads its axis: down a list or a vertical carousel, else right.
fn along(theme: &Path, choices: &Choices, view: &str) -> UiButton {
    let t = loader::load(&theme.to_string_lossy(), &ThemeSystem::new("snes", "Super Nintendo", "snes"), choices, None);
    let vertical = match t.view(view).primary() {
        Some(p) if p.kind() == "textlist" => true,
        Some(p) if p.kind() == "carousel" => p.string("type").is_some_and(|t| t.starts_with("vertical")),
        _ => false,
    };
    if vertical {
        UiButton::Down
    } else {
        UiButton::Right
    }
}

/// What the view puts in the window besides the theme's elements: none (the quick-scroll overlay is hidden at rest).
fn chrome(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let Some(view) = tree.nodes.iter().find(|n| n.name == "view").map(|n| n.id.clone()) else { return vec!["no view".into()] };
    tree.nodes.iter().filter(|n| n.parent == view && n.visible && !n.name.starts_with("element-") && n.cls != "QQuickRepeater").map(|n| format!("{} {}", n.cls, n.name)).collect()
}

/// The primary's items and the help's entries the pointer can reach: their centres inside the window.
fn unreachable(scene: &mut Scene<'_>, w: f64, h: f64) -> (Vec<String>, Vec<(f64, f64)>) {
    let tree = scene.tree();
    let mut bad = vec![];
    let mut items = vec![];
    for n in tree.nodes.iter().filter(|n| n.visible && n.w > 0.0 && n.h > 0.0) {
        let item = n.name.starts_with("slot-") || n.name.starts_with("row-") || n.name.starts_with("cell-");
        let entry = n.name.starts_with("help-");
        if !item && !entry {
            continue;
        }
        let (cx, cy) = (n.x + n.w / 2.0, n.y + n.h / 2.0);
        if item && (0.0..w).contains(&cx) && (0.0..h).contains(&cy) {
            items.push((cx, cy));
        }
        if entry && !((0.0..w).contains(&cx) && (0.0..h).contains(&cy) && (n.hit || n.operable)) {
            bad.push(format!("{} at {cx:.0},{cy:.0}", n.name));
        }
    }
    (bad, items)
}

#[test]
fn the_watched_themes_hold_at_the_five_sizes() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    assert!(common::oracle_library().join("gamelists").exists(), "the oracle's library: Testing/esde-oracle/library.py");
    let _kwin = common::start();
    let scratch = Scratch::new("sizes");
    let (shelves, media, presence) = common::oracle_texts(&ORDER);
    let out = common::compare_dir();
    let mut harness = Harness::new();
    let mut failures = vec![];
    let (w0, h0) = SIZES[0];
    let mut scene = harness.load(common::VIEW, w0, h0);
    for (theme, variant, scheme, font) in common::watched() {
        let folder = common::corpus().join(&theme);
        assert!(folder.join("capabilities.xml").exists(), "{theme} in the corpus at {}", folder.display());
        let mut choices = format!("\"Variant\":\"{variant}\",\"FontSize\":\"{font}\"");
        if let Some(s) = &scheme {
            choices += &format!(",\"ColorScheme\":\"{s}\"");
        }
        let picked = Choices { variant: Some(variant.clone()), color_scheme: scheme.clone(), font_size: Some(font.clone()), ..Choices::default() };
        for view in ["system", "gamelist"] {
            let text = settings(&folder, &format!("\"DisplayClock\":true,\"StartupView\":\"{view}\"")).replacen('{', &format!("{{\"choices\":{{{choices}}},"), 1);
            assert!(scene.set_property("view", "shelves", "[]"));
            until(&mut scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
            scene.resize(w0, h0);
            assert!(scene.set_property("view", "media", &media));
            assert!(scene.set_property("view", "presence", &presence));
            assert!(scene.set_property("view", "settings", &text));
            assert!(scene.set_property("view", "shelves", &shelves));
            until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
            let problem = scene.property("esdeModel", "problem");
            if !problem.is_empty() {
                failures.push(format!("{theme}: {problem}"));
            }
            let step = along(&folder, &picked, view);
            if view == "gamelist" {
                // The first game: the folder Tower Set is entry 0, as ES-DE lists it.
                scene.pad(step);
            }
            let state = if view == "system" { "system".to_owned() } else { "gamelist-1".to_owned() };
            let mut by_ratio: BTreeMap<(i32, i32), (String, Boxes)> = BTreeMap::new();
            for (w, h) in SIZES {
                let before = scene.property("esdeModel", "revision");
                let at = |s: &mut Scene<'_>, p: &str| s.property("view", p).parse::<f64>().unwrap_or(-1.0);
                let fresh = at(&mut scene, "width") != f64::from(w) || at(&mut scene, "height") != f64::from(h);
                scene.resize(w, h);
                // A new size is installed when its load comes back and publishes; until then the old frame stands.
                if fresh {
                    until(&mut scene, "the size's install", |s| s.property("esdeModel", "revision") != before);
                }
                until(&mut scene, "the size's install", |s| s.property("esdeModel", "themed") == "true");
                if !common::settle_picture(&mut scene, &scratch.0) {
                    failures.push(format!("{theme} {state} {w}x{h}: the picture did not settle in 10 s"));
                }
                assert!(scene.grab(&out.join(format!("{theme}-{state}-{w}x{h}.png")).to_string_lossy()));
                let at = format!("{theme} {state} {w}x{h}");
                let boxes = fractions(&mut scene, f64::from(w), f64::from(h));
                if boxes.is_empty() {
                    failures.push(format!("{at}: no element drawn"));
                }
                match by_ratio.get(&ratio(w, h)) {
                    None => {
                        by_ratio.insert(ratio(w, h), (format!("{w}x{h}"), boxes));
                    }
                    Some((first, held)) => {
                        for (name, b) in &boxes {
                            match held.get(name) {
                                None => failures.push(format!("{at}: {name} drawn here but not at {first}")),
                                Some((a, worded)) => {
                                    let (b, _) = b;
                                    let words = |extent: f64| if *worded { WORDS * extent } else { 0.0 };
                                    let off = [(b.0 - a.0, a.2), (b.1 - a.1, a.3), (b.2 - a.2, a.2), (b.3 - a.3, a.3)]
                                        .iter()
                                        .fold(0.0f64, |m, (d, extent)| m.max(d.abs() - words(*extent)));
                                    if off > 0.001 {
                                        failures.push(format!("{at}: {name} {b:.4?} against {a:.4?} at {first}, {off:.4} off"));
                                    }
                                }
                            }
                        }
                        for name in held.keys().filter(|n| !boxes.contains_key(*n)) {
                            failures.push(format!("{at}: {name} drawn at {first} but not here"));
                        }
                    }
                }
                let extra = chrome(&mut scene);
                if !extra.is_empty() {
                    failures.push(format!("{at}: chrome {extra:?}"));
                }
                let (bad, items) = unreachable(&mut scene, f64::from(w), f64::from(h));
                if !bad.is_empty() {
                    failures.push(format!("{at}: help entries out of the pointer's reach {bad:?}"));
                }
                if items.is_empty() {
                    failures.push(format!("{at}: no item of the primary element in the window"));
                }
            }

            // Pad reach at the last size: a step along the primary moves the picture, A opens or plays, B goes back.
            let shot = scratch.path("pad.png").to_string_lossy().into_owned();
            assert!(scene.grab(&shot));
            scene.pad(step);
            common::settle_picture(&mut scene, &scratch.0);
            if scene.difference(&shot, 0, 0, &[], "") == Some(0) {
                failures.push(format!("{theme} {state}: a step along the primary changed nothing"));
            }
            if view == "system" {
                scene.pad(UiButton::Accept);
                until(&mut scene, "A opened the gamelist", |s| s.property("esdeModel", "view") == "gamelist" && s.property("esdeModel", "moving") == "false");
                scene.pad(UiButton::Back);
                until(&mut scene, "B came back", |s| s.property("esdeModel", "view") == "system" && s.property("esdeModel", "moving") == "false");
            } else {
                // Pointer reach: the item under the pointer is selected as the pad's step selects it.
                let (w, h) = SIZES[SIZES.len() - 1];
                let (_, items) = unreachable(&mut scene, f64::from(w), f64::from(h));
                assert!(scene.grab(&shot));
                // The item nearest the middle that is not the selected one, clear of the help and the edges.
                let mid = (f64::from(w) / 2.0, f64::from(h) / 2.0);
                let mut near = items.clone();
                near.sort_by(|a, b| ((a.0 - mid.0).hypot(a.1 - mid.1)).total_cmp(&(b.0 - mid.0).hypot(b.1 - mid.1)));
                if let Some(&(x, y)) = near.get(1).or(near.first()) {
                    scene.hover(x, y);
                    common::settle_picture(&mut scene, &scratch.0);
                    if scene.difference(&shot, 0, 0, &[], "") == Some(0) {
                        failures.push(format!("{theme} {state}: the pointer over an item selected nothing"));
                    }
                }
                scene.pad(UiButton::Back);
                until(&mut scene, "B went back", |s| s.property("esdeModel", "view") == "system" && s.property("esdeModel", "moving") == "false");
            }
            scene.hover(-1.0, -1.0);
        }
    }
    eprintln!("{} failures\n{}", failures.len(), failures.join("\n"));
    scene.assert_quiet();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
