//! Eclipse's Choose a console in the running program (EmuSen_Qt.md §3.4.9), driven by an SDL pad inside this process over a
//! library of the eleven canvas consoles, some games played: at each of the gate's sizes every press from every plate lands
//! where the layout's walk puts it and every plate is reached, none cut or overlapping; A opens the console's games; LT and
//! RT step the makers; the disc's rim and glows are the focused console's own colour; a plate's words and the played count
//! are the model's. One test, as the harness is one per process.

mod common;
mod eclipse_board;

use common::{approved_console_core, wait_for, CONSOLES};
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
use emusen_qt_endymion::reading::sdl_button;
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

/// SDL's trigger axes (SDL_GAMEPAD_AXIS_LEFT_TRIGGER and _RIGHT_TRIGGER) and a full pull.
const LEFT_TRIGGER: i32 = 4;
const RIGHT_TRIGGER: i32 = 5;
const PULLED: i16 = i16::MAX;

/// The board's elements, one a row (eclipse/consoles.tsv), held by eclipse_board's rules.
const BOARD: &str = include_str!("eclipse/consoles.tsv");

/// Every row of the board's table held to the screen at each of the gate's sizes, with `c` focused: its box, colour, type
/// and part; and both fields painted in the board's saturation and lightness with `c`'s hue (R-20261115-28).
fn board_held(scene: &mut Scene<'_>, c: &Console) {
    let (board_height, rows) = eclipse_board::rows(BOARD);
    let k = DESIGN_HEIGHT / board_height;
    let hue = Colour::parse(&c.glow).expect("the console's glow").to_hsl().0;
    let mut differences = Vec::new();
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let at = format!("{width}x{height}");
        let unit: f64 = eclipse_board::read(&tree, "consolesCanvas", "unit").parse().expect("the canvas's unit");
        let screen: u64 = tree.nodes[tree.find_named("consolesScreen").expect("Eclipse's Choose a console") as usize].id.parse().unwrap();
        let tokens = eclipse_board::tokens(screen);
        // The battery is drawn only where the machine reports one, as Status draws it.
        let no_battery = eclipse_board::read(&tree, "battery", "text").is_empty();
        eclipse_board::held(&tree, &rows, k, unit, f64::from(width) / unit, &tokens, &at, &|name, _, _| name == "battery" && no_battery, &mut differences);
        // The disc's glows blurred at the scene's pixels.
        let scale = eclipse_board::read(&tree, "consoleDisc", "pixelScale").parse::<f64>().unwrap_or(f64::NAN);
        if (scale - unit).abs() > 1e-9 {
            differences.push(format!("{at}: the disc's glows blurred at {scale} pixels a unit where the scene draws {unit}"));
        }
        for row in rows.iter().filter(|r| r.kind.starts_with("field ")) {
            let board = Colour::parse(row.kind.split(' ').nth(3).unwrap()).expect("a field's board colour").to_hsl();
            let have = Colour::parse(&eclipse_board::read(&tree, &row.name, "paint")).map(|p| p.to_hsl());
            // Two degrees and a hundredth of saturation and lightness: the colour read back as 8 bits a channel.
            if !have.is_some_and(|h| emusen_kit_core::world::wrap_hue(h.0 - hue).abs() <= 2.0 && (h.1 - board.1).abs() <= 0.01 && (h.2 - board.2).abs() <= 0.01) {
                differences.push(format!("{at}: {} painted {have:?} where the board's colour with {}'s hue {hue:.1} gives {board:?}", row.name, c.display_name));
            }
        }
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

/// The eleven consoles' games, in each console's ES-DE folder, which settles an extension two consoles share
/// (`Platform/library/src/scan.rs`, `route`): one more for each console in the canvas's order up to three, the first of each
/// played and the second of the first console too. A Game Boy Color game carries its header's colour flag (0x143).
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

/// The library written, the fixture's PlayStation core approved in `cores` beside the build's own, and its plays recorded
/// through the real clients; the consoles as `Library.consoles` gives them: those of the eleven a core runs.
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
    let model = consoles_listed(&view, Show::All, Order::Maker)
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
fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// A control pressed and let go on the pad, as a hand does: down until the router holds it (or the screen it opens is
/// shown, whose frame holds nothing yet), up until it has let it go.
fn press(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, control: PadControl) {
    let button = control.press().expect("a control the interface hears");
    let shown = scene.property("screens", "shown");
    let set = |down: bool| match sdl_button(control) {
        Some(b) => sdl.press(pad, b, down),
        None => sdl.axis(pad, if control == PadControl::LeftTrigger { LEFT_TRIGGER } else { RIGHT_TRIGGER }, if down { PULLED } else { 0 }),
    };
    for down in [true, false] {
        set(down);
        let heard = |s: &mut Scene<'_>| s.pad_held(button) == down || (down && s.property("screens", "shown") != shown);
        let mut waited = 0;
        while !heard(scene) && waited < 2000 {
            scene.wait(8);
            waited += 8;
        }
        assert!(heard(scene), "the router heard {control:?} {}; shown {}, focus {}", if down { "down" } else { "up" }, scene.property("screens", "shown"), focused(scene));
    }
    scene.wait(60);
}

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

/// The words a plate and the place line say for a console, and its four facts' values (games, played, core, screen), from
/// the program's words over the model's row.
fn says(c: &Console, groups: &[usize]) -> ([String; 3], [String; 4]) {
    let year = if c.year > 0 { c.year.to_string() } else { String::new() };
    (
        [
            c.display_name.clone(),
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

#[test]
fn qt_3_4_9_eclipse_s_consoles_walks_opens_steps_makers_and_draws_each_console_in_its_colour_in_the_program() {
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
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();

    // Home's Games place by the pad (Down from the played games to the places, Left to the first), then Eclipse chosen over
    // it: its own Choose a console is placed.
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
    board_held(&mut scene, &model[0]);

    // The screen's plates are the model's consoles, in its order.
    let tree = scene.tree();
    let drawn: Vec<(String, String, f64, f64)> = tree
        .nodes
        .iter()
        .filter(|n| n.name.starts_with("plateName.") && n.text.is_some())
        .map(|n| (n.name.clone(), n.text.clone().unwrap_or_default(), n.x, n.y))
        .collect();
    for (k, c) in model.iter().enumerate() {
        let name = drawn.iter().find(|d| d.0 == format!("plateName.{k}")).map(|d| d.1.as_str());
        assert_eq!(name, Some(c.display_name.as_str()), "plate.{k} is the model's: {drawn:?}");
    }
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
            // Where each press lands: the walk inside the grid; past its left edge the maker, past its right Open, down
            // from the last row the next maker's first; up from the first row the chip pressed, All.
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
            let ([name, line, place], values) = says(c, &groups);
            let said = |n: &str| shown(&tree, n).and_then(|n| n.text.clone()).unwrap_or_default();
            assert_eq!(said(&format!("plateName.{k}")), name, "{size}: plate.{k}'s name");
            assert_eq!(said(&format!("plateLine.{k}")), line, "{size}: plate.{k}'s line");
            assert_eq!(said("consoleName"), name, "{size}: the focused console's name");
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
    for shoulder in [PadControl::LeftShoulder, PadControl::RightShoulder] {
        press(&mut scene, &sdl, &pad, shoulder);
        assert_eq!(focused(&mut scene), format!("plate.{}", firsts[groups.len() - 2]), "{shoulder:?} is not bound here (R-20261115-17)");
    }
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
    // The last maker's first console by RT, Left to its maker, Up through the makers (each chosen in turn) to the chips, and
    // Down: the first maker's first console, not the plate last focused, which choosing another maker hid.
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
    wheel_scrolls(&mut scene, &sdl, &pad, &view);
    sdl.detach(pad);
    scene.assert_quiet();
}

/// Whether `name` is drawn whole inside the view `view`.
fn whole_in(tree: &Tree, name: &str, view: &str) -> bool {
    let (Some(n), Some(v)) = (shown(tree, name), shown(tree, view)) else { return false };
    let (n, v) = (edges(n), edges(v));
    n[0] >= v[0] - 1e-6 && n[1] >= v[1] - 1e-6 && n[2] <= v[2] + 1e-6 && n[3] <= v[3] + 1e-6
}

/// The wheel scrolls the plates and the makers, each under the pointer, and the pad's next press brings what it lands on
/// back into sight. Over a crowd shown to the theme's Library directly, since the program shelves only the consoles a core
/// runs: forty more consoles of the first maker and twenty makers more, so both columns are longer than the screen.
fn wheel_scrolls(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, view: &LibraryView) {
    scene.resize(1280, 800);
    scene.wait(200);
    let first = text(&consoles_listed(view, Show::All, Order::Maker)[0], "key");
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
    emusen_theme_qt::host::library(eclipse_board::object(screen)).expect("the theme's Library").show(&crowd, "[]");
    scene.wait(300);
    for _ in 0..30 {
        if focused(scene) == "plate.0" {
            break;
        }
        press(scene, sdl, pad, PadControl::LeftTrigger);
    }
    assert_eq!(focused(scene), "plate.0", "the first maker's first console by LT");
    let at = |scene: &mut Scene<'_>, view: &str| -> f64 {
        let tree = scene.tree();
        eclipse_board::read(&tree, view, "contentY").parse().expect("a view's contentY")
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

/// The chips, Y and X, each pressed by the pad and by the pointer: a chip shows its consoles with their count, Y and the
/// order's entry step the order, X and the Options pill open Settings at the console's page, and B comes back.
fn chips_sort_and_options(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, model: &[Console], world: &World, cores: &std::path::Path) {
    let _ = world;
    let source_names = |show: Show, order: Order| -> Vec<String> {
        let source = emusen_qt_models::clients::source_for(&library().to_string_lossy(), cores.to_str().unwrap()).unwrap();
        let mut view = LibraryView::empty();
        view.apply(refresh(&source));
        consoles_listed(&view, show, order).iter().map(|r| text(r, "displayName")).collect()
    };
    let all: Vec<String> = model.iter().map(|c| c.display_name.clone()).collect();
    let with: Vec<String> = model.iter().filter(|c| c.count > 0).map(|c| c.display_name.clone()).collect();
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
    assert_eq!(plate_names(scene), source_names(Show::All, Order::Name), "the plates by name");
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
        // B closes the console's page, then leaves Settings for Choose a console.
        // As a hand does, B once Settings holds the focus: its screen is made as it is shown (Main.qml's settingsSlot).
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
