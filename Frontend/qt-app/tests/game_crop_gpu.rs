//! A game's own crop in a running game (EmuSen_Qt.md §7.8.4), on Vulkan in a compositor of the test's own: two Super
//! Nintendo images the test makes, each drawing red with its left quarter forced black by the colour window, on VenusRT.
//! The console's crop is a custom 25% at the left, so the black quarter is cut; the first game's own crop set to Whole
//! picture brings the black quarter back on its next start, and the second game, with none of its own, keeps it cut.
//! A GPU job.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::cores::installation_with_snes;
use emusen_client_fixture::library;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, FILE_GRAPHICS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::display::Crop;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};
use serde_json::json;
use std::path::Path;

fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..300 {
        if done(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("{what} within 15 s");
}

/// A 128 KiB LoROM that sets the backdrop to red, forces the main screen black inside the colour window (x 0 to 63, the
/// left quarter of 256), turns the screen on and waits: `CGWSEL` 0x80 and `WOBJSEL` 0x20 (fullsnes, "Color Math").
fn quarter_black(title: &str) -> Vec<u8> {
    let mut rom = emusen_client_fixture::cores::snes_image(title);
    let code: &[u8] = &[
        0x78, 0x18, 0xFB, // SEI; CLC; XCE
        0x9C, 0x21, 0x21, // STZ CGADD
        0xA9, 0x1F, 0x8D, 0x22, 0x21, // red, low byte
        0x9C, 0x22, 0x21, // high byte
        0x9C, 0x2C, 0x21, 0x9C, 0x2D, 0x21, 0x9C, 0x31, 0x21, // no layers, no colour math
        0x9C, 0x26, 0x21, // WH0 = 0
        0xA9, 0x3F, 0x8D, 0x27, 0x21, // WH1 = 63
        0xA9, 0x20, 0x8D, 0x25, 0x21, // WOBJSEL: the colour window is window 1
        0xA9, 0x80, 0x8D, 0x30, 0x21, // CGWSEL: the main screen black inside it
        0xA9, 0x0F, 0x8D, 0x00, 0x21, // INIDISP: on, full brightness
        0x80, 0xFE, // BRA to itself
    ];
    rom[..code.len()].copy_from_slice(code);
    rom[0x7FDC..0x7FE0].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    let sum = rom.iter().fold(0u16, |s, &b| s.wrapping_add(u16::from(b)));
    rom[0x7FDC..0x7FDE].copy_from_slice(&(!sum).to_le_bytes());
    rom[0x7FDE..0x7FE0].copy_from_slice(&sum.to_le_bytes());
    rom
}

/// Sixteen points across the shown picture at three heights, left to right, each read dark (true) or not.
fn dark_columns(scene: &mut Scene<'_>) -> Vec<bool> {
    let tree = scene.tree();
    let shown = &tree.nodes[tree.find_named("shown").expect("the shown picture") as usize];
    let (x, y, w, h) = (shown.x, shown.y, shown.w, shown.h);
    let points: Vec<(f64, f64)> = (0..16).flat_map(|i| [0.3, 0.5, 0.7].map(|f| (x + w * (f64::from(i) + 0.5) / 16.0, y + h * f))).collect();
    let seen = scene.colours_at(&points);
    seen.chunks(3)
        .map(|c| {
            let c: Vec<_> = c.iter().map(|p| p.expect("a point read")).collect();
            let dark = c.iter().all(|p| u32::from(p.r) + u32::from(p.g) + u32::from(p.b) < 60);
            let red = c.iter().all(|p| p.r > 120 && p.g < 60 && p.b < 60);
            assert!(dark || red, "each column black or red: {c:?}");
            dark
        })
        .collect()
}

/// The game started by the game screen's `game`, its picture read once it is drawn, and the game closed.
fn played(scene: &mut Scene<'_>, game: &Path) -> Vec<bool> {
    assert!(scene.set_property("game", "game", game.to_str().unwrap()));
    wait_for(scene, "screens", "shown", "Game", 2000);
    until(scene, "the game running", |s| s.property("session", "running") == "true");
    let problem = scene.property("session", "problem");
    assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
    for _ in 0..300 {
        if scene.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0) {
            break;
        }
        scene.wait(50);
    }
    let first = scene.property("session", "firstPictureMs");
    assert!(first.parse::<f64>().is_ok_and(|ms| ms > 0.0), "the first picture within 15 s: {first}, running {}, {}", scene.property("session", "running"), common::state(scene));
    let mut last = Vec::new();
    // Steady: the same columns read three times running.
    let mut same = 0;
    until(scene, "the picture drawn and steady", |s| {
        let now = dark_columns(s);
        same = if now == last { same + 1 } else { 0 };
        last = now;
        same >= 3 && last.iter().any(|d| !d)
    });
    assert!(scene.set_property("game", "game", ""));
    until(scene, "the game closed", |s| s.property("session", "running") == "false");
    last
}

#[test]
fn a_games_own_crop_is_drawn_on_its_next_start_and_another_game_keeps_the_consoles() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let world = sandbox("qt-app-game-crop-gpu", &[]);
    // Each start from the beginning, never from where the game was left off.
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings[emusen_qt_models::resume::KEY] = json!("Restart");
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &world.dir("presets")).expect("a Vulkan device");
    let adoption = host.adoption();
    let cores = installation_with_snes(&world.dir("lib"));
    let first = world.dir("Roms/snes").join("Amber Isle.sfc");
    let second = world.dir("Roms/snes").join("Cobalt Harbor.sfc");
    std::fs::write(&first, quarter_black("AMBER ISLE")).unwrap();
    std::fs::write(&second, quarter_black("COBALT HARBOR")).unwrap();

    // The console's crop: the picture plain, 25% custom at the left, under the console's name as its session says it.
    let config = emusen_qt_models::clients::runtime_config(&platform, cores.to_str().unwrap()).unwrap();
    let rt = emusen_crystal_client::Runtime::new(&emusen_crystal_client::Api::load(library()).unwrap(), &config).unwrap();
    let console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the Super Nintendo").console;
    drop(rt);
    let graphics = json!({"Consoles": {console.as_str(): {"ScreenFilter": "effect.None.nearest", "Overscan": "custom", "CropLeft": "25"}}});
    let file = platform.file(FILE_GRAPHICS).unwrap();
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, file.file_name().unwrap().to_str().unwrap(), &graphics).unwrap();

    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);

    // The console's crop: the black quarter cut, every column red.
    let before = played(&mut scene, &first);
    eprintln!("the console's crop: {before:?}");
    assert!(before.iter().all(|d| !d), "the left quarter cut by the console's crop: {before:?}");

    // The first game's own crop, Whole picture, as Display crop stores it: on its next start the black quarter is drawn.
    let sqlite = Lent::open(emusen_qt_models::clients::SQLITE).unwrap();
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    games.change(&Crop::default().set_record(first.to_str().unwrap())).unwrap();
    let own = played(&mut scene, &first);
    eprintln!("the game's own crop: {own:?}");
    assert_eq!(own, [[true; 4].as_slice(), [false; 12].as_slice()].concat(), "the left quarter black again, the rest red");

    // The second game has no crop of its own: its black quarter stays cut.
    let other = played(&mut scene, &second);
    eprintln!("the other game: {other:?}");
    assert!(other.iter().all(|d| !d), "the other game keeps the console's crop: {other:?}");

    assert_eq!(host.feed.problem(), None, "the render thread met no failure");
    drop(scene);
    drop(harness);
}
