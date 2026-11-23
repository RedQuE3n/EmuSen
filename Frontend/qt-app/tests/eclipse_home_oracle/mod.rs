//! The oracle of Eclipse's Home, run by eclipse_home.rs on the software backend and by eclipse_home_gpu.rs on the card.

use crate::eclipse_board::{held, object, read, rows, Row};
use emusen_qt_test::Scene;
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
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::host;
use emusen_theme_qt::session::SessionFacts;
use serde_json::json;

const TABLE: &str = include_str!("../eclipse_home.tsv");
const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");

/// The board's six games in Recent's order, each with an extension only its console lists; the first is the one suspended.
pub const GAMES: [(&str, &str); 6] = [
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

pub fn rom(world: &World, title: &str, ext: &str) -> String {
    world.dir("Roms").join(format!("{title}.{ext}")).to_string_lossy().into_owned()
}

/// The covers' size in pixels, three by four, as theme-audit's library draws them.
const COVER: (u32, u32) = (450, 600);

/// The six games written, played an hour apart with the first the latest, each but the last with a cover in an ES-DE media
/// folder, and read through the real clients; the media folder, which the host lets a theme show.
pub fn synthetic(world: &World, harness: &mut emusen_qt_test::Harness) -> (LibraryView, std::path::PathBuf) {
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
    // Home shown, the section the program's screens name (qt-models/src/sections.rs).
    host::library(object(screen)).expect("Library").show_section(emusen_qt_models::sections::all()[0].key);
    host::session(object(screen)).expect("Session").show(&session);
    let frame: u64 = tree.nodes[tree.find_named("pausedFrame").expect("the paused picture") as usize].id.parse().unwrap();
    // Safety: the scene's item lives as long as the scene, longer than the screen that draws it.
    host::session(object(screen)).expect("Session").show_frame(frame as *mut _);
    // One pad, of the family whose letters the board draws (A, X, Y; Main:62-68).
    let family = emusen_kit_qt::kinds::family_of(emusen_kit_core::glyph::PadFamily::Xbox).repr;
    // The machine's battery at the board's 82%, discharging (Main.dc.html:45-47).
    host::device(object(screen)).expect("Device").show(&DeviceFacts { controllers: 1, family, battery: 82, charging: false,
        clock: emusen_qt_models::clock::clock_time(now),
        short_time: emusen_qt_models::clock::short_time(now),
        ..DeviceFacts::default()
    });
    let package = emusen_theme_host::package::read(&std::fs::read_to_string(format!("{ECLIPSE}/theme.json")).unwrap()).unwrap();
    host::set_token_overrides(object(screen), &package.overrides());
    host::kit_tokens(object(screen)).expect("the kit's tokens").set_form(kinds::form_of(WalkForm::BigScreen));
    let tokens = crate::eclipse_board::tokens(screen);
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

    let (board_height, rows) = rows(TABLE);
    // A board pixel in canvas units, by the canvas's owner of its height.
    let k = DESIGN_HEIGHT / board_height;
    let mut differences = Vec::new();
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "homeCanvas", "unit").parse().expect("the canvas's unit");
        let canvas_width = width as f64 / unit;
        let at = format!("{width}x{height}");
        let named = |n: &str| tree.nodes.iter().rev().find(|m| m.name == n && m.visible).or_else(|| tree.nodes.iter().rev().find(|m| m.name == n));

        // The board's state: Recent's first game, the suspended one, focused and shown.
        assert_eq!(read(&tree, "tile.0", "activeFocus"), "true", "{at}: the first tile holds the focus");
        // Every built row at this size; a tile past the window's edge may be hidden.
        held(&tree, &rows, k, unit, canvas_width, &tokens, &at, &|name, x, w| name.starts_with("tile.") && x + w > canvas_width, &mut differences);

        // The focused tile is ringed by the kit's FocusRing, and no other is.
        let rings: Vec<_> = tree.nodes.iter().filter(|n| n.name == "tileRing" && n.visible).collect();
        let tile0 = named("tile.0").map(|n| n.id.clone()).unwrap_or_default();
        if rings.len() != 1 || rings[0].parent != tile0 || !rings[0].ty.contains("FocusRing") {
            differences.push(format!("{at}: the focused tile's ring is {:?}", rings.iter().map(|r| (&r.parent, &r.ty)).collect::<Vec<_>>()));
        }

        // The words as drawn, the board's own literals (Main.dc.html:30, 47, 49, 54-59, 62-68, 75, 79, 85-86, 94-97) where the
        // fixture's game is the board's: its first game, Aurora Drift on the Super Nintendo, suspended yesterday at 9:12 PM
        // after 3 h 20 min. "upper" words are drawn in capitals, which their type's case holds; their text is compared in them.
        // The count is the fixture's six games where the board has 24, and the other tiles are the fixture's own games.
        let mut words_of = vec![
            ("wordmark", "EmuSen", false),
            ("clock", "9:41 PM", false),
            ("batteryLevel", "82%", false),
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
            ("navWords.home", "Home", false),
            ("navWords.library", "Library", false),
            ("navWords.consoles", "Consoles", false),
            ("navWords.collections", "Collections", false),
            ("navWords.tools", "Tools", false),
            ("helpSectionsWords", "Sections", false),
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
        // The glass's shoulders, named in their boxes (Main:100-101).
        let shoulders: Vec<String> = ["ownGlyph-Sections", "ownGlyph2-Sections"]
            .iter()
            .map(|g| {
                let g = tree.nodes.iter().rposition(|n| n.name == *g && n.visible);
                let child = g.and_then(|g| tree.nodes.iter().position(|n| n.parent == tree.nodes[g].id));
                child.map(|c| {
                    let id: u64 = tree.nodes[c].id.parse().unwrap();
                    emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(tree.nodes[c].name.as_str()), &QString::from("word")).to_string()
                })
                .unwrap_or_default()
            })
            .collect();
        if shoulders != ["LB", "RB"] {
            differences.push(format!("{at}: the Sections glass's glyphs read {shoulders:?} where the board draws LB, RB"));
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
    // A cover written again at its path, as a scrape replaces one, is read again once the library is fed: the shown game's
    // second field takes the new picture's hue, not the hue read before.
    {
        let cover = |i: usize| {
            let game = view.game(&rom(&world, GAMES[i].0, GAMES[i].1)).expect("a game of the library");
            if game.hero.is_empty() { game.cover } else { game.hero }
        };
        let (shown, other) = (cover(0), cover(2));
        let was = emusen_qt_models::art_hues::hues_of_file(&shown, 3);
        std::fs::copy(&other, &shown).expect("the cover written again");
        let now = emusen_qt_models::art_hues::hues_of_file(&shown, 3);
        assert!(now.len() > 1 && was.len() > 1 && emusen_kit_core::world::wrap_hue(now[1] - was[1]).abs() > 10.0, "the new picture's hues differ: {was:?} {now:?}");
        host::library(object(screen)).expect("Library").show(&view, "[]");
        let mut seen = f64::NAN;
        for _ in 0..40 {
            scene.wait(50);
            let tree = scene.tree();
            seen = Colour::parse(&read(&tree, "backdropField.1", "paint")).expect("a field's paint").to_hsl().0;
            if emusen_kit_core::world::wrap_hue(seen - now[1]).abs() <= 2.0 {
                break;
            }
        }
        if emusen_kit_core::world::wrap_hue(seen - now[1]).abs() > 2.0 {
            differences.push(format!("{backend}: the cover written again keeps the hue {seen:.1} where its new picture gives {:.1}", now[1]));
        }
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
