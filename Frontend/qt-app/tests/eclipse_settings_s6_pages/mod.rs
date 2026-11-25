//! The oracle of Eclipse's Settings on Players, Profiles, One profile, Hotkeys and Sound Output, by pad and keyboard (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{held, read, rows};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, help_line, lines_held, pictures, settle, shown};
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::bindings::consts as binding_files;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

const WORLD_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/eclipse/settings_s6_world.json");
const PLAYERS_BOARD: &str = include_str!("../eclipse/settings_players.tsv");
const PLAYERS_LINES: &str = include_str!("../eclipse/settings_players_lines.tsv");
const PROFILES_BOARD: &str = include_str!("../eclipse/settings_profiles.tsv");
const PROFILES_LINES: &str = include_str!("../eclipse/settings_profiles_lines.tsv");
const PROFILE_BOARD: &str = include_str!("../eclipse/settings_profile.tsv");
const PROFILE_LINES: &str = include_str!("../eclipse/settings_profile_lines.tsv");
const HOTKEYS_BOARD: &str = include_str!("../eclipse/settings_hotkeys.tsv");
const HOTKEYS_LINES: &str = include_str!("../eclipse/settings_hotkeys_lines.tsv");
const OUTPUT_BOARD: &str = include_str!("../eclipse/settings_output.tsv");
const OUTPUT_LINES: &str = include_str!("../eclipse/settings_output_lines.tsv");

/// Waits until `pred` returns true or 5 seconds pass.
fn until(scene: &mut Scene<'_>, what: &str, mut pred: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..100 {
        if pred(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("timed out waiting for {what}");
}

/// Checks a board's table, line metrics, glow, help bar, words and wrapped row line counts across every gate size.
#[allow(clippy::too_many_arguments)]
fn check_board(
    scene: &mut Scene<'_>,
    board_tsv: &str,
    lines_tsv: &str,
    glow: &str,
    want_help: &[&str],
    words: &[(String, String)],
    want_lines: &[(usize, i32)],
    differences: &mut Vec<String>,
) {
    let (board_height, table) = rows(board_tsv);
    let k = DESIGN_HEIGHT / board_height;
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let at_size = format!("{width}x{height}");
        let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
        let screen: u64 = tree.nodes[tree.find_named("settingsScreen").unwrap() as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, differences);
        lines_held(&tree, lines_tsv, k, &at_size, differences);
        if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse(glow) {
            differences.push(format!("{at_size}: glow is {} where board wants {glow}", read(&tree, "backdropField.0", "paint")));
        }
        for (name, want) in words {
            let got = read(&tree, name, "text");
            if got != *want {
                differences.push(format!("{at_size}: {name} is {got:?} where board wants {want:?}"));
            }
        }
        for &(row_i, lines) in want_lines {
            let name = format!("rowAbout.{row_i}");
            let lc: i32 = read(&tree, &name, "lineCount").parse().unwrap_or(-1);
            let cut = read(&tree, &name, "truncated") == "true";
            if lc != lines || cut {
                differences.push(format!("{at_size}: {name} has lineCount={lc} truncated={cut} where board wants {lines} lines uncut"));
            }
        }
        let (help, _) = help_line(scene);
        if help != want_help {
            differences.push(format!("{at_size}: help is {help:?} where board wants {want_help:?}"));
        }
    }
}

/// The program in a scripted s6 world, Eclipse on Settings -> Controllers, with three virtual pads seated.
fn with_s6(tag: &str, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad, &Platform)) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new(tag);
    std::fs::write(world.dir("Roms/snes").join("Hero.sfc"), vec![0u8; 4096]).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut stored = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    stored["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    stored["SoundOutputDevice"] = serde_json::json!("Living room television");
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &stored).unwrap();
    let pad_defaults = platform.bindings_defaults(binding_files::GAMEPAD).unwrap();
    let profiles = serde_json::json!({ "SNES": { "Default": "Default", "Profiles": { "Couch": { "1": pad_defaults } } } });
    platform.bindings_save(binding_files::PROFILES, &profiles).unwrap();
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }
    let sdl = Sdl::open();
    let pad1 = sdl.attach("Wireless Controller", 0x045e, 0x028e);
    let pad2 = sdl.attach("Travel Pad", 0x045e, 0x028f);
    let pad3 = sdl.attach("Arcade Stick", 0x045e, 0x0290);
    let only_pads = format!("{},{},{}", pad1.id, pad2.id, pad3.id);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &only_pads));
    assert!(scene.set_property("settingsModel", "world", WORLD_JSON));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "3", 5000);
    scene.run_pad_clock();
    // Detach Travel Pad so Player 2 becomes a kept disconnected seat.
    sdl.detach(pad2);
    wait_for(&mut scene, "pads", "connected", "2", 5000);
    for _ in 0..40 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..5 {
        press(&mut scene, &sdl, &pad1, PadControl::Right);
    }
    press(&mut scene, &sdl, &pad1, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    for _ in 0..40 {
        if scene.property("theme", "settings") == "true" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(scene.property("theme", "settings"), "true", "Eclipse draws Settings");
    assert_eq!(at(&mut scene).1, "controllers");
    f(&mut scene, &sdl, &pad1, &platform);
    sdl.detach(pad3);
    sdl.detach(pad1);
}

/// Boards #52 (Players), #53 (Profiles), #54 (One profile), #55 (Hotkeys) and #56 (Sound Output).
pub fn s6(backend: &str) {
    with_s6("qt-app-eclipse-settings-s6", |scene, sdl, pad, platform| {
        let mut differences = Vec::new();
        let page_help = &["Value", "Change", "Back", "Home"];

        // Board #52: Controllers -> Players.
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "players".to_owned()), "opens Players");
        // Move Arcade Stick from Player 3 to None (Right twice: 3 -> 4 -> None) without disturbing kept Player 2.
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::Right);
        until(scene, "Arcade Stick Player 4", |s| read(&s.tree(), "rowWords.1", "text") == "Player 4");
        press(scene, sdl, pad, PadControl::Right);
        until(scene, "Arcade Stick None", |s| read(&s.tree(), "rowWords.1", "text") == "None");
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        // Hold LeftStick (button 7, no UI action) on Wireless Controller so row 0 shows the held-button mark ● Player 1.
        sdl.press(pad, 7, true);
        until(scene, "Wireless Controller marked", |s| read(&s.tree(), "rowWords.0", "text") == "● Player 1");
        let players_words = vec![
            ("settingsCrumb".into(), "Controllers".into()),
            ("settingsHeading".into(), "Players".into()),
            ("settingsPlace".into(), "1 of 4".into()),
            ("asideHeading".into(), "Players".into()),
            ("asideSentence".into(), "Player 1 · Wireless Controller · Default profile".into()),
            ("asideLine.0".into(), "Player 2 · Travel Pad · kept".into()),
            ("asideMuted.0".into(), "Players 3 and 4 · none".into()),
            ("asideMuted.1".into(), "Left and Right on a controller's row pick its player.".into()),
            ("rowName.0".into(), "Wireless Controller".into()),
            ("rowAbout.0".into(), "The player this controller plays as; a button held on it marks its row.".into()),
            ("rowWords.0".into(), "● Player 1".into()),
            ("rowName.1".into(), "Arcade Stick".into()),
            ("rowAbout.1".into(), "Plays as no one, and still steers the menus; a button held on it marks its row.".into()),
            ("rowWords.1".into(), "None".into()),
            ("rowName.2".into(), "Player 2: Travel Pad, disconnected".into()),
            ("rowAbout.2".into(), "Kept for this controller, which gets the number back when it reconnects.".into()),
            ("rowWords.2".into(), "Forget".into()),
            ("rowName.3".into(), "Keyboard".into()),
            ("rowAbout.3".into(), "The player the keyboard plays as, beside that player's controller.".into()),
            ("rowWords.3".into(), "Player 1".into()),
        ];
        check_board(scene, PLAYERS_BOARD, PLAYERS_LINES, "#3F6FE0", page_help, &players_words, &[(0, 1), (1, 2), (2, 1), (3, 1)], &mut differences);
        pictures(scene, backend, "players");
        sdl.press(pad, 7, false);
        until(scene, "Wireless Controller unmarked", |s| read(&s.tree(), "rowWords.0", "text") == "Player 1");
        // Keyboard steps Player 1 -> Player 2 -> Player 1; Forget removes kept seat row.
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.3", "text") != "Player 2" {
            differences.push(format!("Keyboard Right gave {:?}", read(&scene.tree(), "rowWords.3", "text")));
        }
        press(scene, sdl, pad, PadControl::Left);
        settle(scene);
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        until(scene, "kept Travel Pad forgotten", |s| read(&s.tree(), "settingsPlace", "text") == "3 of 3");
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene), (0, "controllers".to_owned()), "B returns to Controllers Players row");

        // Board #53: Controllers -> Profiles (Couch focused -> 3 of 4).
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "profiles".to_owned()), "opens Profiles");
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let profiles_words = vec![
            ("settingsCrumb".into(), "Controllers".into()),
            ("settingsHeading".into(), "Profiles".into()),
            ("settingsPlace".into(), "3 of 4".into()),
            ("asideHeading".into(), "Couch".into()),
            ("asideSentence".into(), "Couch is kept for the games whose page chooses it.".into()),
            ("asideMuted.0".into(), "A opens it: its bindings, starting games with it, a new name, or deleting it.".into()),
            ("asideMuted.1".into(), "Super Nintendo · 2 profiles · Default starts its games".into()),
            ("rowName.0".into(), "Console".into()),
            ("rowAbout.0".into(), "The console whose profiles these are.".into()),
            ("rowWords.0".into(), "Super Nintendo".into()),
            ("rowName.1".into(), "Default".into()),
            ("rowAbout.1".into(), "The bindings every console starts with; it cannot be renamed or deleted.".into()),
            ("rowWords.1".into(), "● Starts games".into()),
            ("rowName.2".into(), "Couch".into()),
            ("rowAbout.2".into(), "A set of Super Nintendo bindings of your own.".into()),
            ("rowName.3".into(), "New profile".into()),
            ("rowAbout.3".into(), "A copy of Default, named by you, for Super Nintendo games.".into()),
            ("rowWords.3".into(), "…".into()),
        ];
        check_board(scene, PROFILES_BOARD, PROFILES_LINES, "#3F6FE0", page_help, &profiles_words, &[(0, 1), (1, 2), (2, 1), (3, 1)], &mut differences);
        pictures(scene, backend, "profiles");
        // Stepping Console Right switches to PlayStation and back to Super Nintendo.
        press(scene, sdl, pad, PadControl::Up);
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "PlayStation"
            || read(&scene.tree(), "asideMuted.0", "text") != "PlayStation · 1 profile · Default starts its games"
        {
            differences.push(format!("Profiles Console Right mismatch: val={:?} muted={:?}", read(&scene.tree(), "rowWords.0", "text"), read(&scene.tree(), "asideMuted.0", "text")));
        }
        press(scene, sdl, pad, PadControl::Left);
        settle(scene);
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);

        // Board #54: Controllers -> Profiles -> Couch (Starts every game focused -> 2 of 4).
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "profile:SNES:Couch".to_owned()), "opens Couch profile");
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let profile_words = vec![
            ("settingsCrumb".into(), "Controllers  ›  Profiles".into()),
            ("settingsHeading".into(), "Couch".into()),
            ("settingsPlace".into(), "2 of 4".into()),
            ("asideHeading".into(), "Starts every game".into()),
            ("asideSentence".into(), "Super Nintendo games start with Couch unless a game's page chooses another.".into()),
            ("asideMuted.0".into(), "A game can start with a profile of its own, chosen on its page.".into()),
            ("rowName.0".into(), "Controller bindings".into()),
            ("rowAbout.0".into(), "The Super Nintendo buttons this profile puts on your controller.".into()),
            ("rowName.1".into(), "Starts every game".into()),
            ("rowAbout.1".into(), "Super Nintendo games start with Couch unless a game's page chooses another.".into()),
            ("rowName.2".into(), "Rename".into()),
            ("rowAbout.2".into(), "A new name; games that chose it keep it.".into()),
            ("rowWords.2".into(), "…".into()),
            ("rowName.3".into(), "Delete".into()),
            ("rowAbout.3".into(), "Games that chose Couch start with the console's profile again.".into()),
        ];
        check_board(scene, PROFILE_BOARD, PROFILE_LINES, "#3F6FE0", page_help, &profile_words, &[(0, 1), (1, 1), (2, 1), (3, 1)], &mut differences);
        pictures(scene, backend, "profile");
        // Starts every game sets Couch as default (● Chosen), then Delete removes Couch cleanly.
        press(scene, sdl, pad, PadControl::South);
        until(scene, "Couch chosen as default", |s| read(&s.tree(), "rowWords.1", "text") == "● Chosen");
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        until(scene, "Delete returns to Profiles", |s| at(s).1 == "profiles" && read(&s.tree(), "settingsPlace", "text") == "1 of 3");
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene), (2, "controllers".to_owned()), "B returns to Controllers Profiles row");

        // Board #55: Controllers -> Hotkeys (Save State focused -> 3 of 9; Big Picture hidden per R-20261112-12).
        for _ in 0..5 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "hotkeys".to_owned()), "opens Hotkeys");
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let hotkeys_words = vec![
            ("settingsCrumb".into(), "Controllers".into()),
            ("settingsHeading".into(), "Hotkeys".into()),
            ("settingsPlace".into(), "3 of 9".into()),
            ("asideHeading".into(), "Save State".into()),
            ("asideSentence".into(), "Bound to the key F5.".into()),
            ("asideLine.0".into(), "Fires once each time the key is pressed.".into()),
            ("asideMuted.0".into(), "A listens for a key; X clears it. A hotkey is every console's, so its key is taken from any console's button.".into()),
            ("rowName.0".into(), "Fast Forward".into()),
            ("rowAbout.0".into(), "Applies while the key is held.".into()),
            ("rowWords.0".into(), "Tab".into()),
            ("rowName.1".into(), "Rewind".into()),
            ("rowAbout.1".into(), "Applies while the key is held.".into()),
            ("rowWords.1".into(), "Back".into()),
            ("rowName.2".into(), "Save State".into()),
            ("rowAbout.2".into(), "Fires once each time the key is pressed.".into()),
            ("rowWords.2".into(), "F5".into()),
            ("rowName.3".into(), "Load State".into()),
            ("rowAbout.3".into(), "Fires once each time the key is pressed.".into()),
            ("rowWords.3".into(), "F8".into()),
            ("rowName.4".into(), "Pause / Resume".into()),
            ("rowAbout.4".into(), "Fires once each time the key is pressed.".into()),
            ("rowWords.4".into(), "P".into()),
            ("rowName.5".into(), "Fullscreen".into()),
            ("rowAbout.5".into(), "Fires once each time the key is pressed.".into()),
            ("rowWords.5".into(), "F11".into()),
            ("rowName.6".into(), "Exit to Library".into()),
            ("rowAbout.6".into(), "Fires once each time the key is pressed.".into()),
            ("rowWords.6".into(), "Escape".into()),
            ("rowName.7".into(), "Screenshot".into()),
            ("rowAbout.7".into(), "Fires once each time the key is pressed.".into()),
            ("rowWords.7".into(), "F9".into()),
        ];
        check_board(
            scene,
            HOTKEYS_BOARD,
            HOTKEYS_LINES,
            "#3F6FE0",
            &["Change", "Clear", "Back", "Home"],
            &hotkeys_words,
            &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)],
            &mut differences,
        );
        pictures(scene, backend, "hotkeys");
        // Live A rebinding (F6) and X clearing on Hotkeys, plus scrolling to 9th row (Frame Advance) with no Big Picture.
        let south = sdl_button(PadControl::South).unwrap();
        sdl.press(pad, south, true);
        until(scene, "Save State listening", |s| s.property("settingsModel", "listening") == "true" && read(&s.tree(), "rowWords.2", "text") == "Press a key…");
        sdl.press(pad, south, false);
        scene.wait(60);
        scene.key(0x0100_0035, 0);
        until(scene, "Save State rebound to F6", |s| s.property("settingsModel", "listening") == "false" && read(&s.tree(), "rowWords.2", "text") == "F6");
        if read(&scene.tree(), "asideSentence", "text") != "Bound to the key F6." {
            differences.push(format!("Hotkeys rebound sentence is {:?}", read(&scene.tree(), "asideSentence", "text")));
        }
        press(scene, sdl, pad, PadControl::West);
        until(scene, "Save State cleared", |s| read(&s.tree(), "rowWords.2", "text") == "Not bound");
        if read(&scene.tree(), "asideSentence", "text") != "Not bound to a key." {
            differences.push(format!("Hotkeys cleared sentence is {:?}", read(&scene.tree(), "asideSentence", "text")));
        }
        for _ in 0..6 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        let ninth = scene.tree();
        if read(&ninth, "settingsPlace", "text") != "9 of 9" || read(&ninth, "rowName.8", "text") != "Frame Advance" {
            differences.push(format!("9th hotkey mismatch: place={:?} name={:?}", read(&ninth, "settingsPlace", "text"), read(&ninth, "rowName.8", "text")));
        }
        if ninth.nodes.iter().any(|n| n.text.as_deref() == Some("Big Picture")) {
            differences.push("Big Picture hotkey unexpectedly drawn".into());
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene), (7, "controllers".to_owned()), "B returns to Controllers Hotkeys row");

        // Board #56: Sound -> Output (Living room television focused -> 2 of 3, ● In use).
        press(scene, sdl, pad, PadControl::RightShoulder);
        press(scene, sdl, pad, PadControl::RightShoulder);
        settle(scene);
        assert_eq!(at(scene).1, "sound");
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "output".to_owned()), "A on Sound Output opens Output page");
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let output_words = vec![
            ("settingsCrumb".into(), "Sound".into()),
            ("settingsHeading".into(), "Output".into()),
            ("settingsPlace".into(), "2 of 3".into()),
            ("asideHeading".into(), "Output".into()),
            ("asideSentence".into(), "The game and the interface's sounds play on Living room television.".into()),
            ("asideMuted.0".into(), "A device not connected when EmuSen starts falls back to the system default.".into()),
            ("rowName.0".into(), "System default".into()),
            ("rowAbout.0".into(), "The game and the interface's sounds play where the system sends sound.".into()),
            ("rowName.1".into(), "Living room television".into()),
            ("rowAbout.1".into(), "The game and the interface's sounds play on Living room television.".into()),
            ("rowWords.1".into(), "● In use".into()),
            ("rowName.2".into(), "Headphones".into()),
            ("rowAbout.2".into(), "The game and the interface's sounds play on Headphones.".into()),
        ];
        check_board(
            scene,
            OUTPUT_BOARD,
            OUTPUT_LINES,
            "#B8862B",
            &["Choose", "Back", "Home"],
            &output_words,
            &[(0, 1), (1, 1), (2, 1)],
            &mut differences,
        );
        if shown(scene, "rowWords.0") || shown(scene, "rowWords.2") {
            differences.push("Inactive Output rows unexpectedly drew value words".into());
        }
        pictures(scene, backend, "output");
        // Choosing Headphones saves SoundOutputDevice and moves ● In use in place; B returns to Sound.
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        until(scene, "Headphones active", |s| read(&s.tree(), "rowWords.2", "text") == "● In use" && !shown(s, "rowWords.1"));
        assert_eq!(at(scene).1, "output", "A stays on Output moving ● In use");
        let saved = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
        if saved["SoundOutputDevice"] != serde_json::json!("Headphones") {
            differences.push(format!("SoundOutputDevice saved as {:?}", saved["SoundOutputDevice"]));
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene), (3, "sound".to_owned()), "B returns to Sound Output row");
        if read(&scene.tree(), "rowWords.3", "text") != "Headphones" {
            differences.push(format!("Sound Output row shows {:?} after choosing Headphones", read(&scene.tree(), "rowWords.3", "text")));
        }

        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        wait_for(scene, "screens", "shown", "Home", 2000);
        scene.assert_quiet();
    });
}
