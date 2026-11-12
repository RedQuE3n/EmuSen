//! The five fit sizes (EmuSen_Qt.md §4.8.5): each watched theme's system view at the SNES and its gamelist at the first
//! game, at the harness's `SIZES`, on the oracle's synthetic library, on the GPU tier. Held: pad reach, pointer reach, no
//! chrome, nothing logged, and every element's rectangle ÷ (W, H) equal within one pixel of the smaller window across the
//! sizes of one aspect ratio (a theme lays itself out per aspect ratio; ES-DE places at whole pixels, EmuSen_Qt.md §4.8.5;
//! R-20261111-88), on an axis ES-DE sizes from the words within half a pixel a letter plus one (R-20261111-112). Each grab
//! is written for the side-by-sides beside ES-DE's captures.

mod common;

use std::collections::BTreeMap;

use common::{until, Scratch};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::worded;
use emusen_esde::loader::View;
use emusen_esde::scene::paint::sized_by_words;
use emusen_qt_test::{Harness, Scene};

/// Which rule an element's box is held to across sizes, axis by axis, and why.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rule {
    /// The horizontal and the vertical axis sized by the words, held to R-20261111-112; else to R-20261111-88's one pixel.
    words: (bool, bool),
    why: &'static str,
}

// What sizes a text-like box in ES-DE 3.4.1: a help strip its entries (HelpComponent::updateGrid), a status row its
// entries' width at the theme's height (SystemStatusComponent::updateGrid, applyTheme), a text, date or clock its words on
// the axes its theme's size leaves at 0 (GuiComponent::applyTheme, TextComponent::onTextChanged; the scene's
// `sized_by_words`). A text whose name the view holds more than once with different rules takes the stricter, one pixel.
// Any other box has no words and is held to one pixel (None).
fn rule_of(cls: &str, name: &str, view: &View) -> Option<Rule> {
    if cls.contains("HelpElement") {
        return Some(Rule { words: (true, true), why: "R-112: a help strip, sized by its entries (HelpComponent::updateGrid)" });
    }
    if cls.contains("StatusElement") {
        return Some(Rule { words: (true, false), why: "R-112 across, R-88 down: a status row, its width its entries' at the theme's height (SystemStatusComponent::updateGrid)" });
    }
    if !cls.contains("TextElement") {
        return None;
    }
    let texts: Vec<(bool, bool)> =
        view.elements.iter().filter(|e| e.name == name && matches!(e.kind(), "text" | "datetime" | "clock")).map(sized_by_words).collect();
    Some(match texts.first() {
        None => Rule { words: (false, false), why: "R-88: no text of that name in the resolved view" },
        Some(&first) if texts.iter().any(|t| *t != first) => Rule { words: (false, false), why: "R-88: texts of that name sized differently" },
        Some((false, false)) => Rule { words: (false, false), why: "R-88: the theme gives its width and height" },
        Some((true, true)) => Rule { words: (true, true), why: "R-112: the theme gives no width or height (size absent or 0 0)" },
        Some((false, true)) => Rule { words: (false, true), why: "R-88 across, R-112 down: the theme gives its width, its height 0" },
        Some((true, false)) => Rule { words: (true, false), why: "R-112 across, R-88 down: the theme gives its height, its width 0" },
    })
}

// A box as a fraction of the window, the letters of the visible words under it, and its rule if it is text-like.
type Boxes = BTreeMap<String, ((f64, f64, f64, f64), usize, Option<Rule>)>;

// Every drawn element's box as a fraction of the window, keyed by its name and its place among elements of that name,
// with the letters of the visible words drawn under it, white space left out, and the rule `view` gives it.
fn fractions(scene: &mut Scene<'_>, w: f64, h: f64, view: &View) -> Boxes {
    let tree = scene.tree();
    let parent: BTreeMap<&str, &str> = tree.nodes.iter().map(|n| (n.id.as_str(), n.parent.as_str())).collect();
    let under = |id: &str, top: &str| {
        let mut at = id;
        while !at.is_empty() {
            if at == top {
                return true;
            }
            at = parent.get(at).copied().unwrap_or("");
        }
        false
    };
    let shown = |top: &str| -> usize {
        tree.nodes.iter().filter(|n| n.visible && under(&n.id, top)).filter_map(|n| n.text.as_deref()).map(worded::letters).sum()
    };
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = Boxes::new();
    for n in tree.nodes.iter().filter(|n| n.visible && n.name.starts_with("element-")) {
        let k = seen.entry(n.name.clone()).or_default();
        let rule = rule_of(&n.cls, &n.name["element-".len()..], view);
        let letters = if rule.is_some_and(|r| r.words != (false, false)) { shown(&n.id) } else { 0 };
        out.insert(format!("{}#{k}", n.name), ((n.x / w, n.y / h, n.w / w, n.h / h), letters, rule));
        *k += 1;
    }
    out
}

// Why a help strip may pass its letters' allowance: ES-DE's icon box is 1.25 × the rows of the hinted S at the help's
// size (es-core/src/components/HelpComponent.cpp, applyTheme's mLetterHeight; EmuSen_Qt.md §4.12.4), whole rows that do
// not scale with the window, and each entry has one. Whether help strips get a wider rule waits on the tester.
const HELP_ICONS: &str = "; a help strip also carries one icon box per entry, 1.25 × the hinted S's whole rows (HelpComponent.cpp applyTheme), which do not scale with the window: whether help strips get a wider rule waits on the tester";

fn ratio(w: i32, h: i32) -> (i32, i32) {
    let g = (1..=w.min(h)).rev().find(|d| w % d == 0 && h % d == 0).unwrap();
    (w / g, h / g)
}

/// What the view puts in the window besides the theme's elements: none (the quick-scroll overlay is hidden at rest).
fn chrome(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let Some(view) = tree.nodes.iter().find(|n| n.name == "view").map(|n| n.id.clone()) else { return vec!["no view".into()] };
    tree.nodes.iter().filter(|n| n.parent == view && n.visible && !n.name.starts_with("element-") && n.cls != "QQuickRepeater").map(|n| format!("{} {}", n.cls, n.name)).collect()
}

/// Points of the window with any transparency: a grid over it and a line through each help bar, whose clear `Rectangle`
/// once wrote alpha 0 over its strip on Vulkan (EmuSen_Qt.md §4.8.5). The view is opaque everywhere.
fn clear_points(scene: &mut Scene<'_>, w: f64, h: f64) -> Vec<(i64, i64)> {
    let mut points: Vec<(f64, f64)> = (0..32).flat_map(|i| (0..20).map(move |j| ((f64::from(i) + 0.5) * w / 32.0, (f64::from(j) + 0.5) * h / 20.0))).collect();
    let tree = scene.tree();
    for n in tree.nodes.iter().filter(|n| n.visible && n.name.starts_with("element-") && n.cls.contains("HelpElement") && n.w > 0.0) {
        points.extend((0..48).map(|i| (n.x + (f64::from(i) + 0.5) * n.w / 48.0, n.y + n.h / 2.0)));
    }
    let colours = scene.colours_at(&points);
    points.iter().zip(colours).filter(|(_, c)| c.is_some_and(|c| c.a < 255)).map(|(p, _)| (p.0 as i64, p.1 as i64)).take(4).collect()
}

/// The help's glyphs not sized so the kit's disc reaches ES-DE's 58 of 64 units of the icon's box (EmuSen_Qt.md §4.12.1).
fn glyphs_off(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let mut off = vec![];
    for n in tree.nodes.iter().filter(|n| n.visible && n.name.starts_with("help-")) {
        let label = &n.name["help-".len()..];
        let Some(g) = tree.nodes.iter().find(|g| g.visible && g.parent == n.id && g.name == format!("helpGlyph-{label}")) else {
            off.push(format!("{label}: no glyph drawn"));
            continue;
        };
        // The theme's picture is drawn at the icon's box whether it is shown or not.
        let side = tree.nodes.iter().find(|p| p.parent == n.id && p.name == format!("helpPicture-{label}")).map_or(f64::NAN, |p| p.w);
        let want = side * 58.0 / 64.0 / emusen_kit_core::glyph::FILLED_DISC_SHARE;
        if !((g.w - want).abs() < 0.01) {
            off.push(format!("{label}: {:.2} against {want:.2} for a box of {side:.2}", g.w));
        }
    }
    off
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
    let texts = common::oracle_texts(&common::ORDER);
    let out = common::compare_dir();
    let mut harness = Harness::new();
    let mut failures = vec![];
    // Every text-like box at every size pair: its edges, its rule axis by axis and why, its letters and the most each axis may move.
    let mut table = vec!["theme\tstate\tsize\tagainst\tbox\tx_px\ty_px\tw_px\th_px\trule\tletters\tallowed_across\tallowed_down\twithin".to_owned()];
    let (w0, h0) = SIZES[0];
    let mut scene = harness.load(common::VIEW, w0, h0);
    for (theme, variant, scheme, font) in common::watched() {
        let folder = common::corpus().join(&theme);
        assert!(folder.join("capabilities.xml").exists(), "{theme} in the corpus at {}", folder.display());
        let (choices, picked) = common::choices_of(&variant, &scheme, &font);
        for view in ["system", "gamelist"] {
            // The clock held at one instant, so its words, and so its box, are the same at every size and every hour.
            assert!(scene.set_property("view", "now", common::HELD_NOW));
            common::install(&mut scene, &folder, &choices, view, &texts, w0, h0);
            let problem = scene.property("esdeModel", "problem");
            if !problem.is_empty() {
                failures.push(format!("{theme}: {problem}"));
            }
            let step = common::along(&folder, &picked, view);
            if view == "gamelist" {
                // The first game: the folder Tower Set is entry 0, as ES-DE lists it.
                scene.pad(step);
            }
            let state = if view == "system" { "system".to_owned() } else { "gamelist-1".to_owned() };
            let mut by_ratio: BTreeMap<(i32, i32), ((i32, i32), Boxes)> = BTreeMap::new();
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
                // The theme as ES-DE resolves it at this size, for which axes its texts' words size.
                let resolved = emusen_esde::loader::load(
                    &folder.to_string_lossy(),
                    &common::oracle_snes(),
                    &emusen_esde::loader::Choices { screen_width: w, screen_height: h, ..picked.clone() },
                    None,
                );
                let boxes = fractions(&mut scene, f64::from(w), f64::from(h), resolved.view(view));
                if boxes.is_empty() {
                    failures.push(format!("{at}: no element drawn"));
                }
                match by_ratio.get(&ratio(w, h)) {
                    None => {
                        by_ratio.insert(ratio(w, h), ((w, h), boxes));
                    }
                    Some((size, held)) => {
                        let first = format!("{}x{}", size.0, size.1);
                        // One pixel of the smaller window: ES-DE 3.4.1 rounds a component's position and its origin's
                        // offset to whole pixels (GuiComponent::getTransform) and an image's corners (ImageComponent::
                        // updateVertices), so each edge is up to half a pixel from proportional at either size.
                        let pixel = (f64::from(w.min(size.0)), f64::from(h.min(size.1)));
                        for (name, b) in &boxes {
                            match held.get(name) {
                                None => failures.push(format!("{at}: {name} drawn here but not at {first}")),
                                Some((a, _, _)) => {
                                    let (b, letters, texty) = b;
                                    let rule = texty.unwrap_or(Rule { words: (false, false), why: "R-88: no words" });
                                    let px = [(b.0 - a.0) * pixel.0, (b.1 - a.1) * pixel.1, (b.2 - a.2) * pixel.0, (b.3 - a.3) * pixel.1];
                                    let allowed = |words: bool| if words { worded::allowance(*letters) } else { 1.0 };
                                    let (across, down) = (allowed(rule.words.0), allowed(rule.words.1));
                                    let off_across = px[0].abs().max(px[2].abs());
                                    let off_down = px[1].abs().max(px[3].abs());
                                    let ok = off_across <= across && off_down <= down;
                                    if texty.is_some() {
                                        table.push(format!(
                                            "{theme}\t{state}\t{w}x{h}\t{first}\t{name}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{}\t{letters}\t{across:.1}\t{down:.1}\t{}",
                                            px[0], px[1], px[2], px[3], rule.why, if ok { "yes" } else { "no" }
                                        ));
                                    }
                                    if !ok {
                                        let why = if name.starts_with("element-help") { HELP_ICONS } else { "" };
                                        failures.push(format!(
                                            "{at}: {name} {b:.4?} against {a:.4?} at {first}, {off_across:.2} px across and {off_down:.2} down at the smaller window, past {across:.1} and {down:.1} ({}; {letters} letters; ES-DE places at whole pixels, GuiComponent::getTransform, and rounds each glyph's advance, Font::shapeText, v3.4.1){why}",
                                            rule.why
                                        ));
                                    }
                                }
                            }
                        }
                        for name in held.keys().filter(|n| !boxes.contains_key(*n)) {
                            failures.push(format!("{at}: {name} drawn at {first} but not here"));
                        }
                    }
                }
                let see_through = clear_points(&mut scene, f64::from(w), f64::from(h));
                if !see_through.is_empty() {
                    failures.push(format!("{at}: the window see-through at {see_through:?}"));
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
                let glyphs = glyphs_off(&mut scene);
                if !glyphs.is_empty() {
                    failures.push(format!("{at}: help glyphs not at ES-DE's disc {glyphs:?}"));
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
                // The two items nearest the middle: one at most is the selected one, so the pointer over the pair selects anew.
                let mid = (f64::from(w) / 2.0, f64::from(h) / 2.0);
                let mut near = items.clone();
                near.sort_by(|a, b| ((a.0 - mid.0).hypot(a.1 - mid.1)).total_cmp(&(b.0 - mid.0).hypot(b.1 - mid.1)));
                let mut selected = false;
                for &(x, y) in near.iter().take(2) {
                    scene.hover(x, y);
                    common::settle_picture(&mut scene, &scratch.0);
                    selected |= scene.difference(&shot, 0, 0, &[], "") != Some(0);
                }
                if !selected {
                    failures.push(format!("{theme} {state}: the pointer over an item selected nothing"));
                }
                scene.pad(UiButton::Back);
                until(&mut scene, "B went back", |s| s.property("esdeModel", "view") == "system" && s.property("esdeModel", "moving") == "false");
            }
            scene.hover(-1.0, -1.0);
        }
    }
    std::fs::write(out.join("worded-boxes.tsv"), table.join("\n") + "\n").unwrap();
    eprintln!("{} failures\n{}", failures.len(), failures.join("\n"));
    scene.assert_quiet();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
