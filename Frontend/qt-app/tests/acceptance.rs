//! Stage 1's acceptance over the canvas's library (EmuSen_Qt.md §7.1 steps 4 and 6, §8.2): each screen in the canvas's own
//! focus, held to its `.mockup.spec` at 1280 x 800 and 1920 x 1200, and to the kit's gate at its five sizes; each state is
//! recorded as a spec and grabbed into `EMUSEN_QT_COMPARE` (else the temporary folder) for the side-by-side pictures. The
//! program runs as a player's does: on Vulkan, its window adopting Serenity's device, in a compositor of the test's own on
//! the RX 6800 (EMUSEN_GPU_DEVICE names another), its pads an SDL virtual pad alone, and Home's paused game a real
//! session on the test core.

mod common;

use common::{draw_covers, hold, sandbox, wait_for};

use emusen_client_fixture::{image, library};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::words;
use emusen_qt_play::host::{Adoption, Host};
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use serde_json::json;
use emusen_qt_test::{mockup, Harness, Scene};
use std::path::{Path, PathBuf};

/// Choose a console's elements whose boxes follow from the canvas's eleven consoles on three makers' orbits; the fixture
/// shelves three consoles of one maker, so these are held by kit-core's orbits test against the canvas's own numbers
/// (EmuSen_Qt.md §3.4.2).
const COUNTED: [&str; 7] = ["tile.", "consoleBody.", "consoleName.", "consoleCount.", "consoleRing.", "rowCover.", "makerLabel."];

/// A fault explained by a difference named in EmuSen_Qt.md §7.1 step 4 and §3.4.2: the orbit line and the places' glass
/// edge in the canvas's own ink, a place's line built from data (a fixed word such as "Game packages" may not elide, as
/// theme-audit holds it), and the focused shelf cover's name, whose glyph box the focus
/// ring's foot reaches at 1280 x 720 alone, where Mistress's render puts both (2 units under the shelf's covers).
fn named_difference(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    let node = |n: &str| n.trim().trim_end_matches(':').parse::<usize>().ok().and_then(|i| tree.nodes.get(i));
    let item = |n: &str| node(n).map_or("", |n| n.name.as_str());
    let said = |n: &str| node(n).and_then(|n| n.text.as_deref()).unwrap_or("");
    let after = |key: &str| fault.split(key).nth(1).map(|r| r.split_whitespace().next().unwrap_or(""));
    after("palette on ").is_some_and(|i| item(i).starts_with("orbit.") || item(i).starts_with("placeDisc."))
        || after("ellipsis: ").is_some_and(|i| item(i).starts_with("placeStatus.") && !words::is_fixed(said(i)))
        || (size == (1280, 720) && after("low contrast: ").is_some_and(|i| item(i) == "shelfTitle"))
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
    draw_covers(harness, &world.root, &covers, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
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

/// The Pause Card's first page as Mistress's shows it over Cobalt Harbor: the title, the seven rows in her words, the help
/// bar, Resume focused; then held at the gate's sizes and recorded for the C# reader (PauseCard-*.spec).
fn pause_card(scene: &mut Scene<'_>, report: &mut Vec<String>) {
    assert_eq!(scene.property("pause", "title"), "Cobalt Harbor (Synthetic)");
    let tree = scene.tree();
    let words = |parent: &str, name: &str| tree.nodes.iter().find(|n| n.parent == parent && n.name == name).and_then(|n| n.text.clone()).unwrap_or_default();
    let mut rows: Vec<(f64, String, String)> =
        tree.nodes.iter().filter(|n| n.visible && n.ty.starts_with("MenuRow")).map(|n| (n.y, words(&n.id, "label"), words(&n.id, "value"))).collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let labels: Vec<&str> = rows.iter().map(|r| r.1.as_str()).collect();
    // The test core claims a drive (MEDIA with a drive named), so Change Disc stands after Speed, as Mistress's rule has it.
    assert_eq!(labels, ["Resume", "Rewind", "Save State", "Load State", "Speed", "Change Disc", "Game Options", "Leave"]);
    assert_eq!(rows[2].2, "Slot 1 (Empty)", "the session's first slot holds nothing");
    assert_eq!(rows[4].2, "100%");
    let focus = scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone());
    assert_eq!(focus.as_deref(), Some("Resume"), "the card opens on Resume");
    assert!(tree.nodes.iter().any(|n| n.visible && n.name == "help-Screenshot"), "the help bar names X");
    hold(scene, "PauseCard", &mockup::Mockup::default(), named_difference, report);
}

/// Whether the first shown picture of that name is drawn: its centre in another colour than the pane's own surface,
/// sampled at the pane's foot (a Repeater's items are found in the tree, not by name).
fn drawn(scene: &mut Scene<'_>, name: &str) -> bool {
    let tree = scene.tree();
    let Some(n) = tree.nodes.iter().find(|n| n.visible && n.name == name) else { return false };
    let c = scene.colours_at(&[(n.x + n.w / 2.0, n.y + n.h / 2.0), (590.0, 670.0)]);
    c[0].is_some() && c[0] != c[1]
}

/// Rewind, Save State and Load State over the real session (EmuSen_Qt.md §7.3): the reel's newest pictures served by
/// address, a slot saved from the card with its picture beside it and loaded, and the reel's page going back to a moment;
/// each page held at the gate's sizes and recorded (PauseCardRewind, PauseCardSave, PauseCardReel).
fn pages(scene: &mut Scene<'_>, report: &mut Vec<String>, chord: &dyn Fn(&mut Scene<'_>)) {
    // Half a second into the game the reel holds no picture yet, as in Mistress's render of the row.
    scene.pad(UiButton::Down);
    wait_for(scene, "PaneSentence", "text", "Under a second available. A opens the reel; Left and Right step through it.", 1000);
    hold(scene, "PauseCardRewind", &mockup::Mockup::default(), named_difference, report);
    scene.pad(UiButton::Down);
    wait_for(scene, "PaneSentence", "text", "A saves to Slot 1.", 1000);
    hold(scene, "PauseCardSave", &mockup::Mockup::default(), named_difference, report);

    // A saves to slot 1 and closes the card; the game resumes once the state is written, and plays on for the reel.
    scene.pad(UiButton::Accept);
    until(scene, "the save resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    assert_eq!(scene.property("session", "notice"), "State saved to slot 1");
    // The notice over the gameplay, at the top right, with its shadow beneath it (EmuSen_Interface.md §5.4, §18.6): the
    // game's picture 8 pixels under the card darker while the notice shows than once it has gone.
    wait_for(scene, "notices", "shown", "true", 1000);
    scene.wait(400);
    let tree = scene.tree();
    let card = &tree.nodes[tree.find_named("notice").expect("the notice") as usize];
    let under = (card.x + card.w / 2.0, card.y + card.h + 8.0);
    assert!(card.x + card.w > 1100.0 && card.y < 120.0, "at the top right: {:?}", (card.x, card.y, card.w, card.h));
    let shaded = scene.colours_at(&[under])[0].expect("under the notice");
    scene.wait(2500);
    assert_eq!(scene.property("notices", "shown"), "false");
    let bare = scene.colours_at(&[under])[0].expect("under the notice");
    let sum = |c: emusen_kit_core::tokens::Colour| u32::from(c.r) + u32::from(c.g) + u32::from(c.b);
    assert!(sum(shaded) + 30 < sum(bare), "the notice's shadow: {shaded:?} under it, {bare:?} once gone");
    chord(scene);
    scene.pad(UiButton::Down);
    until(scene, "the reel's pictures served", |s| drawn(s, "PanePicture"));
    hold(scene, "PauseCardRewindHeld", &mockup::Mockup::default(), named_difference, report);
    scene.pad(UiButton::Down);
    wait_for(scene, "PaneSentence", "text", "A saves to Slot 1, replacing it.", 1000);
    until(scene, "the slot's picture beside its state", |s| drawn(s, "PanePicture"));
    hold(scene, "PauseCardSaved", &mockup::Mockup::default(), named_difference, report);
    scene.pad(UiButton::Down);
    wait_for(scene, "PaneSentence", "text", "A loads Slot 1.", 1000);
    scene.pad(UiButton::Accept);
    until(scene, "the load resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    assert_eq!(scene.property("session", "notice"), "State loaded from slot 1");
    scene.wait(2500);

    // The reel's page on its first stop, its moment drawn large; A goes back there and the game plays on from it.
    chord(scene);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(scene, "pause", "title", "Rewind", 1000);
    assert_eq!(scene.property("cardCanvas", "slide"), "0", "a page turn moves nothing");
    until(scene, "the moment's picture", |s| drawn(s, "PaneMomentPicture"));
    hold(scene, "PauseCardReel", &mockup::Mockup::default(), named_difference, report);
    scene.pad(UiButton::Accept);
    until(scene, "the rewind resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    assert_eq!(scene.property("session", "notice"), "Rewound");
    chord(scene);

    // Speed: Right gives the running game turbo through the runtime, read back into the card's row; Left restores it.
    for _ in 0..4 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Right);
    until(scene, "turbo read back from the runtime", |s| s.property("pause", "facts").contains("\"speed\":300"));
    let problem = scene.property("session", "problem");
    assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the runtime took the speed: {problem}");
    scene.pad(UiButton::Left);
    until(scene, "the game's own speed again", |s| s.property("pause", "facts").contains("\"speed\":100"));

    // X saves the picture on show as a screenshot, and the card stays.
    scene.pad(UiButton::Screensaver);
    until(scene, "the screenshot saved", |s| s.property("session", "notice") == "Screenshot saved");
    // Its notice shown over the card, then gone (2 s, then Exit), so the pages held next are the card's alone.
    wait_for(scene, "notices", "shown", "true", 1000);
    wait_for(scene, "notices", "shown", "false", 3000);
    // shown turns false as the fade starts; the card draws until its opacity reaches 0.
    wait_for(scene, "notice", "visible", "false", 1000);
    scene.wait(400);
    assert_eq!(scene.property("cardCanvas", "slide"), "0", "the card stays");
}

/// The card's shadow beside its panel over the help band, as Mistress's falls (her render at 1280 x 800, 2026-11-02: 15 of
/// Surface's 27 at the panel's edge, 25 at 16 pixels out, Surface again by 24): darker than Surface at the edge and back
/// to it within 32 pixels.
fn shadow_over_band(scene: &mut Scene<'_>) {
    let profile: Vec<u8> = scene.colours_at(&(0..12).map(|i| (512.0 + 4.0 * f64::from(i), 760.0)).collect::<Vec<_>>()).iter().map(|c| c.map_or(255, |c| c.r)).collect();
    eprintln!("the shadow over the band, every 4 pixels from the panel's edge: {profile:?}");
    let at = scene.colours_at(&[(513.0, 760.0), (529.0, 760.0), (548.0, 760.0)]);
    let level = |c: &Option<emusen_kit_core::tokens::Colour>| c.map_or(255, |c| c.r);
    let (edge, mid, out) = (level(&at[0]), level(&at[1]), level(&at[2]));
    assert!(edge < 22 && edge < mid && mid <= out && out >= 26, "the shadow over the band: {edge}, {mid}, {out} at 1, 17 and 36 pixels out");
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
    // A hang detector, not a gate: the run's own length grew past 600 s under load (EmuSen_Qt.md §7.8).
    kwin.watch(1200);
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
    // The clock held at one instant, so no state depends on the minute it is read in: a paused game's "Paused N minutes
    // ago" turning over under Home once failed assert_still (EmuSen_Qt.md §7.8).
    assert!(scene.set_property("clock", "now", common::HELD_NOW));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("library", "toolSummaries", TOOLS));
    // The folder the theme's covers may come from, as the program's start reads it from the settings.
    assert!(scene.set_property("program", "coverStores", world.dir("Media").to_str().unwrap()));
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
    // The top reads its own GameModel, filled off the GUI thread apart from the shelf's, so it may name the game a little after the focus.
    until(&mut scene, "the top named the shelf's first game", |s| text_of(s, "continueTitle").as_deref() == Some("Aurora Drift (Synthetic)"));
    let hero = scene.property("continueArt", "source");
    assert!(hero.ends_with("screenshots/Aurora Drift (Synthetic).png"), "the top shows the hero art: {hero}");
    assert_eq!(text_of(&mut scene, "shelfTitle").as_deref(), Some("Aurora Drift (Synthetic)"), "the focused cover's name under the shelf");
    // Its corners rounded as a cover's: the top's picture fills its box at (64, 84), 373 by 280, so at 1280 x 800 the
    // box's very corner shows what is behind while a point within the radius shows the picture.
    let colours = scene.colours_at(&[(65.0, 85.0), (73.0, 93.0)]);
    let (corner, inside) = (colours[0].expect("the corner"), colours[1].expect("inside"));
    assert!(corner.g < 120 && inside.g > 150, "the top's picture rounded at its corner: corner {corner:?}, inside {inside:?}");
    hold(&mut scene, "HomeNothingLeft", &mockup::Mockup::default(), named_difference, &mut report);

    // Choose a console as the accepted render shows it: the first console focused.
    let tree = scene.tree();
    let games = tree.find_named("place.0").expect("Home's Games place");
    scene.focus_item(&tree, games);
    scene.pad(UiButton::Accept);
    scene.wait(50);
    let first = named(&mut scene, "tile.3").expect("the first console's tile");
    let tree = scene.tree();
    scene.focus_item(&tree, first);
    // Each engine's sky-named symbol after its console's count, and its name, less the port's mark, in the foot's engine
    // line when its console has the focus (EmuSen_Interface.md §24.6).
    for (_, name, _) in ENGINES {
        let shown = emusen_qt_models::library::without_port_mark(name);
        let tree = scene.tree();
        let node = |name: String| tree.find_named(&name).map(|i| &tree.nodes[i as usize]);
        let world = (0..ENGINES.len()).find(|t| node(format!("consoleEngineMark.{t}")).is_some_and(|n| n.heard == shown.to_lowercase()));
        let world = world.unwrap_or_else(|| panic!("{shown}'s mark on no world"));
        let item = tree.find_named(&format!("tile.{}", world + 3)).expect("its world");
        scene.focus_item(&tree, item);
        scene.wait(50);
        let tree = scene.tree();
        let engine = tree.nodes.iter().find(|n| n.name == "engineName" && n.visible).and_then(|n| n.text.clone());
        assert_eq!(engine.as_deref(), Some(shown), "the foot names the focused console's engine");
    }
    let tree = scene.tree();
    scene.focus_item(&tree, first);
    let mut consoles_mockup = mockup_of("Consoles");
    consoles_mockup.elements.retain(|e| !COUNTED.iter().any(|c| e.name.starts_with(c)));
    hold(&mut scene, "Consoles", &consoles_mockup, named_difference, &mut report);

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
    hold(&mut scene, "Grid", &mockup_of("Grid"), named_difference, &mut report);

    // Select: the List card over the grid, as Mistress's GamesListCard over the same eight games, its shadow falling over
    // the help band beside the panel (EmuSen_Qt.md §7.1, stage 2).
    scene.pad(UiButton::Options);
    until(&mut scene, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    shadow_over_band(&mut scene);
    hold(&mut scene, "GamesListCard", &mockup::Mockup::default(), named_difference, &mut report);
    scene.assert_still("the List card in place");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);

    // X: the first game's page, its cover casting the raised shadow on Vulkan (EmuSen_Kit.md §7.55): the frame 6 units
    // under the cover's foot an eighth darker than 40 units beside it; B back to the grid.
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    scene.wait(400);
    let tree = scene.tree();
    let cover = &tree.nodes[tree.find_named("pageCover").expect("the page's cover") as usize];
    let foot = cover.y + cover.h + 6.0 * cover.scale;
    let colours = scene.colours_at(&[(cover.x + cover.w / 2.0, foot), (cover.x - 40.0 * cover.scale, foot)]);
    let (under, beside) = (colours[0].expect("under the cover"), colours[1].expect("beside the cover"));
    let sum = |c: emusen_kit_core::tokens::Colour| u32::from(c.r) + u32::from(c.g) + u32::from(c.b);
    eprintln!("the cover's shadow on Vulkan: {under:?} under it, {beside:?} beside it");
    assert!(sum(under) * 8 <= sum(beside) * 7, "the cover's shadow: {under:?} under it, {beside:?} beside it");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);

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
        until(scene, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    };
    chord(&mut scene);
    pause_card(&mut scene, &mut report);
    scene.assert_still("the Pause Card over the paused game");
    pages(&mut scene, &mut report, &chord);

    // Change Disc from the Speed row the pages left, over the test core's drive: the game's own disc first, marked
    // Inserted, its pane the generated card beside its name (PauseCardChangeDisc); B back to its row.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pause", "title", "Change Disc", 1000);
    assert_eq!(scene.property("cardCanvas", "slide"), "0", "a page turn moves nothing");
    assert_eq!(text_of(&mut scene, "PaneTitle").as_deref(), Some("Cobalt Harbor (Synthetic)"));
    hold(&mut scene, "PauseCardChangeDisc", &mockup::Mockup::default(), named_difference, &mut report);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pause", "current", "5", 1000);
    // Game Options: Mistress's four rows and Memory Cards for a console with a drive (PauseCardGameOptions), B back to its row.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pause", "title", "Game Options", 1000);
    hold(&mut scene, "PauseCardGameOptions", &mockup::Mockup::default(), named_difference, &mut report);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pause", "current", "6", 1000);

    // B resumes the game once the card has slid away; the chord opens it again, and Leave ▸ Back to Library is Home over
    // the paused game, as the render's.
    scene.pad(UiButton::Back);
    until(&mut scene, "B resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    assert_eq!(scene.property("cardCanvas", "visible"), "false", "the card gone");

    // The Guide chords on, over the running game: Guide with Up saves to the slot through the session and opens no menu;
    // Guide let go alone opens the card (EmuSen_Kit.md §7.26).
    assert!(scene.set_property("program", "guideChords", "true"));
    let up = emusen_qt_endymion::reading::sdl_button(emusen_kit_core::input::PadControl::Up).unwrap();
    scene.run_pad_clock();
    sdl.press(&pad, guide, true);
    scene.wait(100);
    sdl.press(&pad, up, true);
    until(&mut scene, "Guide with Up saving", |s| s.property("session", "notice") == "State saved to slot 1");
    sdl.press(&pad, up, false);
    sdl.press(&pad, guide, false);
    scene.wait(300);
    assert_eq!((scene.property("screens", "shown"), scene.property("session", "paused")), ("Game".into(), "false".into()), "a chord opens no menu");
    sdl.press(&pad, guide, true);
    scene.wait(300);
    assert_eq!(scene.property("screens", "shown"), "Game", "Guide going down opens nothing yet");
    sdl.press(&pad, guide, false);
    until(&mut scene, "Guide let go alone opening the card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    until(&mut scene, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    assert!(scene.set_property("program", "guideChords", "false"));
    for _ in 0..7 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pause", "title", "Leave", 1000);
    assert_eq!(scene.property("cardCanvas", "slide"), "0", "a page turn moves nothing");
    // The chord's "State saved" notice gone, so the page held is the card's alone.
    wait_for(&mut scene, "notices", "shown", "false", 3000);
    wait_for(&mut scene, "notice", "visible", "false", 1000);
    hold(&mut scene, "PauseCardLeave", &mockup::Mockup::default(), named_difference, &mut report);
    // Quit Game asks first, on the card form's question page, No first; B goes back to its row and Up to Back to Library.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pause", "title", "Quit Game?", 1000);
    hold(&mut scene, "PauseCardQuitGame", &mockup::Mockup::default(), named_difference, &mut report);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pause", "title", "Leave", 1000);
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Accept);
    until(&mut scene, "Back to Library showing Home over the paused game", |s| s.property("screens", "shown") == "Home");
    assert_eq!(scene.property("session", "paused"), "true", "the game kept suspended");
    wait_for(&mut scene, "continue", "activeFocus", "true", 1000);
    assert_eq!(text_of(&mut scene, "continueTitle").as_deref(), Some("Cobalt Harbor (Synthetic)"), "Continue holds the suspended game");
    assert!(text_of(&mut scene, "continueFacts").is_some_and(|t| t == "Paused just now"), "Continue says when it was paused");
    assert!(shown(&mut scene, "continueFrame") && !shown(&mut scene, "continueArt"), "Continue shows the paused frame");
    assert!(shown(&mut scene, "continueHeading"));
    // The screenshot's notice from the card gone, so Home is still (it holds 2 s, then fades over Exit).
    wait_for(&mut scene, "notices", "shown", "false", 3000);
    wait_for(&mut scene, "notice", "visible", "false", 1000);
    scene.wait(400);
    hold(&mut scene, "Home", &mockup_of("Home"), named_difference, &mut report);
    scene.assert_still("Home over the suspended game");

    // Continue resumes the game; closed, Home has no Continue again and the shelf leads with it.
    scene.pad(UiButton::Accept);
    until(&mut scene, "Continue resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    let before = scene.frames();
    scene.wait(500);
    assert!(scene.frames() - before > 10, "the resumed game drawn at the display's rate: {} frames in 0.5 s", scene.frames() - before);
    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed and Home shown", |s| s.property("session", "running") == "false" && s.property("screens", "shown") == "Home");

    // Closed, the game was left off: the runtime holds its resume state, and its next start asks the Resume question
    // over the game held unplayed (EmuSen_Interface.md §18.3), Continue focused; B goes back to Home with the game closed.
    assert!(scene.set_property("game", "game", cobalt.to_str().unwrap()));
    wait_for(&mut scene, "session", "resumeAsked", "true", 5000);
    scene.wait(450);
    assert_eq!(text_of(&mut scene, "questionTitle").as_deref(), Some("Continue where you left off?"));
    assert!(text_of(&mut scene, "questionCaption").is_some_and(|t| t.contains("Resume ◂ Ask each time ▸")));
    assert_eq!(scene.property("session", "firstPictureMs"), "0", "the game held unplayed behind the question");
    until(&mut scene, "Continue focused", |s| {
        let tree = s.tree();
        s.focus(&tree).is_some_and(|i| tree.nodes[i as usize].name == "answer.0")
    });
    hold(&mut scene, "ResumeQuestion", &mockup::Mockup::default(), named_difference, &mut report);
    scene.pad(UiButton::Back);
    until(&mut scene, "B closing the held game", |s| s.property("session", "running") == "false" && s.property("screens", "shown") == "Home");
    // B let the game go unplayed, so choosing it again opens it again and asks again; Continue loads where it was left
    // off and the game plays.
    assert!(scene.set_property("game", "game", cobalt.to_str().unwrap()));
    wait_for(&mut scene, "session", "resumeAsked", "true", 5000);
    scene.wait(450);
    scene.pad(UiButton::Accept);
    until(&mut scene, "Continue playing the game", |s| s.property("session", "resumeAsked") == "false" && s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game");
    let before = scene.frames();
    scene.wait(500);
    assert!(scene.frames() - before > 10, "the resumed game drawn: {} frames in 0.5 s", scene.frames() - before);
    assert!(!scene.property("session", "notice").starts_with("Could not"), "{}", scene.property("session", "notice"));
    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed again", |s| s.property("session", "running") == "false" && s.property("screens", "shown") == "Home");
    until(&mut scene, "Continue gone", |s| !shown(s, "continueHeading"));

    // Power: the Home card at its Power page, then Quit EmuSen...'s question, each held as Mistress's HomeCardPower and
    // HomeCardQuit; B twice closes it.
    let power = named(&mut scene, "place.6").expect("the Power place");
    let tree = scene.tree();
    scene.focus_item(&tree, power);
    scene.pad(UiButton::Accept);
    until(&mut scene, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    shadow_over_band(&mut scene);
    hold(&mut scene, "HomeCardPower", &mockup::Mockup::default(), named_difference, &mut report);
    scene.assert_still("the Home card in place");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "homeCardModel", "page", "quit", 1000);
    hold(&mut scene, "HomeCardQuit", &mockup::Mockup::default(), named_difference, &mut report);
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    until(&mut scene, "the Home card closed", |s| s.property("screens", "shown") == "Home");

    scene.assert_quiet();
    sdl.detach(pad);
    // Hidden while its scene graph still holds its instance, as the covers' window is.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
    assert!(report.is_empty(), "{}", report.join("\n"));
}
