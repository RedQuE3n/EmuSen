//! A game's cheats in the program on Vulkan: a game of the test core whose own list holds one cheat, a poke holding the
//! RAM byte the core counts each frame. Switched on with the pad from the Pause Card's Game Options ▸ Cheats, the game's
//! memory holds the poke's value; switched off, the count runs on; the game started again has the switch as it was
//! saved, the poke in force. The game's memory is read from states the card saves, loaded into a runtime of the test's
//! own; on the RX 6800 unless EMUSEN_GPU_DEVICE names another.

mod common;

use common::console::{card_words, choose_row, guide_alone};
use common::{sandbox, wait_for};
use emusen_client_fixture::{image, library, TEST_SYSTEM};
use emusen_crystal_client::{Api, Config, Open, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_CHEAT_FILE;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::{Harness, Scene};
use serde_json::json;
use std::path::Path;

/// The RAM byte the test core adds one to each frame, and the value the cheat holds it at.
const COUNTED: u32 = 2;
const HELD: u8 = 0x5A;

fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..200 {
        if done(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("{what} within 10 s");
}

/// The chord's Pause Card over the game, in place.
fn pause(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) {
    until(scene, "the game owning the pad", |s| s.pad_chord_only());
    scene.run_pad_clock();
    guide_alone(scene, sdl, pad);
    until(scene, "the card open", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    until(scene, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
}

/// Game Options ▸ Cheats from the card's first page, then A on the cheat: its row's value after.
fn switch_from_the_card(scene: &mut Scene<'_>) -> String {
    choose_row(scene, "Game Options");
    until(scene, "Game Options' page", |s| s.property("pause", "title") == "Game Options");
    choose_row(scene, "Cheats");
    until(scene, "Cheats' page", |s| s.property("pause", "title") == "Cheats");
    scene.pad(UiButton::Accept);
    scene.wait(100);
    let value = card_words(scene)[0].1.clone();
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    until(scene, "the first page again", |s| s.property("pause", "breadcrumb").is_empty());
    value
}

/// Save State on the card's first page to `slot`, chosen with Right on its row; A saves and resumes the game.
fn save(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, slot: u32) {
    pause(scene, sdl, pad);
    for _ in 0..12 {
        if common::console::focused_words(scene).starts_with("Save State") {
            break;
        }
        scene.pad(UiButton::Down);
    }
    for _ in 0..8 {
        if scene.property("session", "slot") == slot.to_string() {
            break;
        }
        scene.pad(UiButton::Right);
        scene.wait(50);
    }
    assert_eq!(scene.property("session", "slot"), slot.to_string());
    choose_row(scene, "Save State");
    until(scene, "the save resuming the game", |s| s.property("session", "notice") == format!("State saved to slot {slot}") && s.property("session", "paused") == "false");
}

/// The counted byte of the game's memory as the state in `slot` holds it, read in a runtime of the test's own.
fn counted_in(config: &Config, game: &Path, slot: u32) -> u8 {
    let rt = Runtime::new(&Api::load(library()).unwrap(), config).unwrap();
    let session = rt.open(&Open { path: Some(game.to_str().unwrap().into()), ..Open::default() }).unwrap();
    assert!(rt.state_slot_load(session, slot).unwrap(), "slot {slot} loads");
    let mut byte = [0u8; 1];
    rt.session_service(session, 0).unwrap().space_read(session, "RAM", COUNTED, &mut byte).unwrap();
    rt.close(session).unwrap();
    byte[0]
}

/// The game's list's one cheat as its file holds it: whether it is on.
fn saved_on(platform: &Platform, list: &(String, String)) -> bool {
    let file = platform.model_load(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0)).unwrap().unwrap();
    file["Cheats"][0]["Enabled"].as_bool().unwrap()
}

#[test]
fn a_cheat_switched_on_the_pause_card_holds_the_games_memory_and_is_saved_for_the_next_start() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let presets = std::env::temp_dir().join("qt-app-cheats-gpu-presets");
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &presets).expect("a Vulkan device");
    let adoption = host.adoption();
    let world = sandbox("qt-app-cheats-gpu", &[]);
    let game = world.dir("Roms").join("Cheat (Synthetic).tst");
    std::fs::write(&game, image(&[1, 2, 3, 4])).unwrap();
    let player = world.dir("Home/Cores");
    let core = world.player_core(&player);
    let config = Config { player_cores: Some(vec![player.to_str().unwrap().into()]), ..Config::default() };
    let rt = Runtime::new(&Api::load(library()).unwrap(), &config).unwrap();
    rt.core_approve(&emusen_client_fixture::cores::sha256(&core)).unwrap();
    // The game's own list where the program looks for it, by the test system's console: one poke, saved off.
    let console = rt.systems().unwrap().into_iter().find(|s| s.id == TEST_SYSTEM).expect("the test system").console;
    drop(rt);
    let list = platform.cheat_game_list(Some(&console), game.to_str().unwrap()).unwrap().expect("a list for the game");
    let poke = json!({ "Kind": "RamPoke", "Description": "Counter held", "Enabled": false,
        "Writes": [{ "Space": "RAM", "Address": format!("{COUNTED:X}"), "Value": format!("{HELD:02X}") }] });
    platform.model_save(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0), &json!({ "Cheats": [poke] })).unwrap();

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, "the virtual pad read", |s| s.property("pads", "connected") == "1");

    assert!(scene.set_property("game", "game", game.to_str().unwrap()));
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    until(&mut scene, "the game running", |s| s.property("session", "running") == "true");
    let problem = scene.property("session", "problem");
    assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");

    // Off as saved: the count runs. Switched on from the card, the game's memory holds the poke's value, and the
    // switch is saved to the game's list.
    pause(&mut scene, &sdl, &pad);
    assert_eq!(switch_from_the_card(&mut scene), "On", "the row turns at once");
    assert!(saved_on(&platform, &list), "the switch saved to the game's list");
    scene.pad(UiButton::Back);
    until(&mut scene, "the game again", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    scene.wait(300);
    save(&mut scene, &sdl, &pad, 1);
    assert_eq!(counted_in(&config, &game, 1), HELD, "switched on, the cheat holds the counted byte");

    // Switched off, the count runs on: two states saved apart differ.
    pause(&mut scene, &sdl, &pad);
    assert_eq!(switch_from_the_card(&mut scene), "Off");
    assert!(!saved_on(&platform, &list));
    scene.pad(UiButton::Back);
    until(&mut scene, "the game again", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    scene.wait(300);
    save(&mut scene, &sdl, &pad, 2);
    scene.wait(300);
    save(&mut scene, &sdl, &pad, 3);
    let (two, three) = (counted_in(&config, &game, 2), counted_in(&config, &game, 3));
    assert_ne!(two, three, "switched off, the count runs on");

    // On again and the game closed: started again, its card shows the cheat on and its memory holds the value.
    pause(&mut scene, &sdl, &pad);
    assert_eq!(switch_from_the_card(&mut scene), "On");
    scene.pad(UiButton::Back);
    until(&mut scene, "the game again", |s| s.property("screens", "shown") == "Game");
    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    assert!(scene.set_property("game", "game", game.to_str().unwrap()));
    until(&mut scene, "the game running again", |s| s.property("session", "running") == "true" && s.property("screens", "shown") == "Game");
    scene.wait(300);
    save(&mut scene, &sdl, &pad, 4);
    assert_eq!(counted_in(&config, &game, 4), HELD, "started again, the saved switch is in force");
    pause(&mut scene, &sdl, &pad);
    choose_row(&mut scene, "Game Options");
    until(&mut scene, "Game Options' page", |s| s.property("pause", "title") == "Game Options");
    choose_row(&mut scene, "Cheats");
    until(&mut scene, "Cheats' page", |s| s.property("pause", "title") == "Cheats");
    assert_eq!(card_words(&mut scene), [("Counter held".to_owned(), "On".to_owned())], "the card lists the game's cheat as saved");

    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    // The window hidden and gone before the device it adopted, as acceptance.rs ends.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
