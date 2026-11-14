//! The Pause Card's Game Options ▸ Display and Sound in the program on Vulkan (settings_card.rs): the client fixture's
//! console core, a PlayStation started empty and paused by Guide. Display's CRT profile stepped with the pad is stored as
//! Settings stores it and the paused picture behind the card is drawn through the CRT at once; Crop overscan stepped to
//! Custom crop shows its four edges, and an edge stepped is stored and crops the paused picture at once; Sound's Volume stepped
//! down is stored and the session's sound player holds it. On the RX 6800 unless EMUSEN_GPU_DEVICE names another; the
//! sound on SDL's dummy driver.

mod common;

use common::console::{card_words, choose_row, console_owns_the_pad, focused_words, guide_alone, list_card_start_console, picture_points, until};
use common::{approved_console_core, sandbox, wait_for};

use emusen_client_fixture::cores::installation_with_snes;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{Harness, Scene};

/// The window's mean red, green and blue over the console's picture, under the card.
fn mean(scene: &mut Scene<'_>) -> [f64; 3] {
    let points = picture_points(scene);
    let seen: Vec<_> = scene.colours_at(&points).into_iter().flatten().collect();
    assert_eq!(seen.len(), points.len(), "every point read");
    let n = seen.len() as f64;
    [seen.iter().map(|c| f64::from(c.r)).sum::<f64>() / n, seen.iter().map(|c| f64::from(c.g)).sum::<f64>() / n, seen.iter().map(|c| f64::from(c.b)).sum::<f64>() / n]
}

fn apart(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f64::max)
}

fn number(scene: &mut Scene<'_>, name: &str, property: &str) -> f64 {
    scene.property(name, property).parse().unwrap_or(f64::NAN)
}

fn stored(platform: &Platform, model: u32, file: &str) -> serde_json::Value {
    platform.model_load(model, None, file).unwrap().unwrap_or_default()
}

fn graphics(platform: &Platform) -> serde_json::Value {
    let file = platform.file(FILE_GRAPHICS).unwrap();
    stored(platform, MODEL_GRAPHICS_CONFIG, file.file_name().unwrap().to_str().unwrap())
}

/// Down until the focused row reads `words`.
fn focus_row(scene: &mut Scene<'_>, words: &str) {
    for _ in 0..12 {
        if focused_words(scene).starts_with(words) {
            return;
        }
        scene.pad(UiButton::Down);
        scene.wait(60);
    }
    panic!("no row reads {words}; the focus reads {}", focused_words(scene));
}

/// An item's place in the window: x, y, width and height.
fn place(scene: &mut Scene<'_>, name: &str) -> (f64, f64, f64, f64) {
    let tree = scene.tree();
    tree.nodes.iter().find(|n| n.name == name).map(|n| (n.x, n.y, n.w, n.h)).unwrap_or_else(|| panic!("the {name} item"))
}

/// The value the card's row `label` shows.
fn value_of(scene: &mut Scene<'_>, label: &str) -> String {
    card_words(scene).into_iter().find(|(l, _)| l == label).map(|(_, v)| v).unwrap_or_else(|| panic!("no row {label}"))
}

#[test]
fn the_pause_cards_display_and_sound_store_settings_rows_and_the_paused_game_follows_them() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(400);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let world = sandbox("qt-app-pause-settings", &[("nes", "Brass Lantern.nes")]);
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &world.dir("presets")).expect("a Vulkan device");
    let adoption = host.adoption();
    let cores = installation_with_snes(&world.dir("lib"));
    approved_console_core(&cores);
    let mut app = stored(&platform, MODEL_APP_SETTINGS, "appsettings.json");
    app["Volume"] = serde_json::json!(0.5);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &app).unwrap();

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", common::HELD_NOW));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 10, "the console listed", |s| s.property("library", "startable").contains("\"PlayStation\""));
    until(&mut scene, 20, "the virtual pad read", |s| s.property("pads", "connected") == "1");

    // The PlayStation started from the List card, plain, and paused by Guide.
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    list_card_start_console(&mut scene);
    scene.pad(UiButton::Accept);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    assert!(graphics(&platform)["Consoles"]["PlayStation"]["ScreenFilter"].as_str().is_none_or(|f| f != "CRT"), "the picture starts plain");
    guide_alone(&mut scene, &sdl, &pad);
    until(&mut scene, 5, "the card open", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    until(&mut scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);

    // Game Options ▸ Display: the console's rows as Settings has them, CRT profile off.
    choose_row(&mut scene, "Game Options");
    until(&mut scene, 5, "Game Options' page", |s| s.property("pause", "title") == "Game Options");
    choose_row(&mut scene, "Display");
    until(&mut scene, 5, "Display's page", |s| s.property("pause", "title") == "Display" && !card_words(s).is_empty());
    let labels: Vec<String> = card_words(&mut scene).into_iter().map(|(l, _)| l).collect();
    assert_eq!(labels, ["CRT profile", "Shader levers", "Crop overscan", "Picture shape"], "a television's display rows");
    assert_eq!(value_of(&mut scene, "CRT profile"), "Off");
    scene.wait(300);
    let plain = mean(&mut scene);
    let counts = host.feed.counts();
    let from = |c: &std::sync::atomic::AtomicU64| c.load(std::sync::atomic::Ordering::Relaxed);
    eprintln!("plain, under the card: {plain:?}; frames drawn {} from a lend {} from a ring {}", from(&counts.shown), from(&counts.shown_lent), from(&counts.shown_ring));

    // Right on CRT profile: stored as Settings stores it, and the paused picture drawn through the CRT behind the card.
    // The paused picture read again with nothing changed: how far two reads of a still picture differ.
    scene.wait(300);
    let still = apart(mean(&mut scene), plain);
    eprintln!("a still picture read twice differs by {still}");
    focus_row(&mut scene, "CRT profile");
    scene.pad(UiButton::Right);
    until(&mut scene, 5, "the CRT stored", |_| graphics(&platform)["Consoles"]["PlayStation"]["ScreenFilter"] == "CRT");
    assert_eq!(graphics(&platform)["ShaderParameters"]["PlayStation"]["CRT"]["quality"], "0", "Performance, the first tier");
    until(&mut scene, 5, "the row turned", |s| value_of(s, "CRT profile") == "Performance");
    assert_eq!(scene.property("session", "paused"), "true", "the game still paused");
    until(&mut scene, 10, "the paused picture drawn through the CRT", |s| apart(mean(s), plain) > 4.0 * still.max(1.0));
    eprintln!("through the CRT, under the card: {:?}", mean(&mut scene));
    assert_eq!(host.feed.problem(), None, "the CRT's chain built and drew");

    // Left again: off, stored, and the picture as it was.
    scene.pad(UiButton::Left);
    until(&mut scene, 5, "the CRT off again", |_| graphics(&platform)["Consoles"]["PlayStation"]["ScreenFilter"] != "CRT");
    until(&mut scene, 10, "the paused picture plain again", |s| apart(mean(s), plain) <= still.max(1.0));

    // Crop overscan stepped to Custom crop: its four edges follow it. Right five times on the left edge stores 5, and the
    // paused picture's shown rectangle behind the card narrows by 5% at once, the picture's left 5% past its clip. The
    // rectangle is the session's crop and shape, which the core does not move.
    focus_row(&mut scene, "Crop overscan");
    scene.pad(UiButton::Right);
    until(&mut scene, 5, "TV overscan stored", |_| graphics(&platform)["Consoles"]["PlayStation"]["Overscan"] == "tv");
    scene.pad(UiButton::Right);
    until(&mut scene, 5, "Custom crop stored", |_| graphics(&platform)["Consoles"]["PlayStation"]["Overscan"] == "custom");
    until(&mut scene, 5, "the edges shown", |s| card_words(s).iter().any(|(l, _)| l == "Bottom edge"));
    let labels: Vec<String> = card_words(&mut scene).into_iter().map(|(l, _)| l).collect();
    assert_eq!(labels, ["CRT profile", "Shader levers", "Crop overscan", "Left edge", "Right edge", "Top edge", "Bottom edge", "Picture shape"]);
    until(&mut scene, 5, "the whole picture shown", |s| place(s, "shown").2 > 0.0 && (place(s, "picture").2 - place(s, "shown").2).abs() < 0.5);
    let (_, _, whole, high) = place(&mut scene, "shown");
    focus_row(&mut scene, "Left edge");
    for _ in 0..5 {
        scene.pad(UiButton::Right);
        scene.wait(80);
    }
    until(&mut scene, 5, "the left edge stored", |_| graphics(&platform)["Consoles"]["PlayStation"]["CropLeft"] == "5");
    until(&mut scene, 5, "the row turned", |s| value_of(s, "Left edge") == "5%");
    until(&mut scene, 5, "the paused picture cropped behind the card", |s| (place(s, "shown").2 - whole * 0.95).abs() < 1.5);
    let (qx, _, qw, qh) = place(&mut scene, "shown");
    let (px, _, pw, _) = place(&mut scene, "picture");
    assert!((qh - high).abs() < 1.5, "the height kept: {qh} against {high}");
    assert!((pw - whole).abs() < 1.5 && (px - (qx - 0.05 * pw)).abs() < 1.5, "the whole picture drawn, its left 5% outside the clip: {px} {pw} against {qx} {qw}");
    assert_eq!(scene.property("session", "paused"), "true", "the game still paused");
    assert_eq!(scene.property("shown", "clip"), "true");

    // B to Game Options, on Display; then Sound: Volume a step down, stored, and the session's player holds it.
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "Game Options again", |s| s.property("pause", "title") == "Game Options" && focused_words(s).starts_with("Display"));
    choose_row(&mut scene, "Sound");
    until(&mut scene, 5, "Sound's page", |s| s.property("pause", "title") == "Sound" && !card_words(s).is_empty());
    assert_eq!(card_words(&mut scene).into_iter().map(|(l, _)| l).collect::<Vec<_>>(), ["Volume", "Output"]);
    assert_eq!(value_of(&mut scene, "Volume"), "50%");
    focus_row(&mut scene, "Volume");
    scene.pad(UiButton::Left);
    until(&mut scene, 5, "the volume stored", |_| stored(&platform, MODEL_APP_SETTINGS, "appsettings.json")["Volume"] == serde_json::json!(0.45));
    until(&mut scene, 5, "the session at the new volume", |s| (number(s, "session", "volume") - 0.45).abs() < 1e-6);
    assert_eq!(scene.property("session", "sounding"), "true", "the console's sound player open: {}", scene.property("session", "problem"));
    until(&mut scene, 5, "the sound player at the new volume", |s| (number(s, "session", "soundVolume") - 0.45).abs() < 1e-6);
    assert_eq!(value_of(&mut scene, "Volume"), "45%");
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "Game Options again", |s| s.property("pause", "title") == "Game Options" && focused_words(s).starts_with("Sound"));

    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(scene);
    drop(harness);
    drop(host);
}
