//! Every Rust core plays a game in the program (EmuSen_Qt.md §8.5). The rows are the base's: each v1 core of the build's
//! list (`xtask/libraries.tsv`) and each system its sidecar offers, and one more for the libretro test core hosted as
//! the PlayStation's. For each row with a test image the client fixture makes itself (`play::image`), the game is started
//! from All Games by the pad, and the program is held to: its first picture, pictures arriving and drawn, sound taken in
//! by the player, the pad's South held reaching the core (the image's program draws another colour while it is held),
//! the chord's Pause Card holding the game still and Resume playing on, a state saved and loaded from the card, and Quit
//! Game back to the library. The pad is pressed ten seconds into a game, so a boot that reads a disc has ended. The
//! hosted row's game runs on the test core because the PlayStation's Engine setting names it, stored as the console's
//! settings page stores it, just before that row. A system the core marks in development, one the systems table lacks,
//! and one no image is made for are listed with the reason. One line per row is printed: passed, or the step that
//! failed and why; the test fails on any row that failed. On Vulkan in a compositor of the test's own; a GPU job.

mod common;

use std::path::{Path, PathBuf};

use common::console::{focused_words, picture_points, row_shown, until};
use common::{sandbox, wait_for};
use emusen_client_fixture::cores::{hosted, libretro_entry_as, sha256, RING_GAME};
use emusen_client_fixture::play::{image, installation_with_cores, v1_cores};
use emusen_client_fixture::library;
use emusen_crystal_client::{Api, Config, Runtime, Scope};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::LibrarySource;
use emusen_kit_core::input::{PadControl, UiButton};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::{Harness, Scene};

/// One row: the system, the core, the firmware its sidecar declares for the system, and the game, or why none is played.
struct Row {
    system: String,
    core: String,
    firmware: String,
    game: Result<PathBuf, String>,
}

/// The firmware a sidecar's system declares, each file as required or by what runs in its place.
fn firmware_words(system: &serde_json::Value) -> String {
    let files = system["firmware"].as_array().cloned().unwrap_or_default();
    if files.is_empty() {
        return "none".into();
    }
    let mut words: Vec<String> = files
        .iter()
        .map(|f| if f["required"].as_bool() == Some(true) { "required".to_owned() } else { format!("optional, replacement {}", f["replacement"]["effect"].as_str().unwrap_or("none")) })
        .collect();
    words.dedup();
    format!("{} file(s): {}", files.len(), words.join("; "))
}

/// The rows of the base's v1 cores, each image written into the world's `Roms/<system>` under a name that sorts it in
/// the rows' order. A system the core marks in development, or one the systems table lacks, has no shelf in the library.
fn rows(roms: &Path, table: &[String]) -> Vec<Row> {
    let mut rows = Vec::new();
    for core in v1_cores() {
        let sidecar: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(core.sidecar()).unwrap()).unwrap();
        for system in sidecar["info"]["systems"].as_array().expect("the sidecar's systems") {
            let id = system["id"].as_str().unwrap().to_owned();
            let game = if system["development"].as_bool() == Some(true) {
                Err("not offered: the core's information marks the system in development".to_owned())
            } else if !table.contains(&id) {
                Err("not offered: the systems table has no such system, so the library has no shelf for it".to_owned())
            } else {
                image(&id).map_err(|why| format!("no image: {why}")).map(|(ext, bytes)| {
                    let dir = roms.join(&id);
                    std::fs::create_dir_all(&dir).unwrap();
                    let path = dir.join(format!("Cores {:02} {id}{ext}", rows.len() + 1));
                    std::fs::write(&path, bytes).unwrap();
                    path
                })
            };
            rows.push(Row { system: id, core: core.core.clone(), firmware: firmware_words(system), game });
        }
    }
    rows
}

/// The average brightness (red, green and blue summed) the window shows over the game's picture.
fn brightness(scene: &mut Scene<'_>, points: &[(f64, f64)]) -> f64 {
    let seen: Vec<u32> = scene.colours_at(points).into_iter().flatten().map(|c| u32::from(c.r) + u32::from(c.g) + u32::from(c.b)).collect();
    seen.iter().sum::<u32>() as f64 / seen.len().max(1) as f64
}

fn number(scene: &mut Scene<'_>, property: &str) -> u64 {
    scene.property("session", property).parse().unwrap_or(0)
}

/// From Home: Games, All Games, the grid with its covers in place.
fn all_games(scene: &mut Scene<'_>) {
    until(scene, 5, "Home's Games", |s| {
        let tree = s.tree();
        tree.find_named("place.0").is_some_and(|games| s.focus(&tree) == Some(games) || s.focus_item(&tree, games))
    });
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Grid", 1000);
    until(scene, 5, "the grid's covers holding the focus", |s| s.property("gridCells", "count") != "0" && s.property("gridCells", "activeFocus") == "true");
}

/// A row's step that did not come to pass, in words. The compositor's watch ends the process on any panic, so a row's
/// steps say their failures instead of panicking, and the other rows still run.
type Step = Result<(), String>;

/// Waits up to `seconds` for a condition, letting Qt run between looks; the failure names what was waited for.
fn wait(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) -> Step {
    let start = std::time::Instant::now();
    while !done(scene) {
        if start.elapsed() >= std::time::Duration::from_secs(seconds) {
            return Err(format!("{what}: not within {seconds} s (shown {}, paused {}, problem {:?}, notice {:?})", scene.property("screens", "shown"), scene.property("session", "paused"), scene.property("session", "problem"), scene.property("session", "notice")));
        }
        scene.wait(8);
    }
    Ok(())
}

fn check(holds: bool, words: impl FnOnce() -> String) -> Step {
    if holds { Ok(()) } else { Err(words()) }
}

/// The pad moving the grid's focus to cover `index` (five covers a row).
fn cover(scene: &mut Scene<'_>, index: usize) -> Step {
    for _ in 0..40 {
        let at: usize = scene.property("gridCells", "currentIndex").parse().unwrap_or(0);
        if at == index {
            return Ok(());
        }
        let button = match (index / 5).cmp(&(at / 5)) {
            std::cmp::Ordering::Greater => UiButton::Down,
            std::cmp::Ordering::Less => UiButton::Up,
            _ if index > at => UiButton::Right,
            _ => UiButton::Left,
        };
        scene.pad(button);
        scene.wait(60);
    }
    Err(format!("the grid's focus did not reach cover {index}: at {}", scene.property("gridCells", "currentIndex")))
}

/// Down on the card until the focused row reads `words`, then A.
fn choose(scene: &mut Scene<'_>, words: &str) -> Step {
    for _ in 0..12 {
        if focused_words(scene).starts_with(words) {
            scene.pad(UiButton::Accept);
            return Ok(());
        }
        scene.pad(UiButton::Down);
    }
    Err(format!("no row of the card reads {words}; the focus reads {}", focused_words(scene)))
}

/// The chord's Pause Card over the running game: Guide alone, held until the router has read it, then let go.
fn pause_card(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) -> Step {
    wait(scene, 5, "the game owning the pad", |s| s.pad_chord_only())?;
    scene.run_pad_clock();
    let guide = emusen_qt_endymion::reading::sdl_button(PadControl::Guide).unwrap();
    sdl.press(pad, guide, true);
    let read = wait(scene, 5, "the router holding Guide", |s| s.pad_held(UiButton::Guide));
    sdl.press(pad, guide, false);
    read?;
    wait(scene, 10, "Guide pausing the game under the Pause Card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause")?;
    wait(scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0")
}

/// The steps of one row, each named in `step`; the first that fails ends the row.
fn play(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, index: usize, game: &Path, step: &mut &'static str) -> Step {
    *step = "start";
    cover(scene, index)?;
    scene.pad(UiButton::Accept);
    wait(scene, 20, "the game running", |s| (s.property("session", "running") == "true" && s.property("screens", "shown") == "Game") || !s.property("session", "problem").is_empty())?;
    let problem = scene.property("session", "problem");
    check(problem.is_empty(), || format!("the session's problem: {problem}"))?;
    let started = scene.property("session", "game");
    check(started == game.to_str().unwrap(), || format!("the game started is {started}, not the cover's"))?;

    *step = "first picture";
    wait(scene, 20, "the first picture", |s| number(s, "frame") > 0 && s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0))?;

    *step = "pictures";
    let points = picture_points(scene);
    let frame = number(scene, "frame");
    scene.wait(1000);
    let ran = number(scene, "frame") - frame;
    check(ran >= 30, || format!("{ran} frames ran in a second"))?;
    wait(scene, 10, "the picture drawn on the window, not black", |s| brightness(s, &points) > 60.0)?;
    let name = game.file_stem().unwrap().to_string_lossy().replace(' ', "-");
    scene.grab(&common::out().join(format!("{name}.png")).display().to_string());

    *step = "sound";
    let sounded = number(scene, "sounded");
    scene.wait(1000);
    let taken = number(scene, "sounded") - sounded;
    check(taken >= 8000, || format!("the sound player took {taken} frames of sound in a second"))?;

    *step = "pad";
    // Ten seconds into the game before the pad is pressed, so a boot that reads a disc has ended.
    wait(scene, 15, "ten seconds of the game run", |s| number(s, "frame") > 600)?;
    wait(scene, 5, "the game owning the pad", |s| s.pad_chord_only())?;
    scene.run_pad_clock();
    let south = emusen_qt_endymion::reading::sdl_button(PadControl::South).unwrap();
    let before = brightness(scene, &points);
    sdl.press(pad, south, true);
    let mut held = before;
    let reached = wait(scene, 5, &format!("South held reaching the core's picture (brightness {before:.0} before)"), |s| {
        held = brightness(s, &points);
        (held - before).abs() > 60.0
    });
    if reached.is_ok() {
        scene.grab(&common::out().join(format!("{name}-held.png")).display().to_string());
    }
    sdl.press(pad, south, false);
    reached?;
    wait(scene, 5, &format!("South let go in the core's picture (brightness {held:.0} held)"), |s| (brightness(s, &points) - before).abs() < 30.0)?;
    let (shown, paused) = (scene.property("screens", "shown"), scene.property("session", "paused"));
    check(shown == "Game" && paused == "false", || format!("South reached the program: shown {shown}, paused {paused}"))?;

    *step = "pause";
    pause_card(scene, sdl, pad)?;
    let still = number(scene, "frame");
    scene.wait(400);
    let now = number(scene, "frame");
    check(now == still, || format!("the game ran from frame {still} to {now} under the Pause Card"))?;
    choose(scene, "Resume")?;
    wait(scene, 5, "Resume playing on", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false" && number(s, "frame") > still + 10)?;

    *step = "save state";
    pause_card(scene, sdl, pad)?;
    choose(scene, "Save State")?;
    wait(scene, 10, "the save resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game")?;
    let notice = scene.property("session", "notice");
    check(notice == "State saved to slot 1", || format!("the save's notice: {notice}"))?;

    *step = "load state";
    scene.wait(500);
    pause_card(scene, sdl, pad)?;
    choose(scene, "Load State")?;
    wait(scene, 10, "the load resuming the game", |s| s.property("session", "paused") == "false" && s.property("screens", "shown") == "Game")?;
    let notice = scene.property("session", "notice");
    check(notice == "State loaded from slot 1", || format!("the load's notice: {notice}"))?;
    let loaded = number(scene, "frame");
    wait(scene, 5, "pictures after the load", |s| number(s, "frame") > loaded + 10)?;

    *step = "exit";
    pause_card(scene, sdl, pad)?;
    choose(scene, "Leave")?;
    wait(scene, 5, "the Leave page", |s| s.property("pause", "title") == "Leave" && row_shown(s, "Quit Game"))?;
    choose(scene, "Quit Game")?;
    wait(scene, 5, "Quit Game's question", |s| row_shown(s, "Yes"))?;
    choose(scene, "Yes")?;
    wait(scene, 10, "Quit Game closing the game to the library", |s| s.property("session", "running") == "false" && ["Grid", "Home"].contains(&s.property("screens", "shown").as_str()))
}

/// The hosted row's game, the last row's.
fn hosted_game(rows: &[Row]) -> &Path {
    rows.last().and_then(|r| r.game.as_ref().ok()).expect("the hosted row")
}

/// After a row that failed: the game closed, the cards gone, and the grid in place again.
fn recover(scene: &mut Scene<'_>) {
    scene.set_property("session", "game", "");
    for _ in 0..8 {
        let shown = scene.property("screens", "shown");
        if (shown == "Grid" || shown == "Home") && scene.property("cardCanvas", "visible") == "false" {
            return;
        }
        scene.pad(UiButton::Back);
        scene.wait(300);
    }
}

#[test]
fn every_rust_core_plays_a_game_in_the_program() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(1800);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    // SAFETY: set before any thread reads the environment; the proxy starts the runner it names.
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", &hosted().runner) };
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &std::env::temp_dir().join("qt-app-cores-play-presets")).expect("a Vulkan device");
    let adoption = host.adoption();

    let world = sandbox("qt-app-cores-play", &[]);
    let roms = world.dir("Roms");
    let table: Vec<String> = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap().systems().unwrap().into_iter().map(|s| s.id).collect();
    let mut rows = rows(&roms, &table);
    let cores = installation_with_cores(&world.dir("lib"));
    // The libretro test core, hosted as the PlayStation's, its game a disc image of the extensions it takes.
    let player = platform.directory(emusen_platform_client::galaxia::consts::DIR_CORES).unwrap().expect("the player's cores folder");
    let entry = libretro_entry_as(&player, "playstation");
    let ring = roms.join("psx").join(format!("Cores {:02} hosted.iso", rows.len() + 1));
    std::fs::write(&ring, RING_GAME).unwrap();
    rows.push(Row { system: "psx".into(), core: "the libretro test core (hosted)".into(), firmware: "none".into(), game: Ok(ring) });
    let runtime = || {
        let config = emusen_qt_models::clients::runtime_config(&platform, cores.to_str().unwrap()).unwrap();
        Runtime::new(&Api::load(library()).unwrap(), &config).unwrap()
    };
    runtime().core_approve(&sha256(&entry)).unwrap();

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 20, "the virtual pad read", |s| s.property("pads", "connected") == "1");
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    all_games(&mut scene);

    let mut lines = Vec::new();
    let mut cover_index = 0;
    for row in &rows {
        let head = format!("{:<7} {:<34} firmware: {}", row.system, row.core, row.firmware);
        let game = match &row.game {
            Ok(game) => game,
            Err(why) => {
                lines.push(format!("{head}: {why}"));
                continue;
            }
        };
        if game == hosted_game(&rows) {
            // The PlayStation's engine chosen as the test core, as Settings' console page chooses it, for its game alone.
            let rt = runtime();
            let engine = rt.cores().unwrap().into_iter().find(|c| c.library.as_deref().is_some_and(|l| l.contains("testcore"))).and_then(|c| c.engine).expect("the test core listed");
            let console = rt.route(game.to_str().unwrap()).unwrap().console.expect("the game's console");
            rt.settings_set(0, Scope::Console, Some(&console), "Engine", Some(&engine)).unwrap();
        }
        let mut step = "start";
        let played = play(&mut scene, &sdl, &pad, cover_index, game, &mut step);
        cover_index += 1;
        match played {
            Ok(()) => lines.push(format!("{head}: passed")),
            Err(said) => {
                lines.push(format!("{head}: failed at {step}: {said}"));
                recover(&mut scene);
            }
        }
        if scene.property("screens", "shown") == "Home" {
            all_games(&mut scene);
        }
        eprintln!("{}", lines.last().unwrap());
    }
    println!("cores_play:\n{}", lines.join("\n"));
    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
    let failed: Vec<&String> = lines.iter().filter(|l| l.contains(": failed at ")).collect();
    assert!(failed.is_empty(), "{} of {} rows failed:\n{}", failed.len(), lines.len(), failed.iter().map(|l| l.as_str()).collect::<Vec<_>>().join("\n"));
}
