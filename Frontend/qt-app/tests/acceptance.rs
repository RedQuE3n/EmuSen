//! Stage 1's acceptance over the canvas's library (EmuSen_Qt.md §7.1 step 4, §8.2): each screen in the canvas's own focus,
//! held to its `.mockup.spec` at 1280 x 800 and 1920 x 1200, and to the kit's gate at its five sizes; each state is
//! recorded as a spec and grabbed into `EMUSEN_QT_COMPARE` (else the temporary folder) for the side-by-side pictures.

mod common;

use common::{sandbox, wait_for, CONSOLES};

use emusen_client_fixture::library;
use emusen_crystal_client::Config;
use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt::MAIN_URL;
use emusen_platform_client::galaxia::consts::FILE_GAMES;
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use serde_json::json;
use emusen_qt_test::harness::{edges, palette, Measures, SIZES};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::{mockup, Harness, Scene};
use std::path::PathBuf;

/// The games on the canvas's shelves: ten on the Super Nintendo, as the grid's canvas shows, one on each other console.
const SNES_GAMES: usize = 10;

/// Choose a console's elements whose boxes follow from the canvas's eleven consoles; the fixture's engines shelve five, so
/// these are held by kit-core's layout test against the canvas's own numbers (EmuSen_Qt.md §7.1 step 4).
const COUNTED: [&str; 6] = ["consoleTile.", "consoleBody.", "consoleName.", "consoleCount.", "consoleRing.", "orbit.1"];

/// A fault explained by a difference named in EmuSen_Qt.md §7.1 step 4: the help bar's opaque Surface, the orbit line in the
/// canvas's own ink, the canvas's 17- and 18-unit words under the floor at 1280 x 720 alone, the focused console's name in
/// the canvas's line box of 1, an engine's name too long for its tile, and the fan's covers laid over each other.
fn named_difference(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    let item = |n: &str| n.trim().trim_end_matches(':').parse::<usize>().ok().and_then(|i| tree.nodes.get(i)).map_or("", |n| n.name.as_str());
    let after = |key: &str| fault.split(key).nth(1).map(|r| r.split_whitespace().next().unwrap_or(""));
    fault.contains("helpBar: alpha 1.000, the mockup's 0.92")
        || after("palette on ").is_some_and(|i| item(i).starts_with("orbit."))
        || (size == (1280, 720) && fault.contains("too small to read"))
        || after("cut at the foot: ").is_some_and(|i| item(i) == "consoleDetailName")
        || after("ellipsis: ").is_some_and(|i| item(i).starts_with("consoleEngineName."))
        || fault.contains("overlap: ") && fault.split("overlap: ").nth(1).is_some_and(|r| r.split(" and ").all(|i| item(i).starts_with("fanCover.")))
}

fn out() -> PathBuf {
    let dir = std::env::var_os("EMUSEN_QT_COMPARE").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// One synthetic image a game for each canvas console, under its own folder with its first extension; three played,
/// the first most recently, and two favourites, through the library's own changes.
fn canvas_library() -> emusen_client_fixture::World {
    let world = sandbox("qt-app-acceptance", &[]);
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    let systems = emusen_qt_models::library::LibrarySource::systems(&source).unwrap();
    let mut played = Vec::new();
    for id in CONSOLES {
        let system = systems.iter().find(|s| s.id == id).unwrap();
        let ext = system.extensions.first().unwrap().trim_start_matches('.').to_owned();
        let count = if id == "snes" { SNES_GAMES } else { 1 };
        let folder = world.dir(&format!("Roms/{id}"));
        for n in 0..count {
            let path = folder.join(format!("{} Game {}.{ext}", id.to_uppercase(), n + 1));
            std::fs::write(&path, [0u8; 4096]).unwrap();
            if played.len() < 3 && (id == "snes" || id == "ps2") && n < 2 {
                played.push(path);
            }
        }
    }
    emusen_qt_models::library::LibrarySource::games(&source).unwrap();
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    for (i, path) in played.iter().rev().enumerate() {
        let at = format!("2026-11-08T0{i}:00:00Z");
        games.change(&json!({ "op": "started", "path": path.to_str().unwrap(), "at": at })).unwrap();
    }
    games.change(&json!({ "op": "toggle_favourite", "path": played[0].to_str().unwrap() })).unwrap();
    world
}

fn named(scene: &mut Scene<'_>, name: &str) -> Option<u32> {
    scene.tree().find_named(name)
}

/// Holds one state: the mockup where the size is the canvas's or its 1.5 times, the gate at every size, the spec and the grab.
fn hold(scene: &mut Scene<'_>, screen: &str, m: &mockup::Mockup, report: &mut Vec<String>) {
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let mut tree = scene.tree();
        let focus = scene.focus(&tree);
        let name = format!("{screen}-{w}x{h}");
        let mut faults = Vec::new();
        if f64::from(w) / f64::from(h) == 1.6 {
            faults.extend(mockup::compare(m, &tree.nodes, f64::from(h) / 800.0, Variant::Dark).into_iter().map(|f| format!("mockup: {f}")));
        }
        assert!(scene.grab(&out().join(format!("{name}.png")).display().to_string()), "grab {name}");
        scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
        let walk = scene.walk(&tree);
        let walked = edges(&walk);
        let declared = palette(Variant::Dark);
        let header = Header {
            state: &name,
            case: screen,
            kind: "menu",
            width: w,
            height: h,
            measures: Measures::of(WalkForm::BigScreen, f64::from(h)),
            palette: &declared,
        };
        std::fs::write(out().join(format!("{name}.spec")), spec::write(&tree, &tree.root, &header, focus, &walk)).unwrap();
        faults.extend(
            scene
                .audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &walked, f64::from(w), f64::from(h))
                .iter()
                .map(|f| format!("gate: {f}")),
        );
        faults.retain(|f| !named_difference(f, &tree, (w, h)));
        let notes = out().join(format!("{name}.faults.txt"));
        if faults.is_empty() {
            let _ = std::fs::remove_file(&notes);
        } else {
            std::fs::write(&notes, format!("{}\n-- items:\n{}\n", faults.join("\n"), tree.legend())).unwrap();
            report.extend(faults.iter().map(|f| format!("{name}: {f}")));
        }
    }
    scene.resize(1280, 800);
    scene.wait(100);
}

fn mockup_of(name: &str) -> mockup::Mockup {
    let path = emusen_qt_test::lint::frontend().join("..").join("Testing").join("mockups").join(format!("{name}.mockup.spec"));
    mockup::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn each_screen_matches_its_mockup_and_passes_the_gate() {
    let _world = canvas_library();
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    assert_eq!(scene.property("library", "places"), "3");
    let mut report = Vec::new();

    wait_for(&mut scene, "continue", "activeFocus", "true", 1000);
    hold(&mut scene, "Home", &mockup_of("Home"), &mut report);

    let tree = scene.tree();
    let games = tree.find_named("place.0").expect("Home's Games place");
    scene.focus_item(&tree, games);
    scene.pad(UiButton::Accept);
    scene.wait(50);
    let consoles: usize = scene.tree().nodes.iter().filter(|n| n.visible && n.name.starts_with("consoleName.")).count();
    let last = named(&mut scene, &format!("tile.{}", 3 + consoles - 1)).expect("the last console's tile");
    let tree = scene.tree();
    scene.focus_item(&tree, last);
    let mut consoles_mockup = mockup_of("Consoles");
    consoles_mockup.elements.retain(|e| !COUNTED.iter().any(|c| e.name.starts_with(c)));
    hold(&mut scene, "Consoles", &consoles_mockup, &mut report);

    let snes = named(&mut scene, &format!("tile.{}", 3 + 1)).expect("the second console's tile");
    let tree = scene.tree();
    scene.focus_item(&tree, snes);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "gridCells", "count", &SNES_GAMES.to_string(), 1000);
    assert_eq!(scene.property("gridCells", "currentIndex"), "0", "the canvas focuses the first cover");
    assert_eq!(scene.property("gridCells", "activeFocus"), "true");
    hold(&mut scene, "Grid", &mockup_of("Grid"), &mut report);

    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}
