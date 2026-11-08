//! Stage 1's acceptance over the canvas's library (EmuSen_Qt.md §7.1 step 4, §8.2): each screen in the canvas's own focus,
//! held to its `.mockup.spec` at 1280 x 800 and 1920 x 1200, and to the kit's gate at its five sizes; each state is
//! recorded as a spec and grabbed into `EMUSEN_QT_COMPARE` (else the temporary folder) for the side-by-side pictures.

mod common;

use common::{sandbox, wait_for};

use emusen_client_fixture::library;
use emusen_crystal_client::Config;
use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt::MAIN_URL;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use serde_json::json;
use emusen_qt_test::harness::{edges, palette, Measures, SIZES};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::{mockup, Harness, Scene};
use std::path::{Path, PathBuf};

/// Choose a console's elements whose boxes follow from the canvas's eleven consoles; the fixture shelves three, so
/// these are held by kit-core's layout test against the canvas's own numbers (EmuSen_Qt.md §7.1 step 4).
const COUNTED: [&str; 6] = ["consoleTile.", "consoleBody.", "consoleName.", "consoleCount.", "consoleRing.", "orbit.1"];

/// A fault explained by a difference named in EmuSen_Qt.md §7.1 step 4: the help bar's opaque Surface, the orbit line in the
/// canvas's own ink, the canvas's 17- and 18-unit words under the floor at 1280 x 720 alone, the focused console's name in
/// the canvas's line box of 1, an engine's name too long for its tile or a place's line for its tile (cut as the accepted
/// render cuts it), and the fan's covers laid over each other.
fn named_difference(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    let item = |n: &str| n.trim().trim_end_matches(':').parse::<usize>().ok().and_then(|i| tree.nodes.get(i)).map_or("", |n| n.name.as_str());
    let after = |key: &str| fault.split(key).nth(1).map(|r| r.split_whitespace().next().unwrap_or(""));
    fault.contains("helpBar: alpha 1.000, the mockup's 0.92")
        || after("palette on ").is_some_and(|i| item(i).starts_with("orbit."))
        || (size == (1280, 720) && fault.contains("too small to read"))
        || after("cut at the foot: ").is_some_and(|i| item(i) == "consoleDetailName")
        || after("ellipsis: ").is_some_and(|i| item(i).starts_with("consoleEngineName.") || item(i).starts_with("placeStatus."))
        || fault.contains("overlap: ") && fault.split("overlap: ").nth(1).is_some_and(|r| r.split(" and ").all(|i| item(i).starts_with("fanCover.") || item(i) == "coverPicture"))
}

fn out() -> PathBuf {
    let dir = std::env::var_os("EMUSEN_QT_COMPARE").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The accepted render's library (Mistress's own-look fixture): eight games on three consoles, by title, each console's
/// extension and each game's index among the fixture's titles, which colours its cover.
const GAMES: [(&str, &str, usize); 8] = [
    ("Aurora Drift", "sfc", 0),
    ("Brass Lantern", "sfc", 1),
    ("Cobalt Harbor", "sfc", 2),
    ("Dune Relay", "sfc", 3),
    ("Ember Circuit", "sfc", 4),
    ("Fable of Tiles", "nes", 5),
    ("Granite Choir", "nes", 6),
    ("Hollow Comet", "gb", 7),
];

/// Each console's ES-DE folder and its box's shape, width by height: a landscape SNES box, a tall NES case, a near-square
/// Game Boy box (EmuSen_Interface.md §21).
const BOXES: [(&str, &str, u32, u32); 3] = [("sfc", "snes", 560, 400), ("nes", "nes", 420, 600), ("gb", "gb", 540, 600)];

/// The installed engines, as the sidecars an installation carries describe them: the name a choice stores, the info's own
/// name and its systems. Only the sidecars are written; nothing is loaded from them.
const ENGINES: [(&str, &str, &[&str]); 3] =
    [("MoonRT (Rust)", "MoonRT", &["nes"]), ("VenusRT (Rust)", "VenusRT", &["snes"]), ("MercuryRT (Rust)", "MercuryRT", &["gb", "gbc"])];

/// The tools' summaries the accepted render shows, as Mistress's fixture answers them (ToolCounts.Mockups).
const TOOLS: &str = r#"{"scraper":{"brief":"41 no art"},"packages":{"brief":"3 changed"},"library":{"brief":"2 suspect","attention":true},"cores":{"brief":"1 to approve","attention":true}}"#;

fn rom(world: &emusen_client_fixture::World, title: &str, ext: &str) -> PathBuf {
    world.dir("Roms").join(format!("{title} (Synthetic).{ext}"))
}

/// The covers as the fixture draws them, each a flat colour with a white rim and its words, grabbed from one Qt Quick
/// window resized to each box's shape (SyntheticLibrary.Label's drawing).
fn draw_covers(harness: &mut Harness, scratch: &Path, covers: &[(PathBuf, (u32, u32), usize, String)]) {
    let qml = scratch.join("cover.qml");
    std::fs::write(
        &qml,
        "import QtQuick\nWindow {\n  Rectangle { objectName: \"fill\"; anchors.fill: parent\n\
         Rectangle { x: 3; y: 3; width: parent.width - 6; height: parent.height - 6; color: \"transparent\"; border.color: \"white\"; border.width: 6 }\n\
         Text { objectName: \"words\"; anchors.centerIn: parent; width: parent.width - 20; horizontalAlignment: Text.AlignHCenter; color: \"white\"; font.pixelSize: Math.max(12, Math.floor(parent.height / 12)) } } }\n",
    )
    .unwrap();
    let mut scene = harness.load(&format!("file://{}", qml.display()), 64, 64);
    for (path, (w, h), index, words) in covers {
        // Green is the covers folder's: 60 + 6 * 23 % 160, the name's length in SyntheticLibrary.Colour.
        let (r, g, b) = (40 + 17 * index % 180, 198, 90 + 31 * index % 150);
        scene.resize(*w as i32, *h as i32);
        assert!(scene.set_property("fill", "color", &format!("#{r:02x}{g:02x}{b:02x}")));
        assert!(scene.set_property("words", "text", words));
        scene.wait(50);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        assert!(scene.grab(&path.display().to_string()), "cover {}", path.display());
    }
}

/// The library: the eight games in one folder, their covers in an ES-DE media folder at their boxes' shapes, the engines'
/// sidecars, each game played three hours apart with Aurora Drift the latest, and Brass Lantern and Fable of Tiles hearted.
fn canvas_library(harness: &mut Harness) -> (emusen_client_fixture::World, PathBuf) {
    let world = sandbox("qt-app-acceptance", &[]);
    let media = world.dir("Media");
    let mut covers = Vec::new();
    for (title, ext, index) in GAMES {
        std::fs::write(rom(&world, title, ext), [0u8; 4096]).unwrap();
        let (_, system, w, h) = BOXES.iter().find(|b| b.0 == ext).copied().unwrap();
        let cover = media.join(system).join("covers").join(format!("{title} (Synthetic).png"));
        covers.push((cover, (w, h), index, format!("{system} covers\n{title}\n{w}x{h}")));
    }
    draw_covers(harness, &world.root, &covers);
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let cores = world.dir("Cores");
    for (engine, name, systems) in ENGINES {
        let systems: Vec<_> = systems.iter().map(|s| json!({ "id": s, "name": s, "extensions": [], "regions": ["ntsc"], "controllers": [], "firmware": [] })).collect();
        let sidecar = json!({ "sidecar": 1, "library": format!("lib{}.so", name.to_lowercase()), "sha256": "00", "abi": "1.19", "capabilities": 0,
            "info": { "abi": "1.19", "id": name.to_lowercase(), "name": name, "display_name": engine, "version": "0.1.0", "license": "GPL-3.0-or-later",
                "authors": ["EmuSen"], "description": "", "systems": systems, "capabilities": [], "host_requires": [], "deterministic": true }, "settings": [] });
        std::fs::write(cores.join(format!("lib{}.so.core.json", name.to_lowercase())), sidecar.to_string()).unwrap();
    }
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    emusen_qt_models::library::LibrarySource::games(&source).unwrap();
    drop(source);
    let now = emusen_qt_models::words::now();
    for (i, (title, ext, _)) in GAMES.iter().enumerate() {
        play(&world, title, ext, now - 3 * 3600 * (i as i64 + 1));
    }
    let games = records();
    for (title, ext) in [("Brass Lantern", "sfc"), ("Fable of Tiles", "nes")] {
        games.change(&json!({ "op": "toggle_favourite", "path": rom(&world, title, ext).to_str().unwrap() })).unwrap();
    }
    (world, cores)
}

fn records() -> Library {
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

/// A game started at `at`, in seconds since 1970, through the library's own change.
fn play(world: &emusen_client_fixture::World, title: &str, ext: &str, at: i64) {
    let days = at.div_euclid(86_400);
    let secs = at.rem_euclid(86_400);
    // The civil date of a day count, Howard Hinnant's algorithm, as words::parse_utc reads it back.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    let stamp = format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", secs / 3600, secs % 3600 / 60, secs % 60);
    records().change(&json!({ "op": "started", "path": rom(world, title, ext).to_str().unwrap(), "at": stamp })).unwrap();
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
            let who = |f: &str| -> String {
                let ids = f.rsplit(": ").next().unwrap_or("").split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse::<usize>().ok());
                ids.filter_map(|i| tree.nodes.get(i)).map(|n| format!("{} {}", n.ty, n.name)).collect::<Vec<_>>().join(", ")
            };
            report.extend(faults.iter().map(|f| format!("{name}: {f} [{}]", who(f))));
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
    let mut harness = Harness::new();
    let (world, cores) = canvas_library(&mut harness);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("library", "toolSummaries", TOOLS));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    assert_eq!(scene.property("library", "places"), "3");
    let mut report = Vec::new();

    // Choose a console as the accepted render shows it: the first console focused.
    wait_for(&mut scene, "continue", "activeFocus", "true", 1000);
    let tree = scene.tree();
    let games = tree.find_named("place.0").expect("Home's Games place");
    scene.focus_item(&tree, games);
    scene.pad(UiButton::Accept);
    scene.wait(50);
    let first = named(&mut scene, "tile.3").expect("the first console's tile");
    let tree = scene.tree();
    scene.focus_item(&tree, first);
    // Each engine named by its own info less the port's mark, with its sky-named symbol (EmuSen_Interface.md §24.6).
    let tree = scene.tree();
    let node = |name: String| tree.find_named(&name).map(|i| &tree.nodes[i as usize]);
    for (_, name, _) in ENGINES {
        let shown = emusen_qt_models::library::without_port_mark(name);
        let tile = (0..ENGINES.len()).find(|t| node(format!("consoleEngineName.{t}")).and_then(|n| n.text.as_deref()) == Some(shown));
        let tile = tile.unwrap_or_else(|| panic!("{shown} on no tile"));
        assert_eq!(node(format!("consoleEngineMark.{tile}")).map(|n| n.heard.as_str()), Some(shown.to_lowercase().as_str()));
    }
    let mut consoles_mockup = mockup_of("Consoles");
    consoles_mockup.elements.retain(|e| !COUNTED.iter().any(|c| e.name.starts_with(c)));
    hold(&mut scene, "Consoles", &consoles_mockup, &mut report);

    // The grid on All games, its first cover focused.
    let all = named(&mut scene, "tile.0").expect("All games");
    let tree = scene.tree();
    scene.focus_item(&tree, all);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "gridCells", "count", &GAMES.len().to_string(), 1000);
    assert_eq!(scene.property("gridCells", "currentIndex"), "0", "the first cover is focused");
    assert_eq!(scene.property("gridCells", "activeFocus"), "true");
    // Every cover drawn from its picture in the media store, none as a generated card.
    let tree = scene.tree();
    let mut drawn: Vec<&str> = tree.nodes.iter().filter(|n| n.visible && n.name == "coverPicture").map(|n| n.heard.as_str()).collect();
    drawn.sort_unstable();
    let titles: Vec<String> = GAMES.iter().map(|(t, _, _)| format!("{t} (Synthetic)")).collect();
    assert_eq!(drawn, titles, "every cover read and drawn");
    hold(&mut scene, "Grid", &mockup_of("Grid"), &mut report);

    // Home with Cobalt Harbor played just now, read again as a new start reads it.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 1000);
    play(&world, "Cobalt Harbor", "sfc", emusen_qt_models::words::now());
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    scene.wait(100);
    wait_for(&mut scene, "library", "loading", "false", 20000);
    let tree = scene.tree();
    let continue_playing = tree.find_named("continue").expect("Continue");
    scene.focus_item(&tree, continue_playing);
    wait_for(&mut scene, "continue", "activeFocus", "true", 1000);
    let tree = scene.tree();
    let title = tree.find_named("continueTitle").and_then(|i| tree.nodes[i as usize].text.clone());
    assert_eq!(title.as_deref(), Some("Cobalt Harbor (Synthetic)"), "Continue follows the games read again");
    hold(&mut scene, "Home", &mockup_of("Home"), &mut report);

    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}
