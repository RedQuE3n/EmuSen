//! Stage 1's acceptance over the canvas's library (EmuSen_Qt.md §7.1 steps 4 and 6, §8.2): each screen in the canvas's own
//! focus, held to its `.mockup.spec` at 1280 x 800 and 1920 x 1200, and to the kit's gate at its five sizes; each state is
//! recorded as a spec and grabbed into `EMUSEN_QT_COMPARE` (else the temporary folder) for the side-by-side pictures. The
//! program runs as a player's does: on Vulkan, its window adopting Serenity's device, in a compositor of the test's own on
//! the RX 6800 (EMUSEN_GPU_DEVICE names another), its pads an SDL virtual pad alone, and Home's paused game a real
//! session on the test core.

mod common;

use common::{sandbox, wait_for};

use emusen_client_fixture::{image, library};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt::MAIN_URL;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_play::host::{Adoption, Host};
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
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
/// render cuts it), the fan's covers laid over each other, and the focused shelf cover's name, whose glyph box the focus
/// ring's foot reaches at 1280 x 720 alone, where Mistress's render puts both (2 units under the shelf's covers).
fn named_difference(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    let item = |n: &str| n.trim().trim_end_matches(':').parse::<usize>().ok().and_then(|i| tree.nodes.get(i)).map_or("", |n| n.name.as_str());
    let after = |key: &str| fault.split(key).nth(1).map(|r| r.split_whitespace().next().unwrap_or(""));
    fault.contains("helpBar: alpha 1.000, the mockup's 0.92")
        || after("palette on ").is_some_and(|i| item(i).starts_with("orbit."))
        || (size == (1280, 720) && fault.contains("too small to read"))
        || after("cut at the foot: ").is_some_and(|i| item(i) == "consoleDetailName")
        || after("ellipsis: ").is_some_and(|i| item(i).starts_with("consoleEngineName.") || item(i).starts_with("placeStatus."))
        || fault.contains("overlap: ") && fault.split("overlap: ").nth(1).is_some_and(|r| r.split(" and ").all(|i| in_fan(tree, i)))
        || (size == (1280, 720) && after("low contrast: ").is_some_and(|i| item(i) == "shelfTitle"))
}

/// Whether the item numbered `n` is one of the fan's covers or inside one.
fn in_fan(tree: &emusen_qt_test::tree::Tree, n: &str) -> bool {
    let mut at = n.trim().trim_end_matches(':').parse::<usize>().ok().and_then(|i| tree.nodes.get(i));
    while let Some(node) = at {
        if node.name.starts_with("fanCover.") {
            return true;
        }
        at = tree.nodes.iter().find(|p| p.id == node.parent);
    }
    false
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
/// window resized to each box's shape (SyntheticLibrary.Label's drawing). The window adopts the program's device, and
/// is hidden once drawn: a second window's swapchain released only when the engine was dropped faulted in Qt's release.
fn draw_covers(harness: &mut Harness, adoption: &Adoption, scratch: &Path, covers: &[(PathBuf, (u32, u32), usize, String)]) {
    let qml = scratch.join("cover.qml");
    std::fs::write(
        &qml,
        "import QtQuick\nWindow {\n  objectName: \"covers\"\n  Rectangle { objectName: \"fill\"; anchors.fill: parent\n\
         Rectangle { x: 3; y: 3; width: parent.width - 6; height: parent.height - 6; color: \"transparent\"; border.color: \"white\"; border.width: 6 }\n\
         Text { objectName: \"words\"; anchors.centerIn: parent; width: parent.width - 20; horizontalAlignment: Text.AlignHCenter; color: \"white\"; font.pixelSize: Math.max(12, Math.floor(parent.height / 12)) } } }\n",
    )
    .unwrap();
    let mut scene = harness.load_prepared(&format!("file://{}", qml.display()), 64, 64, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
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
    // Hidden while its scene graph still holds its instance, so its swapchain is released before the window goes.
    assert!(scene.set_property("covers", "visible", "false"));
    scene.wait(50);
}

/// The library: the eight games in one folder, their covers in an ES-DE media folder at their boxes' shapes, the engines'
/// sidecars, each game played three hours apart with Aurora Drift the latest, and Brass Lantern and Fable of Tiles hearted.
fn canvas_library(harness: &mut Harness, adoption: &Adoption) -> (emusen_client_fixture::World, PathBuf) {
    let world = sandbox("qt-app-acceptance", &[]);
    let media = world.dir("Media");
    let mut covers = Vec::new();
    for (title, ext, index) in GAMES {
        std::fs::write(rom(&world, title, ext), [0u8; 4096]).unwrap();
        let (_, system, w, h) = BOXES.iter().find(|b| b.0 == ext).copied().unwrap();
        let cover = media.join(system).join("covers").join(format!("{title} (Synthetic).png"));
        covers.push((cover, (w, h), index, format!("{system} covers\n{title}\n{w}x{h}")));
    }
    // Aurora Drift's screenshot, the hero art Home's top shows for it with no game suspended (HomeNothingLeft).
    let shot = media.join("snes").join("screenshots").join("Aurora Drift (Synthetic).png");
    covers.push((shot, (512, 448), 0, "snes screenshots\nAurora Drift\n512x448".into()));
    draw_covers(harness, adoption, &world.root, &covers);
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
    let stamp = emusen_qt_models::words::utc(at);
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

/// The Pause Card's first page as Mistress's shows it over Cobalt Harbor: the title, the seven rows in her words, the help
/// bar, Resume focused; then held at the gate's sizes and recorded for the C# reader (PauseCard-*.spec).
fn pause_card(scene: &mut Scene<'_>, report: &mut Vec<String>) {
    assert_eq!(scene.property("pauseTitle", "text"), "Cobalt Harbor (Synthetic)");
    let tree = scene.tree();
    let words = |parent: &str, name: &str| tree.nodes.iter().find(|n| n.parent == parent && n.name == name).and_then(|n| n.text.clone()).unwrap_or_default();
    let mut rows: Vec<(f64, String, String)> =
        tree.nodes.iter().filter(|n| n.visible && n.ty.starts_with("MenuRow")).map(|n| (n.y, words(&n.id, "label"), words(&n.id, "value"))).collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let labels: Vec<&str> = rows.iter().map(|r| r.1.as_str()).collect();
    assert_eq!(labels, ["Resume", "Rewind", "Save State", "Load State", "Speed", "Game Options", "Leave"]);
    assert_eq!(rows[2].2, "Slot 1 (Empty)", "the session's first slot holds nothing");
    assert_eq!(rows[4].2, "100%");
    let focus = scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone());
    assert_eq!(focus.as_deref(), Some("Resume"), "the card opens on Resume");
    assert!(tree.nodes.iter().any(|n| n.visible && n.name == "help-Screenshot"), "the help bar names X");
    hold(scene, "PauseCard", &mockup::Mockup::default(), report);
}

fn mockup_of(name: &str) -> mockup::Mockup {
    let path = emusen_qt_test::lint::frontend().join("..").join("Testing").join("mockups").join(format!("{name}.mockup.spec"));
    mockup::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Waits up to ten seconds for a condition the scene answers, letting Qt run between looks.
fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        assert!(start.elapsed() < std::time::Duration::from_secs(10), "{what}: not within ten seconds");
        scene.wait(8);
    }
}

/// The text an item shows, from the tree, since a Repeater's items are not the window's QObject children.
fn text_of(scene: &mut Scene<'_>, name: &str) -> Option<String> {
    let tree = scene.tree();
    tree.find_named(name).and_then(|i| tree.nodes[i as usize].text.clone())
}

/// Whether a named item is shown.
fn shown(scene: &mut Scene<'_>, name: &str) -> bool {
    let tree = scene.tree();
    tree.find_named(name).is_some_and(|i| tree.nodes[i as usize].visible)
}

/// The test core in the player's cores folder, approved by its hash beside the installation's folder `cores`, as the
/// Cores window approves one.
fn approved_test_core(world: &emusen_client_fixture::World, cores: &Path) {
    let player = world.dir("Home/Cores");
    let core = world.player_core(&player);
    let config = Config { cores: Some(vec![cores.to_str().unwrap().into()]), player_cores: Some(vec![player.to_str().unwrap().into()]), ..Config::default() };
    let rt = Runtime::new(&Api::load(library()).unwrap(), &config).unwrap();
    rt.core_approve(&emusen_client_fixture::cores::sha256(&core)).unwrap_or_else(|e| panic!("{e}; the runtime lists {:?}", rt.cores()));
}

#[test]
fn each_screen_matches_its_mockup_and_passes_the_gate() {
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let presets = std::env::temp_dir().join("qt-app-acceptance-presets");
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &presets).expect("a Vulkan device");
    let adoption = host.adoption();
    let (world, cores) = canvas_library(&mut harness, &adoption);
    approved_test_core(&world, &cores);
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("library", "toolSummaries", TOOLS));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    assert_eq!(scene.property("library", "places"), "3");
    until(&mut scene, "the virtual pad read", |s| s.property("pads", "connected") == "1");
    let mut report = Vec::new();

    // Home with no game suspended, as Mistress's Home has it: no Continue, the shelf's first game at the top with its
    // hero art (Aurora Drift's screenshot), the focus on its cover.
    until(&mut scene, "the shelf's first cover focused", |s| {
        let tree = s.tree();
        s.focus(&tree).is_some_and(|i| tree.nodes[i as usize].name == "shelfCover.0")
    });
    assert!(text_of(&mut scene, "homeStatus").is_some_and(|t| t.starts_with("1 controller · ")), "the status line counts the virtual pad");
    assert!(!shown(&mut scene, "continueHeading") && !shown(&mut scene, "continuePanel"), "no Continue with no game suspended");
    assert_eq!(text_of(&mut scene, "continueTitle").as_deref(), Some("Aurora Drift (Synthetic)"));
    let hero = scene.property("continueArt", "source");
    assert!(hero.ends_with("screenshots/Aurora Drift (Synthetic).png"), "the top shows the hero art: {hero}");
    assert_eq!(text_of(&mut scene, "shelfTitle").as_deref(), Some("Aurora Drift (Synthetic)"), "the focused cover's name under the shelf");
    // Its corners rounded as a cover's: the top's picture fills its box at (64, 84), 373 by 280, so at 1280 x 800 the
    // box's very corner shows what is behind while a point within the radius shows the picture.
    let colours = scene.colours_at(&[(65.0, 85.0), (73.0, 93.0)]);
    let (corner, inside) = (colours[0].expect("the corner"), colours[1].expect("inside"));
    assert!(corner.g < 120 && inside.g > 150, "the top's picture rounded at its corner: corner {corner:?}, inside {inside:?}");
    hold(&mut scene, "HomeNothingLeft", &mockup::Mockup::default(), &mut report);

    // Choose a console as the accepted render shows it: the first console focused.
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

    // Cobalt Harbor played on the test core (the sandbox has no Super Nintendo core, so its file becomes the test core's
    // image, which no shelf of the library holds), then the menu chord: the Pause Card over the paused game.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 1000);
    std::fs::remove_file(rom(&world, "Cobalt Harbor", "sfc")).unwrap();
    let cobalt = rom(&world, "Cobalt Harbor", "tst");
    std::fs::write(&cobalt, image(&[1, 2, 3, 4])).unwrap();
    // The installation's own cores and the player's from here on, as a player's program reads them, so the session
    // finds the test core; the games are read again without Cobalt Harbor's old file.
    assert!(scene.set_property("library", "cores", ""));
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    scene.wait(100);
    wait_for(&mut scene, "library", "loading", "false", 20000);
    assert!(scene.set_property("game", "game", cobalt.to_str().unwrap()));
    wait_for(&mut scene, "screens", "shown", "Game", 1000);
    until(&mut scene, "the game running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true"
    });
    until(&mut scene, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    // The picture at the session's display shape, centred in the 16:10 window with bars, as Mistress letterboxes it.
    until(&mut scene, "the picture's shape", |s| s.property("session", "pictureAspect").parse::<f64>().is_ok_and(|a| a > 0.0));
    let aspect: f64 = scene.property("session", "pictureAspect").parse().unwrap();
    let tree = scene.tree();
    let placed = tree.nodes.iter().find(|n| n.name == "picture" && n.visible).map(|n| (n.x, n.y, n.w, n.h)).unwrap();
    let fitted = if aspect < 1.6 { ((1280.0 - 800.0 * aspect) / 2.0, 0.0, 800.0 * aspect, 800.0) } else { (0.0, (800.0 - 1280.0 / aspect) / 2.0, 1280.0, 1280.0 / aspect) };
    let near = |a: f64, b: f64| (a - b).abs() < 0.5;
    assert!(near(placed.0, fitted.0) && near(placed.1, fitted.1) && near(placed.2, fitted.2) && near(placed.3, fitted.3), "the picture at {placed:?}, its shape {aspect} fits at {fitted:?}");
    let guide = emusen_qt_endymion::reading::sdl_button(emusen_kit_core::input::PadControl::Guide).unwrap();
    let chord = |scene: &mut Scene<'_>| {
        assert!(scene.pad_chord_only(), "the game owns the pad");
        // The router's clock back on the wall clock, as a real pad needs it, since the harness's presses stopped it.
        scene.run_pad_clock();
        sdl.press(&pad, guide, true);
        until(scene, "the chord pausing the game under the Pause Card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
        sdl.press(&pad, guide, false);
        until(scene, "the card in place", |s| s.property("PadCard", "slide") == "0");
    };
    chord(&mut scene);
    pause_card(&mut scene, &mut report);
    scene.assert_still("the Pause Card over the paused game");

    // B resumes the game once the card has slid away; the chord opens it again, and Leave ▸ Back to Library is Home over
    // the paused game, as the render's.
    scene.pad(UiButton::Back);
    until(&mut scene, "B resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    assert_eq!(scene.property("cardCanvas", "visible"), "false", "the card gone");
    chord(&mut scene);
    for _ in 0..6 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pauseTitle", "text", "Leave", 1000);
    assert_eq!(scene.property("PadCard", "slide"), "0", "a page turn moves nothing");
    hold(&mut scene, "PauseCardLeave", &mockup::Mockup::default(), &mut report);
    scene.pad(UiButton::Accept);
    until(&mut scene, "Back to Library showing Home over the paused game", |s| s.property("screens", "shown") == "Home");
    assert_eq!(scene.property("session", "paused"), "true", "the game kept suspended");
    wait_for(&mut scene, "continue", "activeFocus", "true", 1000);
    assert_eq!(text_of(&mut scene, "continueTitle").as_deref(), Some("Cobalt Harbor (Synthetic)"), "Continue holds the suspended game");
    assert!(text_of(&mut scene, "continueFacts").is_some_and(|t| t == "Paused just now"), "Continue says when it was paused");
    assert!(shown(&mut scene, "continueFrame") && !shown(&mut scene, "continueArt"), "Continue shows the paused frame");
    assert!(shown(&mut scene, "continueHeading"));
    hold(&mut scene, "Home", &mockup_of("Home"), &mut report);
    scene.assert_still("Home over the suspended game");

    // Continue resumes the game; closed, Home has no Continue again and the shelf leads with it.
    scene.pad(UiButton::Accept);
    until(&mut scene, "Continue resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    let before = scene.frames();
    scene.wait(500);
    assert!(scene.frames() - before > 10, "the resumed game drawn at the display's rate: {} frames in 0.5 s", scene.frames() - before);
    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed and Home shown", |s| s.property("session", "running") == "false" && s.property("screens", "shown") == "Home");
    until(&mut scene, "Continue gone", |s| !shown(s, "continueHeading"));

    scene.assert_quiet();
    sdl.detach(pad);
    // Hidden while its scene graph still holds its instance, as the covers' window is.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
    assert!(report.is_empty(), "{}", report.join("\n"));
}
