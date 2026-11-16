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
use emusen_qt_models::help::{entries, Screen};
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

/// The clock the device is fed, in the program's "hh:mm".
const CLOCK: &str = "21:41";

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
    drawn: String,
    raised: String,
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
            assert_eq!(c.len(), 16, "a row of 16 columns: {l}");
            let s = |i: usize| c[i].to_owned();
            Row { name: s(0), part: s(4), x: s(5), y: s(6), w: s(7), h: s(8), radius: s(9), colour: s(10), size: s(11), status: s(12), drawn: s(14), raised: s(15) }
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
    tree.nodes
        .iter()
        .filter(|n| !n.name.is_empty())
        .map(|n| {
            // A turned item is held by its own box about its middle, as the board places it before it turns.
            let b = Boxed { x: n.x / unit, y: n.y / unit, w: n.w / unit, h: n.h / unit };
            (n.name.clone(), b)
        })
        .collect()
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
    let Some(i) = tree.find_named(name) else { return String::new() };
    let id: u64 = tree.nodes[i as usize].id.parse().unwrap();
    emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(name), &QString::from(property)).to_string()
}

/// The colour an element paints: a pill's, a mark's, a field's, a disc's fill, a help line's words, else its own colour.
fn painted(tree: &Tree, name: &str) -> Option<Colour> {
    ["paint", "ink", "fill", "labelColor", "color"].iter().find_map(|p| Colour::parse(&read(tree, name, p)))
}

/// Places Eclipse's Home on the synthetic library and holds it to the table; with `EMUSEN_ECLIPSE_PICTURES` naming a folder,
/// grabs it at the board's shape and the handheld's into that folder, each named by the size and `backend`.
pub fn oracle(backend: &str) {
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
    let now = words::now();
    let first = rom(&world, GAMES[0].0, GAMES[0].1);
    let game = view.game(&first).expect("the suspended game is in the library");
    // An hour and a half played, as games.db would hold it.
    let session_played = 5400;
    let session = SessionFacts {
        has_game: true,
        key: first.clone(),
        title: game.title.clone(),
        console_id: game.system.clone(),
        console: game.console.clone(),
        hue: game.hue,
        paused_since: now - 720,
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
    host::device(object(screen)).expect("Device").show(&DeviceFacts { controllers: 1, family, clock: CLOCK.into(), ..DeviceFacts::default() });
    let package = emusen_theme_host::package::read(&std::fs::read_to_string(format!("{ECLIPSE}/theme.json")).unwrap()).unwrap();
    host::set_token_overrides(object(screen), &package.overrides());
    host::kit_tokens(object(screen)).expect("the kit's tokens").set_form(kinds::form_of(WalkForm::BigScreen));
    let tokens = tokens(screen);
    let text_scale: f64 = *host::kit_tokens(object(screen)).expect("the kit's tokens").text_scale();
    scene.wait(200);
    // Placed and fed, the screen has logged nothing, and comes to rest: no frame swapped for two seconds.
    scene.assert_quiet();
    scene.assert_still("Eclipse's Home at rest");

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
        let named = |n: &str| tree.find_named(n).map(|i| &tree.nodes[i as usize]);

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
                // The backdrop's blur for its picture, in canvas units (Main:19's px through the board's measure).
                let want = blur.parse::<f64>().unwrap() * k;
                let have: f64 = read(&tree, "backdrop", "sourceBlur").parse().unwrap_or(f64::NAN);
                if !same(have, want) {
                    differences.push(format!("{at}: {} blur {have} where the table gives {want}", row.name));
                }
            }
            if let Ok(size) = row.size.parse::<f64>() {
                // The words' size in the element's own units, the canvas's, as the harness reads the font; a help line's are its entries'.
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
                    let sizes: Vec<f64> = tree.nodes.iter().filter(|n| n.text.as_deref().is_some_and(|t| !t.is_empty()) && inside(n)).map(|n| n.size).collect();
                    if sizes.iter().all(|s| *s == sizes[0]) { sizes.first().copied().unwrap_or(f64::NAN) } else { f64::NAN }
                } else {
                    node.size
                };
                // The kit's rule for a look's size (EmuSen_Kit.md §7.121), then the whole pixel Qt draws a real size at.
                let units = size * k;
                // In the face and weight the element draws, as Qt writes its font: "family,point size,pixel size,hint,weight,...".
                let font = if row.part == "HelpBar" { read(&tree, &row.name, "labelFont") } else { read(&tree, &row.name, "font") };
                let font: Vec<&str> = font.split(',').collect();
                let (family, weight) = (font.first().copied().unwrap_or_default(), font.get(4).and_then(|w| w.parse::<i32>().ok()).expect("the font's weight"));
                let legible = emusen_kit_qt::type_size::legible_in(units, text_scale, &family, weight);
                let drawn = legible.round();
                // The table says the same, so the tester reads each size where the element is listed.
                let raised = legible > emusen_kit_core::tokens::scaled_text(units, text_scale);
                if row.drawn.parse::<f64>().ok() != Some(drawn) || (row.raised == "raised") != raised {
                    differences.push(format!("{at}: {} the table gives {} {}, the rule {drawn} {raised}", row.name, row.drawn, row.raised));
                }
                if at.as_str() == "1280x800" {
                    println!("type\t{}\t{family} {weight}\t{size}\t{units:.3}\t{legible}\t{drawn}\t{}", row.name, if legible > emusen_kit_core::tokens::scaled_text(units, text_scale) { "raised" } else { "-" });
                }
                if have != drawn {
                    differences.push(format!("{at}: {} type {have} where the kit's rule gives {drawn} ({units:.3} on the board)", row.name));
                }
            }
        }

        // The focused tile is ringed by the kit's FocusRing, and no other is.
        let rings: Vec<_> = tree.nodes.iter().filter(|n| n.name == "tileRing" && n.visible).collect();
        let tile0 = named("tile.0").map(|n| n.id.clone()).unwrap_or_default();
        if rings.len() != 1 || rings[0].parent != tile0 || !rings[0].ty.contains("FocusRing") {
            differences.push(format!("{at}: the focused tile's ring is {:?}", rings.iter().map(|r| (&r.parent, &r.ty)).collect::<Vec<_>>()));
        }

        // Every word is the program's.
        let help = entries(Screen::Home);
        let mut words_of = vec![
            ("wordmark", words::NAME.to_owned()),
            ("eyebrow", words::continue_line(&game.console)),
            ("title", game.title.clone()),
            ("facts", words::played_facts(now - 720, now, session_played)),
            ("resumeWords", words::resume_point(&emusen_qt_models::clock::clock_time(now - 720))),
            ("clock", CLOCK.to_owned()),
            ("recentHeading", words::RECENT_ENTRY.to_owned()),
            ("tileFacts.0", game.console.clone()),
        ];
        for (i, entry) in help.iter().take(3).enumerate() {
            words_of.push((["buttonLabel.0", "buttonLabel.1", "buttonLabel.2"][i], entry.1.to_owned()));
        }
        for (i, (title, ext)) in GAMES.iter().enumerate() {
            if named(&format!("tile.{i}")).is_some_and(|n| n.visible) {
                let row = view.game(&rom(&world, title, ext)).expect("a game of the library");
                words_of.push((["tileTitle.0", "tileTitle.1", "tileTitle.2", "tileTitle.3", "tileTitle.4", "tileTitle.5"][i], row.title));
            }
        }
        for (name, want) in words_of {
            let have = read(&tree, name, "text");
            if have != want {
                differences.push(format!("{at}: {name} says {have:?} where the program says {want:?}"));
            }
        }
        // Nothing on the row is cut: a shown tile's title and console line are drawn whole.
        for n in tree.nodes.iter().filter(|n| n.visible && (n.name.starts_with("tileTitle.") || n.name.starts_with("tileFacts."))) {
            if read(&tree, &n.name, "truncated") == "true" {
                differences.push(format!("{at}: {} is cut: {:?}", n.name, read(&tree, &n.name, "text")));
            }
        }
    }
    if backend == "software" {
        // R-20261115-6: on the software backend the shown game's art is its own colours, unblurred, dimmed to the backdrop's
        // opacity over the ground. The first cover is three bands of qt-test's `covers::art`, a third of its height each,
        // cropped to fill the window; read at the window's right edge in the middle of each band and two cover pixels either side
        // of the bands' edge, where an unblurred picture has changed and a blur of the board's reach (90 px) would still mix.
        // Image's smooth scaling was seen on 2026-11-15 still mixing 4.4 px from the edge (green 168 for 171); its span is not known.
        scene.resize(1280, 800);
        scene.wait(300);
        let tree = scene.tree();
        let opacity: f64 = read(&tree, "backdrop", "sourceOpacity").parse().expect("the backdrop's opacity");
        let ground = Colour::parse(&read(&tree, "backdrop", "ground")).expect("the backdrop's ground");
        let over = |c: (u8, u8, u8)| {
            let mix = |a: u8, b: u8| (f64::from(a) * opacity + f64::from(b) * (1.0 - opacity)).round() as u8;
            Colour::rgb(mix(c.0, ground.r), mix(c.1, ground.g), mix(c.2, ground.b))
        };
        // Image.PreserveAspectCrop: the cover scaled to cover the window, its middle shown.
        let (cw, ch) = (f64::from(COVER.0), f64::from(COVER.1));
        let scale = (1280.0 / cw).max(800.0 / ch);
        let top = (ch * scale - 800.0) / 2.0;
        let edge = |k: f64| k * ch / 3.0 * scale - top;
        let x = 1279.0 - 0.5;
        let past = 2.0 * scale;
        let art = emusen_qt_test::covers::art(0);
        let points = [
            ("the top band", (x, edge(1.0) / 2.0), art[0]),
            ("the middle band", (x, (edge(1.0) + edge(2.0)) / 2.0), art[1]),
            ("the bottom band", (x, (edge(2.0) + 800.0) / 2.0), art[2]),
            ("above the first edge", (x, edge(1.0) - past), art[0]),
            ("below the first edge", (x, edge(1.0) + past), art[1]),
        ];
        let seen = scene.colours_at(&points.iter().map(|p| p.1).collect::<Vec<_>>());
        // One level of 255 is an 8-bit blend's rounding.
        let near = |a: Option<Colour>, b: Colour| a.is_some_and(|a| a.r.abs_diff(b.r) <= 1 && a.g.abs_diff(b.g) <= 1 && a.b.abs_diff(b.b) <= 1);
        for ((what, at, colour), got) in points.iter().zip(seen) {
            if !near(got, over(*colour)) {
                differences.push(format!("software backdrop: {what} at {at:?} is {got:?} where the art dimmed to {opacity} over the ground gives {:?}", over(*colour)));
            }
        }
    }
    if let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") {
        std::fs::create_dir_all(&folder).unwrap();
        for (width, height) in [(1280, 800), (1920, 1200)] {
            scene.resize(width, height);
            scene.wait(500);
            let path = std::path::Path::new(&folder).join(format!("home-{width}x{height}-{backend}.png"));
            assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
        }
        // Once with the shown game one with no art: the ground alone behind the disc.
        scene.resize(1920, 1200);
        for _ in 1..GAMES.len() {
            scene.key(RIGHT, 0);
            scene.wait(50);
        }
        scene.wait(500);
        let path = std::path::Path::new(&folder).join(format!("home-1920x1200-no-art-{backend}.png"));
        assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
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
