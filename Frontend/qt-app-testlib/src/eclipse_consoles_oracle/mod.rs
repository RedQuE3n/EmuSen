//! Eclipse's Choose a console in the running program, driven by an SDL pad in this process (EmuSen_Qt.md §3.4.9, Proof).


use crate::common::{approved_console_core, wait_for, CONSOLES};
use crate::eclipse_pad::{focused, press};
use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::group_grid::group_grid;
use emusen_kit_core::tokens::Colour;
use emusen_kit_core::input::PadControl;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{refresh, LibrarySource, LibraryView};
use emusen_qt_models::{themes, words};
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::tree::{colour, Tree};
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use emusen_qt_models::console_list::{Order, Show};
use emusen_theme_qt::view::{consoles_listed, Cell, CONSOLE_ROLES};
use serde_json::json;

/// The board's elements, one a row (eclipse/consoles.tsv), held by eclipse_board's rules.
const BOARD: &str = include_str!("../../../qt-app/tests/eclipse/consoles.tsv");

/// Every row of the board's table held at each gate size with `c` focused, and both fields in `c`'s hue (R-20261115-28).
fn board_held(scene: &mut Scene<'_>, c: &Console) {
    let (board_height, rows) = crate::eclipse_board::rows(BOARD);
    // The rows of a chip that holds none are held in that state alone (`empty_held`).
    let rows: Vec<_> = rows.into_iter().filter(|r| r.status != "empty").collect();
    let k = DESIGN_HEIGHT / board_height;
    let hue = Colour::parse(&c.glow).expect("the console's glow").to_hsl().0;
    let mut differences = Vec::new();
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let at = format!("{width}x{height}");
        let unit: f64 = crate::eclipse_board::read(&tree, "consolesCanvas", "unit").parse().expect("the canvas's unit");
        let screen: u64 = tree.nodes[tree.find_named("consolesScreen").expect("Eclipse's Choose a console") as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        crate::eclipse_board::held(&tree, &rows, k, unit, f64::from(width) / unit, &tokens, &at, &|_, _, _| false, &mut differences);
        // The nav's words, the board's (EclipseConsoles:29-33).
        for (key, want) in [("home", "Home"), ("library", "Library"), ("consoles", "Consoles"), ("collections", "Collections"), ("tools", "Tools")] {
            let have = crate::eclipse_board::read(&tree, &format!("navWords.{key}"), "text");
            if have != want {
                differences.push(format!("{at}: navWords.{key} says {have:?} where the board says {want:?}"));
            }
        }
        // The disc's glows blurred at the scene's pixels.
        let scale = crate::eclipse_board::read(&tree, "consoleDisc", "pixelScale").parse::<f64>().unwrap_or(f64::NAN);
        if (scale - unit).abs() > 1e-9 {
            differences.push(format!("{at}: the disc's glows blurred at {scale} pixels a unit where the scene draws {unit}"));
        }
        for row in rows.iter().filter(|r| r.kind.starts_with("field ")) {
            let board = Colour::parse(row.kind.split(' ').nth(3).unwrap()).expect("a field's board colour").to_hsl();
            let have = Colour::parse(&crate::eclipse_board::read(&tree, &row.name, "paint")).map(|p| p.to_hsl());
            // Two degrees and a hundredth of saturation and lightness: the colour read back as 8 bits a channel.
            if !have.is_some_and(|h| emusen_kit_core::world::wrap_hue(h.0 - hue).abs() <= 2.0 && (h.1 - board.1).abs() <= 0.01 && (h.2 - board.2).abs() <= 0.01) {
                differences.push(format!("{at}: {} painted {have:?} where the board's colour with {}'s hue {hue:.1} gives {board:?}", row.name, c.display_name));
            }
        }
    }
    // The paths of the played share (rows plateTrack.0, plateArc.0): stroke, colour, the arc's start and turn, the cap.
    let tree = scene.tree();
    let tokens = crate::eclipse_board::tokens(tree.nodes[tree.find_named("consolesScreen").unwrap() as usize].id.parse().unwrap());
    let share_id: u64 = tree.nodes.iter().rev().find(|n| n.name == "plateShare.0").expect("plate.0's share").id.parse().unwrap();
    let of = |name: &str, p: &str| emusen_qt_test::ffi::ffi::harness_property(crate::eclipse_board::object(share_id), &cxx_qt_lib::QString::from(name), &cxx_qt_lib::QString::from(p)).to_string();
    let share = if c.count > 0 { (c.played as f64 / c.count as f64).max(0.04) } else { 0.0 };
    for row in rows.iter().filter(|r| r.part == "ShapePath") {
        let t: Vec<&str> = row.kind.split(' ').collect();
        let ["path", width, start, turn, cap] = t.as_slice() else { panic!("{}: a path's type", row.name) };
        let arc = row.name.replace("plateTrack", "plateRound").replace("plateArc", "plateSweep");
        let num = |v: String| v.parse::<f64>().unwrap_or(f64::NAN);
        let want_turn = if *turn == "share" { 360.0 * share } else { turn.parse().unwrap() };
        let want_colour = match row.colour.strip_prefix("Tokens.") {
            Some(t) => tokens.get(t).and_then(|v| Colour::parse(v)),
            None => Colour::parse(&row.colour),
        };
        // SVG's round and its default butt are Qt's RoundCap and FlatCap, as Qt names them.
        let want_cap = if *cap == "round" { "RoundCap" } else { "FlatCap" };
        let checks = [
            (num(of(&row.name, "strokeWidth")) - width.parse::<f64>().unwrap() * k).abs() < 1e-6,
            Colour::parse(&of(&row.name, "strokeColor")) == want_colour,
            (num(of(&arc, "startAngle")) - start.parse::<f64>().unwrap()).abs() < 1e-6,
            (num(of(&arc, "sweepAngle")) - want_turn).abs() < 1e-6,
            of(&row.name, "capStyle") == want_cap,
        ];
        if checks.iter().any(|ok| !ok) {
            differences.push(format!("{}: {:?} (width, colour, start, turn, cap) where the board gives {}", row.name, checks, row.kind));
        }
    }
    // CSS's line height of each size; the NES's plate gives way to its short name, its name by the disc takes three lines (R-20261116-5).
    for (name, size, height, lines) in [("plateName.0", 21.0, 1.15, 1), ("consoleName", 46.0, 1.08, 3)] {
        let mode = crate::eclipse_board::read(&tree, name, "lineHeightMode");
        let have = crate::eclipse_board::read(&tree, name, "lineHeight").parse::<f64>().unwrap_or(f64::NAN);
        if mode != "FixedHeight" || (have - size * height).abs() > 1e-6 {
            differences.push(format!("{name}: line height {have} (mode {mode}) where the board gives {} fixed", size * height));
        }
        let count = crate::eclipse_board::read(&tree, name, "lineCount");
        // The fixture's first console is the Nintendo Entertainment System.
        assert_eq!(c.display_name, "Nintendo Entertainment System", "the fixture's long name on plate.0");
        if count != lines.to_string() || crate::eclipse_board::read(&tree, name, "truncated") != "false" {
            differences.push(format!("{name}: {count} lines (truncated {}) for {:?} where it wraps to {lines} whole", crate::eclipse_board::read(&tree, name, "truncated"), crate::eclipse_board::read(&tree, name, "text")));
        }
    }
    // Drawn, at the scene's pixels: just past the rim, where the glow is brightest, the window shows the console's hue.
    scene.resize(1920, 1200);
    scene.wait(500);
    let tree = scene.tree();
    let unit: f64 = crate::eclipse_board::read(&tree, "consolesCanvas", "unit").parse().expect("the canvas's unit");
    let disc = tree.nodes.iter().rev().find(|n| n.name == "consoleDisc" && n.visible).expect("the disc");
    let at = (disc.x + disc.w + 12.0 * k * unit, disc.y + disc.h / 2.0);
    let seen = scene.colours_at(&[at])[0].map(|c| c.to_hsl());
    // Six degrees: the glow over the fields, each the same hue, read back as 8 bits a channel.
    if !seen.is_some_and(|h| emusen_kit_core::world::wrap_hue(h.0 - hue).abs() <= 6.0 && h.2 > 0.1) {
        differences.push(format!("the glow past the rim at {at:?} is {seen:?} where {}'s hue is {hue:.1}", c.display_name));
    }
    assert!(differences.is_empty(), "{} differences from the board:\n{}", differences.len(), differences.join("\n"));
}

/// One console of `Library.consoles` as the model gives it.
#[derive(Clone, Debug)]
struct Console {
    key: String,
    name: String,
    display_name: String,
    maker: String,
    year: i32,
    count: usize,
    played: usize,
    glow: String,
    engine_name: String,
    engine_kind: String,
    screen: String,
    group: usize,
    group_index: usize,
}

fn cell<'a>(row: &'a [Cell], role: &str) -> &'a Cell {
    &row[CONSOLE_ROLES.iter().position(|r| *r == role).expect("a console role")]
}

fn text(row: &[Cell], role: &str) -> String {
    match cell(row, role) {
        Cell::Text(t) => t.clone(),
        other => panic!("{role} is no text: {other:?}"),
    }
}

fn whole(row: &[Cell], role: &str) -> i32 {
    match cell(row, role) {
        Cell::Whole(n) => *n,
        other => panic!("{role} is no whole number: {other:?}"),
    }
}

/// The eleven consoles' games in their ES-DE folders, some played (EmuSen_Qt.md §3.4.9, the tests' means).
fn written(world: &World) -> Vec<String> {
    let source = ClientSource::open(&library().to_string_lossy(), &Config::default()).unwrap();
    let systems = source.systems().unwrap();
    let mut played = Vec::new();
    for (n, id) in CONSOLES.iter().enumerate() {
        let system = systems.iter().find(|s| s.id == *id).expect("a canvas console in the table");
        let ext = system.extensions.first().unwrap_or_else(|| panic!("{id} takes an extension"));
        // The Nintendo 64, which a core runs, holds no game, so the All chip holds one With games does not.
        if *id == "n64" {
            continue;
        }
        let mut image = vec![0u8; 4096];
        image[0x143] = if *id == "gbc" { 0x80 } else { 0 };
        for g in 0..=(n % 3) {
            let path = world.dir(&format!("Roms/{}", system.esde_system)).join(format!("{id} game {g}{ext}"));
            std::fs::write(&path, &image).unwrap();
            if g == 0 || (n == 0 && g == 1) {
                played.push(path.to_string_lossy().into_owned());
            }
        }
    }
    played
}

/// The library written, the fixture's PlayStation core approved, its plays recorded; the consoles `Library.consoles` gives.
fn library_of(world: &World, cores: &std::path::Path) -> (Vec<Console>, LibraryView) {
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.root.join("Roms").to_string_lossy());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let played = written(world);
    approved_console_core(cores);
    let source = emusen_qt_models::clients::source_for(&library().to_string_lossy(), cores.to_str().unwrap()).unwrap();
    source.games().unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let records = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    for (i, path) in played.iter().enumerate() {
        records.change(&json!({ "op": "started", "path": path, "at": words::utc(words::now() - 3600 * (i as i64 + 1)) })).unwrap();
    }
    // One favourite, so the Favourites chip holds its console alone.
    records.change(&json!({ "op": "toggle_favourite", "path": played[0] })).unwrap();
    let mut view = LibraryView::empty();
    view.apply(refresh(&source));
    assert!(view.problem.is_empty(), "{}", view.problem);
    // Eclipse shows the All chip in maker order first: every console a core runs, each maker's together.
    let model = consoles_listed(&view, Show::All, Order::Maker, &[])
        .iter()
        .map(|r| Console {
            key: text(r, "key"),
            name: text(r, "name"),
            display_name: text(r, "displayName"),
            maker: text(r, "maker"),
            year: whole(r, "year"),
            count: whole(r, "count") as usize,
            played: whole(r, "played") as usize,
            glow: text(r, "glow"),
            engine_name: text(r, "engineName"),
            engine_kind: text(r, "engineKind"),
            screen: text(r, "screen"),
            group: whole(r, "group") as usize,
            group_index: whole(r, "groupIndex") as usize,
        })
        .collect();
    (model, view)
}

/// The focused item's objectName.
/// Back on plate `k`: Open's Left and a maker's Right land on a plate, LT and RT step to its maker, then Down and Right along.
fn go_to(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, grid: &emusen_kit_core::group_grid::GroupGrid, places: &[(usize, usize)], k: usize) {
    for _ in 0..64 {
        let now = focused(scene);
        let Some(f) = now.strip_prefix("plate.").and_then(|n| n.parse::<usize>().ok()) else {
            let control = match now.as_str() {
                "consoleOpen" | "consoleOptions" => PadControl::Left,
                "sort" => PadControl::Down,
                n if n.starts_with("chip.") => PadControl::Down,
                _ => PadControl::Right,
            };
            press(scene, sdl, pad, control);
            continue;
        };
        let (from, to) = (grid.items[f], grid.items[k]);
        let control = match places[f].0.cmp(&places[k].0) {
            std::cmp::Ordering::Less => PadControl::RightTrigger,
            std::cmp::Ordering::Greater => PadControl::LeftTrigger,
            _ if f == k => return,
            _ if from.row < to.row => PadControl::Down,
            _ if from.row > to.row => PadControl::Up,
            _ if from.column < to.column => PadControl::Right,
            _ => PadControl::Left,
        };
        press(scene, sdl, pad, control);
    }
    panic!("plate.{k} reached again; the focus is on {}", focused(scene));
}

/// The visible item of the tree by name; of a Line (parts/Line.qml), whose words carry its name too, the words.
fn shown<'a>(tree: &'a Tree, name: &str) -> Option<&'a emusen_qt_test::tree::Node> {
    let named = || tree.nodes.iter().filter(|n| n.name == name && n.visible);
    named().find(|n| n.text.is_some()).or_else(|| named().next())
}

/// A box's edges: left, top, right, bottom.
fn edges(n: &emusen_qt_test::tree::Node) -> [f64; 4] {
    [n.x, n.y, n.x + n.w, n.y + n.h]
}

fn overlap(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// Whether the Line `name` lays its words out whole: not `truncated`, and no word wider than its box (EmuSen_Qt.md §3.4.9).
fn whole_words(tree: &Tree, name: &str) -> bool {
    let read = |p: &str| crate::eclipse_board::read(tree, name, p);
    let number = |p: &str| read(p).parse::<f64>().unwrap_or(f64::NAN);
    read("truncated") == "false" && number("contentWidth") <= number("width") + 1e-6
}

/// The name a console's plate shows, as Qt lays out its measure `plateMeasure.<key>` (EmuSen_Qt.md §3.4.9).
fn plate_name(tree: &Tree, c: &Console) -> String {
    if whole_words(tree, &format!("plateMeasure.{}", c.key)) { c.display_name.clone() } else { c.name.clone() }
}

/// No name on a plate or beside the disc is cut, and what follows the name beside the disc stands below it.
fn names_whole(tree: &Tree, plates: usize, size: &str) {
    for k in 0..plates {
        let name = format!("plateName.{k}");
        if tree.nodes.iter().any(|n| n.name == name) {
            assert!(whole_words(tree, &name), "{size}: {name} {:?} is cut", crate::eclipse_board::read(tree, &name, "text"));
        }
    }
    assert!(whole_words(tree, "consoleName"), "{size}: the focused console's name {:?} is cut", crate::eclipse_board::read(tree, "consoleName", "text"));
    let name = edges(shown(tree, "consoleName").expect("the focused console's name"));
    for below in ["consoleFacts", "consoleOpen", "consoleOptions"] {
        let e = edges(shown(tree, below).unwrap_or_else(|| panic!("{size}: {below} shown")));
        assert!(e[1] >= name[3] - 1e-6, "{size}: {below} {e:?} stands below the name {name:?}");
    }
}

/// The words a plate and the place line say for a console, and its four facts, from the program's words over the model's row.
fn says(c: &Console, groups: &[usize], plate: String) -> ([String; 3], [String; 4]) {
    let year = if c.year > 0 { c.year.to_string() } else { String::new() };
    (
        [
            plate,
            words::line(&[year.as_str(), if c.count > 0 { words::games(c.count) } else { words::NO_GAMES_LINE.to_owned() }.as_str()]),
            words::line(&[c.maker.as_str(), emusen_qt_models::game_page::place(c.group_index, groups[c.group]).as_str()]),
        ],
        [words::grouped(c.count), words::grouped(c.played), c.engine_name.clone(), c.screen.clone()],
    )
}

/// The disc's rim and both glows in the focused console's own colour, each glow at its own strength.
fn disc_holds(tree: &Tree, c: &Console, size: &str) {
    let glow = colour(&format!("#FF{}", c.glow.trim_start_matches('#'))).expect("a glow");
    let paint = |name: &str| shown(tree, name).and_then(|n| n.colors.iter().filter_map(|p| colour(p)).find(|p| p.a > 0)).unwrap_or_else(|| panic!("{size}: {name} painted"));
    let rim = paint("coronaRim");
    assert_eq!((rim.r, rim.g, rim.b), (glow.r, glow.g, glow.b), "{size}: {}'s rim is its colour {}", c.display_name, c.glow);
    let (inner, outer) = (paint("coronaGlow.0"), paint("coronaGlow.1"));
    for g in [inner, outer] {
        assert_eq!((g.r, g.g, g.b), (glow.r, glow.g, glow.b), "{size}: {}'s glows are its colour {}", c.display_name, c.glow);
    }
    assert_ne!(inner.a, outer.a, "{size}: each glow at its own strength");
}

/// The walk, the table and the pictures over the running program, on the software backend or the card as `backend` names it.
pub fn oracle(backend: &str) {
    let world = World::new("qt-app-eclipse-consoles");
    let cores = world.dir("lib");
    let (model, view) = library_of(&world, &cores);
    assert!(model.iter().any(|c| c.key == "psx") && model.len() > 5, "the consoles a core runs, the PlayStation among them: {model:?}");
    let groups: Vec<usize> = (0..=model.iter().map(|c| c.group).max().unwrap()).map(|g| model.iter().filter(|c| c.group == g).count()).collect();
    assert!(groups.len() >= 2, "two makers or more, so LT and RT step between them: {groups:?}");
    let colours: Vec<&str> = model.iter().map(|c| c.glow.as_str()).collect();

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    // The machine's battery at the board's 82%, discharging, in the world's made-up power_supply tree (EclipseConsoles:37).
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();

    // Home's Games place by the pad, then Eclipse chosen over it, which places its own Choose a console.
    for _ in 0..40 {
        if !focused(&mut scene).is_empty() {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..12 {
        let now = focused(&mut scene);
        if now == "place.0" {
            break;
        }
        press(&mut scene, &sdl, &pad, if now.starts_with("place.") { PadControl::Left } else { PadControl::Down });
    }
    assert_eq!(focused(&mut scene), "place.0", "Home's Games place reached by the pad");
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    wait_for(&mut scene, "theme", "consoles", "true", 2000);
    for _ in 0..40 {
        if focused(&mut scene).starts_with("plate.") {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), "plate.0", "the first maker's first console has the focus");
    // The battery's level alone, the board's words, and none of Home's picture of it (EclipseConsoles:37).
    let mut tree = scene.tree();
    for _ in 0..100 {
        if shown(&tree, "battery").is_some_and(|n| n.text.is_some()) {
            break;
        }
        scene.wait(50);
        tree = scene.tree();
    }
    assert_eq!(shown(&tree, "battery").and_then(|n| n.text.clone()).as_deref(), Some("82%"), "the battery's level as the board draws it");
    assert!(shown(&tree, "batteryIcon").is_none(), "Choose a console's board draws no picture of the battery");
    board_held(&mut scene, &model[0]);
    pictures(&mut scene, backend, "eleven");

    // The screen's plates are the model's consoles, in its order.
    let tree = scene.tree();
    let drawn: Vec<(String, String, f64, f64)> = tree
        .nodes
        .iter()
        .filter(|n| n.name.starts_with("plateName.") && n.text.is_some())
        .map(|n| (n.name.clone(), n.text.clone().unwrap_or_default(), n.x, n.y))
        .collect();
    for (k, c) in model.iter().enumerate() {
        let name = drawn.iter().find(|d| d.0 == format!("plateName.{k}")).map(|d| d.1.clone());
        assert_eq!(name, Some(plate_name(&tree, c)), "plate.{k} is the model's: {drawn:?}");
    }
    assert_eq!(plate_name(&tree, &model[0]), model[0].name, "the Nintendo Entertainment System's plate gives way to its short name");
    let places: Vec<(usize, usize)> = model.iter().map(|c| (c.group, c.group_index)).collect();
    let firsts: Vec<usize> = (0..groups.len()).map(|g| model.iter().position(|c| c.group == g && c.group_index == 0).unwrap()).collect();
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(200);
        let size = format!("{w}x{h}");
        let columns: usize = scene.property("consoles", "columns").parse().expect("the screen's columns");
        let grid = group_grid(&places, columns);
        let at_plate = |name: &str| name.strip_prefix("plate.").and_then(|n| n.parse::<usize>().ok());
        let mut reached = vec![false; model.len()];
        let start = at_plate(&focused(&mut scene)).expect("a plate focused");
        let mut queue = vec![start];
        while let Some(k) = queue.pop() {
            if reached[k] {
                continue;
            }
            reached[k] = true;
            go_to(&mut scene, &sdl, &pad, &grid, &places, k);
            let item = grid.items[k];
            let c = &model[k];
            let last_row = item.row + 1 >= grid.rows[c.group];
            // Where each press lands: the walk in the grid, the maker, Open, the next maker's first, or All (EmuSen_Qt.md §3.4.9).
            let want = [
                (PadControl::Left, item.left.map_or(format!("maker.{}", c.group), |t| format!("plate.{t}"))),
                (PadControl::Right, item.right.map_or("consoleOpen".to_owned(), |t| format!("plate.{t}"))),
                (PadControl::Up, item.up.map_or("chip.0".to_owned(), |t| format!("plate.{t}"))),
                (
                    PadControl::Down,
                    item.down.or(if last_row { firsts.get(c.group + 1).copied() } else { None }).map_or(format!("plate.{k}"), |t| format!("plate.{t}")),
                ),
            ];
            for (control, to) in want {
                press(&mut scene, &sdl, &pad, control);
                let now = focused(&mut scene);
                assert_eq!(now, to, "{size}: {control:?} from plate.{k} ({})", c.display_name);
                if let Some(t) = at_plate(&now) {
                    queue.push(t);
                }
                go_to(&mut scene, &sdl, &pad, &grid, &places, k);
            }

            // On the plate: drawn whole inside the window and the grid's view, over no other plate, maker or the aside.
            let tree = scene.tree();
            let plate = shown(&tree, &format!("plate.{k}")).unwrap_or_else(|| panic!("{size}: plate.{k} shown"));
            let view = shown(&tree, "plates").expect("the plates' view");
            let (p, v) = (edges(plate), edges(view));
            assert!(p[0] >= v[0] && p[1] >= v[1] && p[2] <= v[2] && p[3] <= v[3], "{size}: plate.{k} {p:?} whole in its view {v:?}");
            assert!(p[0] >= 0.0 && p[1] >= 0.0 && p[2] <= f64::from(w) && p[3] <= f64::from(h), "{size}: plate.{k} {p:?} inside the window");
            let others = tree.nodes.iter().filter(|n| {
                n.visible && n.name != plate.name && (n.name.starts_with("plate.") || n.name.starts_with("maker.") || n.name == "consoleAside")
            });
            for o in others {
                let e = edges(o);
                let seen = o.name.starts_with("plate.") && (e[1] >= v[3] || e[3] <= v[1]);
                assert!(seen || !overlap(p, e), "{size}: plate.{k} {p:?} over {} {e:?}", o.name);
            }
            // Its words and the facts are the model's, the played count among them; the disc is in its colour.
            let ([name, line, place], values) = says(c, &groups, plate_name(&tree, c));
            let said = |n: &str| shown(&tree, n).and_then(|n| n.text.clone()).unwrap_or_default();
            assert_eq!(said(&format!("plateName.{k}")), name, "{size}: plate.{k}'s name");
            assert_eq!(said(&format!("plateLine.{k}")), line, "{size}: plate.{k}'s line");
            assert_eq!(said("consoleName"), c.display_name, "{size}: the focused console's full name");
            names_whole(&tree, model.len(), &size);
            for (i, (label, value)) in words::FACT_LABELS.iter().zip(&values).enumerate() {
                assert_eq!(said(&format!("consoleFactLabel.{i}")), *label, "{size}: the facts' row {i}'s label");
                assert_eq!(said(&format!("consoleFactValue.{i}")), *value, "{size}: {}'s {label}", c.display_name);
            }
            if c.key == "psx" {
                // Serenity's screens name the PlayStation a television: the row says so in words, and the table draws them.
                assert!(!c.screen.is_empty() && c.screen == words::screen("Television"), "the PlayStation's screen: {:?}", c.screen);
            }
            assert_eq!(shown(&tree, "consoleCoreMark").is_some(), !c.engine_name.is_empty(), "{size}: the core's planet beside its name");
            assert_eq!(said("consolePlace"), place, "{size}: the maker and the place");
            disc_holds(&tree, c, &size);
        }
        let missed: Vec<&str> = model.iter().zip(&reached).filter(|(_, r)| !**r).map(|(c, _)| c.display_name.as_str()).collect();
        assert!(missed.is_empty(), "{size}: every plate reached by the pad; not {missed:?}");
    }
    scene.resize(1280, 800);
    scene.wait(200);

    // Two consoles of different colours, each drawn in its own.
    let columns: usize = scene.property("consoles", "columns").parse().expect("the screen's columns");
    let grid = group_grid(&places, columns);
    let other = (1..model.len()).find(|&k| colours[k] != colours[0]).expect("two consoles of different colours");
    for k in [0, other] {
        go_to(&mut scene, &sdl, &pad, &grid, &places, k);
        let tree = scene.tree();
        disc_holds(&tree, &model[k], "1280x800");
    }

    // LT and RT: the previous and next maker's first console, from any plate; nothing past the ends; LB and RB do nothing.
    go_to(&mut scene, &sdl, &pad, &grid, &places, 0);
    press(&mut scene, &sdl, &pad, PadControl::LeftTrigger);
    assert_eq!(focused(&mut scene), "plate.0", "LT on the first maker stays");
    for g in 1..groups.len() {
        if groups[g - 1] > 1 {
            press(&mut scene, &sdl, &pad, PadControl::Right);
        }
        press(&mut scene, &sdl, &pad, PadControl::RightTrigger);
        assert_eq!(focused(&mut scene), format!("plate.{}", firsts[g]), "RT to maker {g}'s first console");
    }
    press(&mut scene, &sdl, &pad, PadControl::RightTrigger);
    assert_eq!(focused(&mut scene), format!("plate.{}", firsts[groups.len() - 1]), "RT on the last maker stays");
    press(&mut scene, &sdl, &pad, PadControl::LeftTrigger);
    assert_eq!(focused(&mut scene), format!("plate.{}", firsts[groups.len() - 2]), "LT to the maker before");
    // LB and RB step the sections (R-20261115-17, R-20261126-2): LB opens Library, RB steps back to Consoles, and B from Library returns to Home.
    press(&mut scene, &sdl, &pad, PadControl::LeftShoulder);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    assert_eq!(scene.property("screens", "key"), emusen_qt_models::library::ALL, "LB on Consoles opens Library, every game (R-20261115-17)");
    press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    for _ in 0..40 {
        if focused(&mut scene).starts_with("plate.") {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), format!("plate.{}", firsts[groups.len() - 2]), "back from Library, the plate keeps the focus");
    let tree = scene.tree();
    let bar = edges(shown(&tree, "consolesHelp").expect("the help line"));
    let help: Vec<String> = tree.nodes.iter().filter(|n| n.visible && n.text.is_some() && overlap(edges(n), bar)).filter_map(|n| n.text.clone()).collect();
    assert!(help.iter().any(|t| t == "Maker") && !help.iter().any(|t| t == "Group"), "the help line draws LT RT Maker and no LB RB: {help:?}");
    let entries: Vec<&String> = help.iter().filter(|t| ["Open", "Back", "Maker", "Favourites", "Random game"].contains(&t.as_str())).collect();
    assert_eq!(entries, ["Open", "Back", "Maker"], "the board's entries, in its order, and none of the screen's own actions: {help:?}");

    // A on a plate opens that console's games; B comes back to it.
    let k = firsts[groups.len() - 2];
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    assert_eq!(scene.property("screens", "key"), model[k].key, "A opens {}'s games", model[k].display_name);
    press(&mut scene, &sdl, &pad, PadControl::East);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    for _ in 0..40 {
        if focused(&mut scene) == format!("plate.{k}") {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), format!("plate.{k}"), "back on the console it opened");
    // RT, Left, Up through the makers to the chips, then Down: the first maker's first console, not a hidden plate (§3.4.9).
    press(&mut scene, &sdl, &pad, PadControl::RightTrigger);
    assert_eq!(focused(&mut scene), format!("plate.{}", firsts[groups.len() - 1]), "RT to the last maker's first console");
    press(&mut scene, &sdl, &pad, PadControl::Left);
    assert_eq!(focused(&mut scene), format!("maker.{}", groups.len() - 1), "Left from the first column to its maker");
    for _ in 0..=groups.len() {
        if focused(&mut scene).starts_with("chip.") {
            break;
        }
        press(&mut scene, &sdl, &pad, PadControl::Up);
    }
    assert!(focused(&mut scene).starts_with("chip."), "Up from the first maker to the chips");
    press(&mut scene, &sdl, &pad, PadControl::Down);
    assert_eq!(focused(&mut scene), format!("plate.{}", firsts[0]), "Down from the chips to the chosen maker's first console");
    chips_sort_and_options(&mut scene, &sdl, &pad, &model, &world, &cores);
    wheel_scrolls(&mut scene, &sdl, &pad, &view, backend);
    sdl.detach(pad);
    scene.assert_quiet();
}

/// With `EMUSEN_ECLIPSE_PICTURES` naming a folder, the screen grabbed at the board's shape and the handheld's into it.
fn pictures(scene: &mut Scene<'_>, backend: &str, library: &str) {
    let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") else { return };
    std::fs::create_dir_all(&folder).unwrap();
    for (width, height) in [(1920, 1200), (1280, 800)] {
        scene.resize(width, height);
        scene.wait(500);
        let path = std::path::Path::new(&folder).join(format!("consoles-{library}-{width}x{height}-{backend}.png"));
        assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
    }
}

/// Whether `name` is drawn whole inside the view `view`.
fn whole_in(tree: &Tree, name: &str, view: &str) -> bool {
    let (Some(n), Some(v)) = (shown(tree, name), shown(tree, view)) else { return false };
    let (n, v) = (edges(n), edges(v));
    n[0] >= v[0] - 1e-6 && n[1] >= v[1] - 1e-6 && n[2] <= v[2] + 1e-6 && n[3] <= v[3] + 1e-6
}

/// The wheel scrolls the plates and the makers, and the pad's next press brings its landing back into sight (EmuSen_Qt.md §3.4.9, Proof).
fn wheel_scrolls(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, view: &LibraryView, backend: &str) {
    scene.resize(1280, 800);
    scene.wait(200);
    let first = text(&consoles_listed(view, Show::All, Order::Maker, &[])[0], "key");
    let seed = view.cored.iter().find(|r| r.key == first).expect("the first maker's first console").clone();
    let mut crowd = view.clone();
    for n in 0..60 {
        let mut row = seed.clone();
        row.key = format!("{}-{n:02}", seed.key);
        row.name = format!("{} {n:02}", seed.name);
        if n >= 40 {
            row.maker = format!("Maker {n:02}");
        }
        crowd.cored.push(row);
    }
    let tree = scene.tree();
    let screen: u64 = tree.nodes[tree.find_named("consolesScreen").expect("Eclipse's Choose a console") as usize].id.parse().unwrap();
    emusen_theme_qt::host::library(crate::eclipse_board::object(screen)).expect("the theme's Library").show(&crowd, "[]");
    scene.wait(300);
    for _ in 0..30 {
        if focused(scene) == "plate.0" {
            break;
        }
        press(scene, sdl, pad, PadControl::LeftTrigger);
    }
    assert_eq!(focused(scene), "plate.0", "the first maker's first console by LT");
    pictures(scene, backend, "crowd");
    let at = |scene: &mut Scene<'_>, view: &str| -> f64 {
        let tree = scene.tree();
        crate::eclipse_board::read(&tree, view, "contentY").parse().expect("a view's contentY")
    };
    assert_eq!(at(scene, "plates"), 0.0, "the plates at their top");
    let (x, y) = middle(scene, "plates");
    scene.wheel(x, y, -10);
    scene.wait(800);
    assert!(at(scene, "plates") > 0.0, "the wheel over the plates scrolls them");
    assert!(!whole_in(&scene.tree(), "plate.1", "plates"), "the first row scrolled out of sight");
    press(scene, sdl, pad, PadControl::Right);
    assert_eq!(focused(scene), "plate.1", "Right along the first row");
    assert!(whole_in(&scene.tree(), "plate.1", "plates"), "the plate Right landed on is brought back into sight");

    assert_eq!(at(scene, "makers"), 0.0, "the makers at their top");
    let (x, y) = middle(scene, "makers");
    scene.wheel(x, y, -10);
    scene.wait(800);
    assert!(at(scene, "makers") > 0.0, "the wheel over the makers scrolls them");
    assert!(!whole_in(&scene.tree(), "maker.1", "makers"), "the second maker scrolled out of sight");
    press(scene, sdl, pad, PadControl::RightTrigger);
    assert!(focused(scene).starts_with("plate."), "RT to the next maker's first console");
    assert!(whole_in(&scene.tree(), "maker.1", "makers"), "the maker RT chose is brought back into sight");
    every_system_whole(scene, view, screen);
    empty_held(scene, view, screen);
}

/// A chip that holds none, drawn as the first-run board, the table's `empty` rows at every gate size (R-20261116-9; EmuSen_Qt.md §3.4.9).
fn empty_held(scene: &mut Scene<'_>, view: &LibraryView, screen: u64) {
    let (board_height, rows) = crate::eclipse_board::rows(BOARD);
    let rows: Vec<_> = rows.into_iter().filter(|r| r.status == "empty").map(|mut r| {
        r.status = "built".into();
        r
    }).collect();
    assert_eq!(rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(), ["noConsolesWords", "noConsolesAbout"]);
    let k = DESIGN_HEIGHT / board_height;
    let library = |cored: Vec<emusen_qt_models::library::LibraryRow>| {
        let mut shown = view.clone();
        shown.cored = cored;
        emusen_theme_qt::host::library(crate::eclipse_board::object(screen)).expect("the theme's Library").show(&shown, "[]");
    };
    let mut differences = Vec::new();
    let every = emusen_qt_models::library::console_rows(&view.systems, &Default::default(), &[]);
    for (chip, cored, held, words_said) in [(0, Vec::new(), 2, (words::NO_CONSOLES, words::ADD_FOLDER)), (2, every, 1, (words::NO_GAMES, ""))] {
        library(cored);
        scene.wait(300);
        let (x, y) = middle(scene, &format!("chip.{chip}"));
        scene.click(x, y);
        scene.wait(300);
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let at = format!("{width}x{height}, chip {chip}");
            assert!(!tree.nodes.iter().any(|n| n.visible && n.name.starts_with("plate.")), "{at}: the chip holds no console");
            let said = |n: &str| shown(&tree, n).and_then(|n| n.text.clone()).unwrap_or_default();
            assert_eq!((said("noConsolesWords").as_str(), said("noConsolesAbout").as_str()), words_said, "{at}: the chip's own words");
            let unit: f64 = crate::eclipse_board::read(&tree, "consolesCanvas", "unit").parse().expect("the canvas's unit");
            let id: u64 = tree.nodes[tree.find_named("consolesScreen").expect("Eclipse's Choose a console") as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(id);
            crate::eclipse_board::held(&tree, &rows[..held], k, unit, f64::from(width) / unit, &tokens, &at, &|_, _, _| false, &mut differences);
            // Nothing else stands where the words do: no maker, no plate, no aside.
            let headline = edges(shown(&tree, "noConsolesWords").expect("the headline"));
            let under: Vec<&str> = tree.nodes.iter().filter(|n| n.visible && (n.name.starts_with("maker.") || n.name == "consoleAside") && overlap(headline, edges(n))).map(|n| n.name.as_str()).collect();
            assert!(under.is_empty(), "{at}: the headline {headline:?} over {under:?}");
        }
    }
    assert!(differences.is_empty(), "{} differences from the board:\n{}", differences.len(), differences.join("\n"));
}

/// Every console of the systems table on a plate, its name whole or its short one, at every gate size (R-20261116-5).
fn every_system_whole(scene: &mut Scene<'_>, view: &LibraryView, screen: u64) {
    let mut every = view.clone();
    every.cored = emusen_qt_models::library::console_rows(&view.systems, &Default::default(), &[]);
    assert!(every.cored.len() > view.cored.len(), "the table holds more consoles than the fixture shelves");
    emusen_theme_qt::host::library(crate::eclipse_board::object(screen)).expect("the theme's Library").show(&every, "[]");
    scene.wait(300);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let size = format!("{width}x{height}");
        names_whole(&tree, every.cored.len(), &size);
        for r in &every.cored {
            let measure = format!("plateMeasure.{}", r.key);
            if !whole_words(&tree, &measure) {
                assert_ne!(r.name, r.shown_name(), "{size}: {}'s full name is cut and it has no short one", r.shown_name());
            }
        }
    }
}

/// The plates' names drawn, in order.
fn plate_names(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let mut named: Vec<(usize, String)> = tree
        .nodes
        .iter()
        .filter(|n| n.text.is_some())
        .filter_map(|n| n.name.strip_prefix("plateName.").and_then(|k| k.parse().ok()).map(|k| (k, n.text.clone().unwrap_or_default())))
        .collect();
    named.sort();
    named.dedup();
    named.into_iter().map(|(_, t)| t).collect()
}

/// The middle of a shown element, where a pointer presses it.
fn middle(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let n = tree.nodes.iter().find(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("{name} shown"));
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

/// The chips, Y and X, each by the pad and by the pointer, and B back from Settings (EmuSen_Qt.md §3.4.9).
fn chips_sort_and_options(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, model: &[Console], world: &World, cores: &std::path::Path) {
    let _ = world;
    let tree = scene.tree();
    let plate = |c: &Console| plate_name(&tree, c);
    let of_key = |key: &str| model.iter().find(|c| c.key == key).expect("a console of the model");
    let source_names = |show: Show, order: Order| -> Vec<String> {
        let source = emusen_qt_models::clients::source_for(&library().to_string_lossy(), cores.to_str().unwrap()).unwrap();
        let mut view = LibraryView::empty();
        view.apply(refresh(&source));
        consoles_listed(&view, show, order, &[]).iter().map(|r| plate(of_key(&text(r, "key")))).collect()
    };
    let all: Vec<String> = model.iter().map(plate).collect();
    let with: Vec<String> = model.iter().filter(|c| c.count > 0).map(plate).collect();
    let favourites = source_names(Show::Favourites, Order::Maker);
    assert!(with.len() < all.len() && favourites.len() == 1, "the fixture: {all:?} {with:?} {favourites:?}");
    let said = |scene: &mut Scene<'_>, name: &str| {
        let tree = scene.tree();
        shown(&tree, name).and_then(|n| n.text.clone()).unwrap_or_default()
    };
    for (i, n) in [all.len(), with.len(), favourites.len()].into_iter().enumerate() {
        assert_eq!(said(scene, &format!("chipWords.{i}")), words::line(&[words::CONSOLE_SHOWS[i], &words::grouped(n)]), "chip {i}'s words and count");
    }
    assert_eq!(plate_names(scene), all, "All shows every console a core runs, the one with no game among them");

    // By the pad: Up from a plate's first row to the chips, Right along them, A presses one.
    for _ in 0..8 {
        if focused(scene).starts_with("chip.") {
            break;
        }
        press(scene, sdl, pad, PadControl::Up);
    }
    assert_eq!(focused(scene), "chip.0", "Up from the plates' first row reaches the pressed chip");
    press(scene, sdl, pad, PadControl::Right);
    assert_eq!(focused(scene), "chip.1");
    press(scene, sdl, pad, PadControl::South);
    scene.wait(200);
    assert_eq!(plate_names(scene), with, "With games by the pad");
    for _ in 0..8 {
        if focused(scene) == "chip.1" {
            break;
        }
        press(scene, sdl, pad, PadControl::Up);
    }
    press(scene, sdl, pad, PadControl::Right);
    press(scene, sdl, pad, PadControl::South);
    scene.wait(200);
    assert_eq!(plate_names(scene), favourites, "Favourites by the pad");
    // By the pointer: All again.
    let (x, y) = middle(scene, "chip.0");
    scene.click(x, y);
    scene.wait(200);
    assert_eq!(plate_names(scene), all, "All by the pointer");

    // Y by the pad, then the entry by the pointer: the order steps, the entry says it, the plates follow it.
    assert_eq!(said(scene, "sortWords"), words::CONSOLE_SORTS[0]);
    for _ in 0..8 {
        if focused(scene).starts_with("plate.") {
            break;
        }
        press(scene, sdl, pad, PadControl::Down);
    }
    press(scene, sdl, pad, PadControl::North);
    scene.wait(200);
    assert_eq!(said(scene, "sortWords"), words::CONSOLE_SORTS[1], "Y steps to the next order");
    assert_eq!(plate_names(scene), source_names(Show::All, Order::Year), "the plates by year");
    let (x, y) = middle(scene, "sort");
    scene.click(x, y);
    scene.wait(200);
    assert_eq!(said(scene, "sortWords"), words::CONSOLE_SORTS[2], "the entry pressed by the pointer steps it too");
    // By the names read from the screen: each maker's plates in the alphabet's order of the names they show.
    let drawn = plate_names(scene);
    let maker = |name: &String| model.iter().find(|c| &plate(c) == name).map(|c| c.maker.clone()).expect("a console of the model");
    for run in drawn.chunk_by(|a, b| maker(a) == maker(b)) {
        let mut sorted = run.to_vec();
        sorted.sort_by_key(|n| n.to_lowercase());
        assert_eq!(run, sorted.as_slice(), "the plates by the names they show");
    }
    assert_eq!(drawn.len(), model.len(), "every console by name");
    for _ in 0..2 {
        let (x, y) = middle(scene, "sort");
        scene.click(x, y);
        scene.wait(100);
    }
    assert_eq!(said(scene, "sortWords"), words::CONSOLE_SORTS[0], "round to the maker's order");

    // X by the pad on a plate, then Options by the pointer: Settings at the console's page; B twice comes back.
    for by_pad in [true, false] {
        for _ in 0..8 {
            if focused(scene).starts_with("plate.") {
                break;
            }
            press(scene, sdl, pad, PadControl::Down);
        }
        let k: usize = focused(scene).strip_prefix("plate.").and_then(|k| k.parse().ok()).expect("a plate focused");
        if by_pad {
            press(scene, sdl, pad, PadControl::West);
        } else {
            let (x, y) = middle(scene, "consoleOptions");
            scene.click(x, y);
        }
        wait_for(scene, "screens", "shown", "Settings", 2000);
        assert_eq!(scene.property("screens", "settingsAt"), model[k].key, "Settings asked at {}'s page", model[k].display_name);
        // The page's trail names the console as Settings lists it, the systems table's short name.
        let tree = scene.tree();
        let texts: Vec<&str> = tree.nodes.iter().filter(|n| n.visible).filter_map(|n| n.text.as_deref()).collect();
        assert!(texts.contains(&model[k].name.as_str()), "{}'s page open over the consoles' section: {texts:?}", model[k].name);
        // B once Settings holds the focus closes the console's page, then leaves Settings (Main.qml's settingsSlot).
        let mut seen = Vec::new();
        for _ in 0..2 {
            for _ in 0..40 {
                if !focused(scene).is_empty() {
                    break;
                }
                scene.wait(50);
            }
            press(scene, sdl, pad, PadControl::East);
            seen.push((scene.property("screens", "shown"), focused(scene)));
        }
        wait_for(scene, "screens", "shown", "Consoles", 2000);
        assert_eq!(scene.property("screens", "shown"), "Consoles", "B twice from the console's page: {seen:?}");
    }
}
