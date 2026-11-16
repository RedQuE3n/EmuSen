//! The oracle of Eclipse's Home, run by eclipse_home.rs on the software backend and by eclipse_home_gpu.rs on the card.

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::QString;
use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::Colour;
use emusen_kit_qt::kinds;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{refresh, LibrarySource, LibraryView};
use emusen_qt_models::words;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::Scene;
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::host;
use emusen_theme_qt::session::SessionFacts;
use serde_json::json;
use std::collections::BTreeMap;

const TABLE: &str = include_str!("../eclipse_home.tsv");
const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");

/// The board's six games in Recent's order, each with an extension only its console lists; the first is the one suspended.
const GAMES: [(&str, &str); 6] = [
    ("Aurora Drift", "sfc"),
    ("Lumen Garden", "gb"),
    ("Tower Set", "nes"),
    ("Cobalt Harbor", "z64"),
    ("Ember Circuit", "sfc"),
    ("Juniper Vale", "gb"),
];

/// Qt's key codes for the arrows, which a theme's walk answers as the pad's.
const UP: i32 = 0x0100_0013;
const RIGHT: i32 = 0x0100_0014;
const LEFT: i32 = 0x0100_0012;
const DOWN: i32 = 0x0100_0015;

/// The moment the board shows, day 20000 at 21:41 in the zone the oracle sets, and the suspended game's pause the day
/// before at 21:12 (Main.dc.html:49, 56, 59).
const NOW: i64 = 20_000 * 86_400 + 21 * 3600 + 41 * 60;
const PAUSED: i64 = NOW - 86_400 - 29 * 60;

/// The theme host and one slot filling the window, as theme-audit's scene places a screen.
const SCENE: &str = r##"import QtQuick
import EmuSen.ThemeHost
Window {
    id: scene
    objectName: "scene"
    visible: false
    width: 1280
    height: 800
    color: "black"
    property bool placed: false
    ThemeHost {
        id: host
        objectName: "host"
        onGenerationChanged: scene.placed = host.ready && host.place(ThemeHost.Home, slot)
    }
    // The paused game's picture, which the host gives Session: two bands, as the board's stand-in for it draws (Main:58).
    Rectangle {
        objectName: "pausedFrame"
        width: 640
        height: 400
        color: "#1FC9A8"
        Rectangle { y: 400 - 26 * 400 / 60; width: 640; height: 26 * 400 / 60; color: "#6A4CF0" }
    }
    Item {
        id: slot
        objectName: "slot"
        anchors.fill: parent
    }
}
"##;

/// One row of the table.
struct Row {
    name: String,
    part: String,
    x: String,
    y: String,
    w: String,
    h: String,
    radius: String,
    colour: String,
    size: String,
    status: String,
    /// The type as the board's CSS gives it: "weight spacing case" for words, "glyph size weight" or "word size weight".
    kind: String,
}

fn rows() -> (f64, Vec<Row>) {
    let height = TABLE
        .lines()
        .find_map(|l| l.strip_prefix("# board_height: "))
        .and_then(|h| h.trim().parse::<f64>().ok())
        .expect("the table names its board's height");
    let rows = TABLE
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
struct Boxed {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Boxed {
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
    fn middle(&self) -> f64 {
        self.y + self.h / 2.0
    }
}

/// Equal but for the last bits of the scene's arithmetic: Qt maps an item to the scene through a matrix of floats, whose
/// precision is a part in about 1.7e7 (seen 2026-11-15: 64.0000011 for 64 at 3840 by 2160), so a millionth is that and no more.
fn same(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * b.abs().max(1.0)
}

fn rom(world: &World, title: &str, ext: &str) -> String {
    world.dir("Roms").join(format!("{title}.{ext}")).to_string_lossy().into_owned()
}

/// The covers' size in pixels, three by four, as theme-audit's library draws them.
const COVER: (u32, u32) = (450, 600);

/// The six games written, played an hour apart with the first the latest, each but the last with a cover in an ES-DE media
/// folder, and read through the real clients; the media folder, which the host lets a theme show.
fn synthetic(world: &World, harness: &mut emusen_qt_test::Harness) -> (LibraryView, std::path::PathBuf) {
    for (title, ext) in GAMES {
        std::fs::write(rom(world, title, ext), [0u8; 4096]).unwrap();
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.root.join("Roms").to_string_lossy());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let none = Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) };
    let source = ClientSource::open(&library().to_string_lossy(), &none).unwrap();
    source.games().unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let file = platform.file(FILE_GAMES).unwrap();
    let records = Library::open(&platform, &file, &sqlite).unwrap();
    let now = words::now();
    for (i, (title, ext)) in GAMES.iter().enumerate() {
        let at = now - 3600 * (i as i64 + 1);
        records.change(&json!({ "op": "started", "path": rom(world, title, ext), "at": words::utc(at) })).unwrap();
    }
    // Each game's cover but the last's under its console's id, which a first reading names; the last has no art.
    let mut first = LibraryView::empty();
    first.apply(refresh(&source));
    let media = world.dir("Media");
    let covers: Vec<_> = GAMES[..GAMES.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, (title, ext))| {
            let system = first.game(&rom(world, title, ext)).expect("a game of the library").system;
            (media.join(&system).join("covers").join(format!("{title}.png")), COVER, i)
        })
        .collect();
    emusen_qt_test::covers::draw_art(harness, &world.dir("Covers"), &covers, |_| {});
    settings["EsdeMediaDirectory"] = json!(media.to_string_lossy());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    drop(source);
    let source = ClientSource::open(&library().to_string_lossy(), &none).unwrap();
    let mut view = LibraryView::empty();
    view.apply(refresh(&source));
    assert!(view.problem.is_empty(), "{}", view.problem);
    (view, media)
}

/// An object of the theme's engine by its number in the tree, its QObject's address.
fn object(id: u64) -> Pin<&'static mut QObject> {
    // Safety: the id was read from the live tree, and the screen it names lives until the host loads again.
    unsafe { Pin::new_unchecked(&mut *(id as *mut QObject)) }
}

/// Every element of the built screen by name, in canvas units.
fn boxes(tree: &Tree, unit: f64) -> BTreeMap<String, Boxed> {
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
fn rule(spec: &str, k: f64, got: &BTreeMap<String, Boxed>, canvas_width: f64) -> Option<(char, f64)> {
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
        ["below", n, g] => Some(('y', of(n).bottom() + num(g) * k)),
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
fn tokens(screen: u64) -> BTreeMap<String, String> {
    let tokens = host::kit_tokens(object(screen)).expect("the theme's tokens");
    Scene::constants(tokens.upcast_pin()).into_iter().collect()
}

/// A property of an element of the placed screen, as Qt writes it as text; the element by its number in the tree, since a
/// Repeater's items are no QObject children of the screen.
fn read(tree: &Tree, name: &str, property: &str) -> String {
    // A Line's words are named as the Line is: the last of a name in the tree, the deepest, is the words themselves.
    let Some(i) = tree.nodes.iter().rposition(|n| n.name == name) else { return String::new() };
    let id: u64 = tree.nodes[i as usize].id.parse().unwrap();
    emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(name), &QString::from(property)).to_string()
}

/// A QFont as Qt writes it: family, point size, pixel size, style hint, weight, style, underline, strike out, fixed pitch, 0,
/// capitalization, letter spacing's type, letter spacing, ...; the weight, the case (1, all capitals) and the absolute spacing.
fn font_of(text: &str) -> Option<(f64, f64, i32, i32, f64)> {
    let f: Vec<&str> = text.split(',').collect();
    let n = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
    Some((n(2)?, n(4)?, n(10)? as i32, n(11)? as i32, n(12)?))
}

/// What differs from a row's type: its words' weight, spacing and case, or its glyph's size and weight.
fn typed(tree: &Tree, row: &Row, k: f64, unit: f64, tokens: &BTreeMap<String, String>) -> Vec<String> {
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
        [kind @ ("glyph" | "word"), size, weight] => {
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
            // The shape drawn: the ellipse its box holds (a circle of the box's width, its radius half that, scaled to the box's
            // height), or the box at the board's corner in canvas units; the other hidden.
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
        _ => panic!("{}: no such type {:?}", row.name, row.kind),
    }
    out
}

/// The colour an element paints: a pill's, a mark's, a field's, a disc's fill, a help line's words, else its own colour.
fn painted(tree: &Tree, name: &str) -> Option<Colour> {
    ["paint", "ink", "fill", "labelColor", "color"].iter().find_map(|p| Colour::parse(&read(tree, name, p)))
}

/// Places Eclipse's Home on the synthetic library and holds it to the table; with `EMUSEN_ECLIPSE_PICTURES` naming a folder,
/// grabs it at the board's shape and the handheld's into that folder, each named by the size and `backend`.
pub fn oracle(backend: &str) {
    // The machine's locale and zone, as `QLocale::system()` and the local zone read them: the board's, en_US in UTC.
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let world = World::new("qt-app-eclipse-home");
    let mut harness = emusen_qt_test::Harness::new();
    let (view, media) = synthetic(&world, &mut harness);
    let file = world.root.join("Scene.qml");
    std::fs::write(&file, SCENE).unwrap();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    assert!(scene.set_property("host", "coverStores", &media.display().to_string()));
    let folder = std::fs::canonicalize(ECLIPSE).unwrap().display().to_string();
    assert!(scene.set_property("host", "folder", &folder));
    scene.wait(200);
    assert_eq!(scene.property("scene", "placed"), "true", "{}", scene.property("host", "warnings"));
    let tree = scene.tree();
    let screen: u64 = tree.nodes[tree.find_named("homeScreen").expect("Eclipse's Home is placed") as usize].id.parse().unwrap();

    // The views fed as the program's host feeds them: the library, the first game suspended twelve minutes ago, the clock.
    let now = NOW;
    let first = rom(&world, GAMES[0].0, GAMES[0].1);
    let game = view.game(&first).expect("the suspended game is in the library");
    // Three hours and twenty minutes played, as games.db would hold it (Main.dc.html:56).
    let session_played = 12_000;
    let session = SessionFacts {
        has_game: true,
        key: first.clone(),
        title: game.title.clone(),
        console_id: game.system.clone(),
        console: game.console.clone(),
        hue: game.hue,
        paused_since: PAUSED,
        played: session_played,
    };
    host::words(object(screen)).expect("Words").show_now(now as f64);
    host::library(object(screen)).expect("Library").show(&view, "[]");
    host::session(object(screen)).expect("Session").show(&session);
    let frame: u64 = tree.nodes[tree.find_named("pausedFrame").expect("the paused picture") as usize].id.parse().unwrap();
    // Safety: the scene's item lives as long as the scene, longer than the screen that draws it.
    host::session(object(screen)).expect("Session").show_frame(frame as *mut _);
    // One pad, of the family whose letters the board draws (A, X, Y; Main:62-68).
    let family = emusen_kit_qt::kinds::family_of(emusen_kit_core::glyph::PadFamily::Xbox).repr;
    host::device(object(screen)).expect("Device").show(&DeviceFacts { controllers: 1, family,
        clock: emusen_qt_models::clock::clock_time(now),
        short_time: emusen_qt_models::clock::short_time(now),
        ..DeviceFacts::default()
    });
    let package = emusen_theme_host::package::read(&std::fs::read_to_string(format!("{ECLIPSE}/theme.json")).unwrap()).unwrap();
    host::set_token_overrides(object(screen), &package.overrides());
    host::kit_tokens(object(screen)).expect("the kit's tokens").set_form(kinds::form_of(WalkForm::BigScreen));
    let tokens = tokens(screen);
    scene.wait(200);
    // Placed and fed, the screen has logged nothing, and comes to rest: no frame swapped for two seconds.
    scene.assert_quiet();
    scene.assert_still("Eclipse's Home at rest");
    // Under reduced motion too, nothing is redrawn for two seconds (EmuSen_Qt.md §3.4.8: a frame was once seen there).
    host::kit_tokens(object(screen)).expect("the kit's tokens").set_reduced_motion(true);
    scene.wait(200);
    scene.assert_still("Eclipse's Home at rest under reduced motion");
    host::kit_tokens(object(screen)).expect("the kit's tokens").set_reduced_motion(false);
    scene.wait(200);

    let (board_height, rows) = rows();
    // A board pixel in canvas units, by the canvas's owner of its height.
    let k = DESIGN_HEIGHT / board_height;
    let mut differences = Vec::new();
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "homeCanvas", "unit").parse().expect("the canvas's unit");
        let canvas_width = width as f64 / unit;
        let got = boxes(&tree, unit);
        let at = format!("{width}x{height}");
        let named = |n: &str| tree.nodes.iter().rev().find(|m| m.name == n && m.visible).or_else(|| tree.nodes.iter().rev().find(|m| m.name == n));

        // The board's state: Recent's first game, the suspended one, focused and shown.
        assert_eq!(read(&tree, "tile.0", "activeFocus"), "true", "{at}: the first tile holds the focus");
        for row in &rows {
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
            // A turned element is held by its own box about its middle, as the board places it before it turns; its own size is
            // in the canvas's units, and the tree gives the box it turns into.
            if read(&tree, &row.name, "rotation").parse::<f64>().is_ok_and(|r| r != 0.0) {
                let (w, h): (f64, f64) = (read(&tree, &row.name, "width").parse().unwrap(), read(&tree, &row.name, "height").parse().unwrap());
                b = Boxed { x: b.x + (b.w - w) / 2.0, y: b.y + (b.h - h) / 2.0, w, h };
            }
            let tile = row.name.starts_with("tile.");
            if !node.visible {
                // A tile past the window's edge is not drawn; one that would fit must be.
                let x = rule(&row.x, k, &got, canvas_width).map(|r| r.1).unwrap_or(b.x);
                if !(tile && x + b.w > canvas_width) {
                    differences.push(format!("{at}: {} is hidden", row.name));
                }
                continue;
            }
            if row.part != "-" && !node.ty.contains(&row.part) && !node.cls.contains(&row.part) {
                differences.push(format!("{at}: {} is drawn by {} ({}), not {}", row.name, node.ty, node.cls, row.part));
            }
            for (spec, axis) in [(&row.x, 'x'), (&row.y, 'y'), (&row.w, 'w'), (&row.h, 'h')] {
                if let Some(n) = spec.strip_prefix("base:") {
                    // The words' baseline in canvas units: the Text's own offset, in board pixels, through the board's measure.
                    let base = |name: &str| read(&tree, name, "baselineOffset").parse::<f64>().unwrap_or(f64::NAN) * k;
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
                let have: f64 = read(&tree, &row.name, "radius").parse().unwrap_or(f64::NAN);
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
                let have = painted(&tree, &row.name);
                if want.is_none() || have != want {
                    differences.push(format!("{at}: {} paints {have:?} where the board gives {} ({want:?})", row.name, row.colour));
                }
            }
            if let Some(blur) = row.size.strip_prefix("blur:") {
                // A field's blur, the Gaussian's deviation in canvas units (Main's px through the board's measure), as the field holds it.
                let want = blur.parse::<f64>().unwrap() * k;
                let have = read(&tree, &row.name, "deviation").parse::<f64>().unwrap_or(f64::NAN);
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
        for row in rows.iter().filter(|r| r.status == "built" && r.kind != "-") {
            for problem in typed(&tree, row, k, unit, &tokens) {
                differences.push(format!("{at}: {} {problem}", row.name));
            }
        }

        // The focused tile is ringed by the kit's FocusRing, and no other is.
        let rings: Vec<_> = tree.nodes.iter().filter(|n| n.name == "tileRing" && n.visible).collect();
        let tile0 = named("tile.0").map(|n| n.id.clone()).unwrap_or_default();
        if rings.len() != 1 || rings[0].parent != tile0 || !rings[0].ty.contains("FocusRing") {
            differences.push(format!("{at}: the focused tile's ring is {:?}", rings.iter().map(|r| (&r.parent, &r.ty)).collect::<Vec<_>>()));
        }

        // The words as drawn, the board's own literals (Main.dc.html:30, 49, 54-59, 62-68, 75, 79, 85-86, 94-97) where the
        // fixture's game is the board's: its first game, Aurora Drift on the Super Nintendo, suspended yesterday at 9:12 PM
        // after 3 h 20 min. "upper" words are drawn in capitals, which their type's case holds; their text is compared in them.
        // The count is the fixture's six games where the board has 24, and the other tiles are the fixture's own games.
        let mut words_of = vec![
            ("wordmark", "EmuSen", false),
            ("clock", "9:41 PM", false),
            ("eyebrow", "CONTINUE · SUPER NINTENDO", true),
            ("title", "Aurora Drift", false),
            ("facts", "Last played yesterday · 3 h 20 min played", false),
            ("resumeWords", "Resume point · saved 9:12 PM", false),
            ("buttonLabel.0", "Resume", false),
            ("buttonLabel.1", "Options", false),
            ("buttonLabel.2", "Favourite", false),
            ("artWords", "GAME ART · AURORA DRIFT", true),
            ("recentHeading", "Recent", false),
            ("recentCount", "1 of 6", false),
            ("tileTitle.0", "Aurora Drift", false),
            ("tileFacts.0", "SUPER NINTENDO", true),
        ];
        for (i, (title, _)) in GAMES.iter().enumerate().skip(1) {
            if named(&format!("tile.{i}")).is_some_and(|n| n.visible) {
                words_of.push((["", "tileTitle.1", "tileTitle.2", "tileTitle.3", "tileTitle.4", "tileTitle.5"][i], title, false));
            }
        }
        for (name, want, upper) in words_of {
            let have = read(&tree, name, "text");
            let drawn = if upper { have.to_uppercase() } else { have.clone() };
            if drawn != want {
                differences.push(format!("{at}: {name} says {drawn:?} where the board says {want:?}"));
            }
        }
        // The help line's words in order, the board's (Main:94-97), and its glyphs' letters and name.
        let help_id = named("help").map(|n| n.id.clone()).unwrap_or_default();
        let under_help = |n: &emusen_qt_test::tree::Node| {
            let mut at = n.parent.clone();
            while let Some(p) = tree.nodes.iter().find(|p| p.id == at) {
                if p.id == help_id {
                    return true;
                }
                at = p.parent.clone();
            }
            false
        };
        let line: Vec<&str> =
            tree.nodes.iter().filter(|n| n.visible && n.ty.contains("Text") && under_help(n)).filter_map(|n| n.text.as_deref()).filter(|t| !t.is_empty()).collect();
        if line != ["Resume", "Options", "Favourite", "Home card"] {
            differences.push(format!("{at}: the help line says {line:?} where the board says Resume, Options, Favourite, Home card"));
        }
        let glyphs: Vec<String> = ["Resume", "Options", "Favourite", "Home card"]
            .iter()
            .map(|w| {
                let g = tree.nodes.iter().rposition(|n| n.name == format!("ownGlyph-{w}") && n.visible);
                let child = g.and_then(|g| tree.nodes.iter().position(|n| n.parent == tree.nodes[g].id));
                child.map(|c| {
                    let id: u64 = tree.nodes[c].id.parse().unwrap();
                    let p = |q: &str| emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(tree.nodes[c].name.as_str()), &QString::from(q)).to_string();
                    if p("named") == "true" { p("word") } else { p("letter") }
                })
                .unwrap_or_default()
            })
            .collect();
        if glyphs != ["A", "X", "Y", "MENU"] {
            differences.push(format!("{at}: the help line's glyphs read {glyphs:?} where the board draws A, X, Y, MENU"));
        }
        // Nothing on the row is cut: a shown tile's title and console line are drawn whole.
        for n in tree.nodes.iter().filter(|n| n.visible && (n.name.starts_with("tileTitle.") || n.name.starts_with("tileFacts."))) {
            if read(&tree, &n.name, "truncated") == "true" {
                differences.push(format!("{at}: {} is cut: {:?}", n.name, read(&tree, &n.name, "text")));
            }
        }
    }
    // R-20261115-27, -28: the board's fields in the shown game's hues, over the first two covers. Each field's colour is its
    // board colour's saturation and lightness with the art hue the table gives it, the art's three spread by the board's
    // offsets where it has fewer (Home.qml `hues`). At the second field's centre, which no other field nor the disc reaches,
    // the window shows that colour at its opacity over the ground: on the card blurred (3.45 deviations inside the ellipse's
    // edge, so all but a trace of it), on the software backend unblurred (R-20261115-6). The cover's own colours are not there.
    {
        scene.resize(1920, 1200);
        scene.wait(500);
        let fields: Vec<(&Row, Colour, usize)> = rows
            .iter()
            .filter(|r| r.kind.starts_with("field "))
            .map(|r| {
                let p: Vec<&str> = r.kind.split(' ').collect();
                (r, Colour::parse(p[3]).expect("a field's board colour"), p[4].parse().expect("a field's hue"))
            })
            .collect();
        assert_eq!(fields.len(), 5, "the board's five fields");
        for (cover, (title, ext)) in GAMES.iter().take(2).enumerate() {
            if cover > 0 {
                scene.key(RIGHT, 0);
                scene.wait(500);
            }
            let tree = scene.tree();
            let game = view.game(&rom(&world, title, ext)).expect("a game of the library");
            let hero = if game.hero.is_empty() { game.cover } else { game.hero };
            let art = emusen_qt_models::art_hues::hues_of_file(&hero, 3);
            assert!(!art.is_empty(), "cover {cover} has hues: {hero}");
            // The board's three hues, 168, 251 and 218 degrees: the second 83 past the first, the third 50 past it.
            let spread = |from: f64, to: f64| from + (((to - from) % 360.0 + 540.0) % 360.0 - 180.0) * 50.0 / 83.0;
            let hues = match art.len() {
                1 => vec![art[0], art[0] + 83.0, art[0] + 50.0],
                2 => vec![art[0], art[1], spread(art[0], art[1])],
                _ => art.clone(),
            };
            for (row, board, which) in &fields {
                let have = Colour::parse(&read(&tree, &row.name, "paint")).expect("a field's paint").to_hsl();
                let (want, board) = (hues[*which], board.to_hsl());
                // Two degrees and a hundredth of saturation and lightness: the colour read back as 8 bits a channel.
                if emusen_kit_core::world::wrap_hue(have.0 - want).abs() > 2.0 || (have.1 - board.1).abs() > 0.01 || (have.2 - board.2).abs() > 0.01 {
                    differences.push(format!("{backend}: cover {cover}: {} painted {have:?} where the board's colour with hue {want:.1} gives {:?}", row.name, (want, board.1, board.2)));
                }
            }
            let board = fields[1].1.to_hsl();
            let field = Colour::hsl(hues[1], board.1, board.2, 1.0);
            let opacity: f64 = read(&tree, "backdropField.1", "opacity").parse().expect("the field's opacity");
            let ground = Colour::parse(&read(&tree, "backdrop", "ground")).expect("the backdrop's ground");
            let mix = |a: u8, b: u8| (f64::from(a) * opacity + f64::from(b) * (1.0 - opacity)).round() as u8;
            let want = Colour::rgb(mix(field.r, ground.r), mix(field.g, ground.g), mix(field.b, ground.b));
            let centre = (1380.0 + 380.0, 420.0 + 380.0);
            let seen = scene.colours_at(&[centre])[0];
            // Two levels of 255: an 8-bit blend's rounding, and on the card the blur's trace from the edge.
            let near = |a: Colour, b: Colour| a.r.abs_diff(b.r) <= 2 && a.g.abs_diff(b.g) <= 2 && a.b.abs_diff(b.b) <= 2;
            if !seen.is_some_and(|c| near(c, want)) {
                differences.push(format!("{backend}: cover {cover}: the second field's centre is {seen:?} where the art's hue {:.1} in the board's field gives {want:?}", hues[1]));
            }
            for band in emusen_qt_test::covers::art(cover) {
                if seen.is_some_and(|c| near(c, Colour::rgb(band.0, band.1, band.2))) {
                    differences.push(format!("{backend}: cover {cover}: the second field's centre shows the cover's own colour {band:?}"));
                }
            }
            // The disc's rim takes the first hue at the look's rim's saturation and lightness.
            let rim = Colour::parse(&read(&tree, "disc", "rimColor")).expect("the rim's colour").to_hsl();
            // Two degrees: the rim's colour read as 8 bits a channel, whose chroma near white leaves a step of about two degrees.
            if emusen_kit_core::world::wrap_hue(rim.0 - hues[0]).abs() > 2.0 {
                differences.push(format!("{backend}: cover {cover}: the rim's hue {:.1} where the art's first is {:.1}", rim.0, hues[0]));
            }
        }
        scene.key(LEFT, 0);
        scene.wait(500);
    }
    if let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") {
        std::fs::create_dir_all(&folder).unwrap();
        for (width, height) in [(1280, 800), (1920, 1200)] {
            scene.resize(width, height);
            scene.wait(500);
            let path = std::path::Path::new(&folder).join(format!("home-{width}x{height}-{backend}.png"));
            assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
        }
        // Each game shown in turn: the first four covers' colours in the board's fields, then the sixth game's, which has no
        // art and so takes its console's hue.
        scene.resize(1920, 1200);
        for i in 1..GAMES.len() {
            scene.key(RIGHT, 0);
            scene.wait(500);
            let name = if i + 1 == GAMES.len() { format!("home-1920x1200-no-art-{backend}.png") } else { format!("home-1920x1200-game{i}-{backend}.png") };
            let path = std::path::Path::new(&folder).join(name);
            assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
        }
        for _ in 1..GAMES.len() {
            scene.key(LEFT, 0);
            scene.wait(50);
        }
    }

    // The pad reaches everything drawn: Up from the shown tile to the buttons, Up to the search, and Down back again.
    scene.resize(1280, 800);
    scene.wait(200);
    let focused = |scene: &mut Scene<'_>| {
        let tree = scene.tree();
        scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
    };
    let mut walked = vec![focused(&mut scene)];
    for key in [UP, RIGHT, UP, DOWN, DOWN] {
        scene.key(key, 0);
        scene.wait(50);
        walked.push(focused(&mut scene));
    }
    if walked != ["tile.0", "button.0", "button.1", "search", "button.0", "tile.0"] {
        differences.push(format!("the pad walks {walked:?}"));
    }
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    scene.assert_quiet();
    // The host drops the screen and its engine and places it again, as theme-check's audit places each screen: nothing is logged.
    assert!(scene.set_property("host", "folder", ""));
    scene.wait(300);
    assert!(scene.set_property("host", "folder", &folder));
    scene.wait(300);
    assert!(scene.set_property("host", "folder", ""));
    scene.wait(300);
    scene.assert_quiet();
}
