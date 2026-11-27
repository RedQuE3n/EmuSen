//! The oracle of Eclipse's Settings on Consoles, one console, Core and Firmware, each at its board's state, by pad and pointer (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{boxes, held, read, rows};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, centre, help_line, lines_held, pictures, settle, shown};
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

const WORLD_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qt-app/tests/eclipse/settings_consoles_world.json");
const CONSOLES_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_consoles.tsv");
const CONSOLES_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_consoles_lines.tsv");
const CONSOLE_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_console.tsv");
const CONSOLE_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_console_lines.tsv");
const CORE_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_core.tsv");
const CORE_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_core_lines.tsv");
const FIRMWARE_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_firmware.tsv");
const FIRMWARE_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_firmware_lines.tsv");
const CONTENT_NOTE: &str = "A file with a known checksum is found by its content, even renamed; others by name and size. Your folder is only ever read.";

/// The program in a scripted Consoles world with games on NES, SNES and PlayStation, Eclipse chosen on Settings and RB to Consoles.
fn with_consoles(tag: &str, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad)) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new(tag);
    let mut nes = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    nes.resize(16 + 16384, 0);
    let snes = vec![0u8; 32768];
    let mut psx = vec![0u8; 4096];
    psx[..8].copy_from_slice(b"PS-X EXE");
    std::fs::write(world.dir("Roms/nes").join("One.nes"), &nes).unwrap();
    std::fs::write(world.dir("Roms/snes").join("One.sfc"), &snes).unwrap();
    std::fs::write(world.dir("Roms/psx").join("One.psexe"), &psx).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut stored = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    stored["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    stored["FirmwareDirectory"] = serde_json::json!("/home/deck/Firmware");
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &stored).unwrap();
    let mut graphics = platform.model_new(MODEL_GRAPHICS_CONFIG).unwrap();
    graphics["Consoles"] = serde_json::json!({
        "PlayStation": { "ScreenFilter": "CRT", "Overscan": "tv", "PictureShape": "tv" }
    });
    let graphics_file = platform.file(FILE_GRAPHICS).unwrap();
    let graphics_name = graphics_file.file_name().unwrap().to_str().unwrap();
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, graphics_name, &graphics).unwrap();
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("settingsModel", "world", WORLD_JSON));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();
    for _ in 0..40 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..5 {
        press(&mut scene, &sdl, &pad, PadControl::Right);
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
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
    for _ in 0..3 {
        press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
        settle(&mut scene);
    }
    assert_eq!(at(&mut scene).1, "consoles", "RB reaches Consoles");
    f(&mut scene, &sdl, &pad);
}

/// Navigates from Consoles card 0 to PlayStation (index 5 in our 14-console world).
fn focus_playstation(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) {
    press(scene, sdl, pad, PadControl::Down);
    press(scene, sdl, pad, PadControl::Down);
    press(scene, sdl, pad, PadControl::Right);
    settle(scene);
    assert_eq!(at(scene), (5, "consoles".to_owned()), "2-col pad nav focuses PlayStation at index 5");
}

/// Checks a page's row short-line ellipses: cut at 24 px before value where and only where expected.
fn check_ellipses(scene: &mut Scene<'_>, count: usize, want_cut: &[bool], k: f64, at_size: &str, differences: &mut Vec<String>) {
    let tree = scene.tree();
    let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
    let got = boxes(&tree, unit);
    for i in 0..count {
        if !shown(scene, &format!("row.{i}")) || !shown(scene, &format!("rowAbout.{i}")) {
            continue;
        }
        let (line, value) = (got[&format!("rowAbout.{i}")], got[&format!("rowValue.{i}")]);
        let cut = read(&tree, &format!("rowAbout.{i}"), "truncated") == "true";
        if line.right() > value.x - 24.0 * k + 1e-6 || (cut && line.right() < value.x - 24.0 * k - 1e-6) {
            differences.push(format!("{at_size}: row {i}'s short line ends at {} with value at {}, cut {cut}", line.right(), value.x));
        }
        if cut != want_cut[i] {
            differences.push(format!("{at_size}: row {i}'s short line cut {cut} where board wants {}", want_cut[i]));
        }
    }
}

/// Board #27: Settings -> Consoles (2-column grid, dynamic glow, chips, LT/RT letter jump, Y filter, scrolling).
pub fn consoles(backend: &str) {
    with_consoles("qt-app-eclipse-settings-consoles", |scene, sdl, pad| {
        focus_playstation(scene, sdl, pad);
        let mut differences = Vec::new();
        let (board_height, table) = rows(CONSOLES_BOARD);
        let k = DESIGN_HEIGHT / board_height;
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let at_size = format!("{width}x{height}");
            let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
            let screen: u64 = tree.nodes[tree.find_named("settingsScreen").unwrap() as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(screen);
            held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
            lines_held(&tree, CONSOLES_LINES, k, &at_size, &mut differences);
            if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse("#4A7890") {
                differences.push(format!("{at_size}: PlayStation glow is {}", read(&tree, "backdropField.0", "paint")));
            }
            for (name, want) in [
                ("consolesHeading", "Consoles"),
                ("consolesSubtitle", "14 consoles. Each one's core, firmware, memory cards and picture."),
                ("settingsPlace", "6 of 14"),
                ("consoleName.0", "Nintendo Entertainment System"),
                ("consoleSub.0", "1 game · Moon"),
                ("consoleName.4", "Super Nintendo"),
                ("consoleSub.4", "1 game · Venus"),
                ("consoleName.5", "PlayStation"),
                ("consoleSub.5", "0 games · Diamond"),
                ("consoleName.7", "Dreamcast"),
                ("consoleSub.7", "0 games · Flycast"),
                ("consoleName.8", "Game Boy Color"),
                ("consoleSub.8", "0 games · Mercury"),
                ("consoleName.9", "PlayStation 2"),
                ("consoleChipWords.9", "Needs firmware"),
                ("consoleName.10", "32X"),
                ("consoleSub.10", "0 games · no core yet"),
                ("consoleChipWords.10", "Not supported yet"),
                ("consoleName.11", "Saturn"),
                ("consoleChipWords.11", "Not supported yet"),
            ] {
                let got = read(&tree, name, "text");
                if got != want {
                    differences.push(format!("{at_size}: {name} is {got:?} where board wants {want:?}"));
                }
            }
            let (help, _) = help_line(scene);
            let want_help = ["Open the console", "Jump a letter", "Only consoles with games", "Back", "Section", "Home"];
            if help != want_help {
                differences.push(format!("{at_size}: help is {help:?} where board wants {want_help:?}"));
            }
        }
        pictures(scene, backend, "consoles");

        // Dynamic page glow: Super Nintendo (index 4, Venus #7A5416), Dreamcast (index 7, libretro fallback #4A7890).
        press(scene, sdl, pad, PadControl::Left);
        settle(scene);
        if Colour::parse(&read(&scene.tree(), "backdropField.0", "paint")) != Colour::parse("#7A5416") {
            differences.push(format!("SNES glow is {}", read(&scene.tree(), "backdropField.0", "paint")));
        }
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        assert_eq!(at(scene).0, 7, "Down then Right reaches Dreamcast");
        if Colour::parse(&read(&scene.tree(), "backdropField.0", "paint")) != Colour::parse("#4A7890") {
            differences.push(format!("Dreamcast libretro fallback glow is {}", read(&scene.tree(), "backdropField.0", "paint")));
        }

        // Scroll past row 6 to cards 12 and 13 (Game Boy Advance, GameCube).
        press(scene, sdl, pad, PadControl::Left);
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        assert_eq!(at(scene).0, 12, "Down scrolls to row 6 card 12");
        if !shown(scene, "console.12") || !shown(scene, "console.13") || shown(scene, "console.0") {
            differences.push("scrolling to row 6 did not shift window".to_owned());
        }
        // Unsupported console declines A.
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if at(scene) != (12, "consoles".to_owned()) {
            differences.push(format!("unsupported console moved to {:?}", at(scene)));
        }

        // LT/RT letter jump steps distinct starting letters and wraps.
        let before_letter = read(&scene.tree(), "consoleName.12", "text");
        press(scene, sdl, pad, PadControl::RightTrigger);
        settle(scene);
        let after_rt = at(scene).0;
        let rt_name = read(&scene.tree(), &format!("consoleName.{after_rt}"), "text");
        if rt_name.chars().next() == before_letter.chars().next() {
            differences.push(format!("RT stayed on same initial: {before_letter} -> {rt_name}"));
        }
        press(scene, sdl, pad, PadControl::LeftTrigger);
        settle(scene);
        let after_lt = at(scene).0;
        let lt_name = read(&scene.tree(), &format!("consoleName.{after_lt}"), "text");
        if lt_name.chars().next() != before_letter.chars().next() {
            differences.push(format!("LT did not return to initial: {rt_name} -> {lt_name}"));
        }

        // Y toggles "Only consoles with games" <-> "All consoles" and leaving Consoles resets it.
        press(scene, sdl, pad, PadControl::North);
        settle(scene);
        let tree = scene.tree();
        if read(&tree, "consolesSubtitle", "text") != "2 consoles. Each one's core, firmware, memory cards and picture." {
            differences.push(format!("filtered subtitle is {:?}", read(&tree, "consolesSubtitle", "text")));
        }
        let (filtered_help, _) = help_line(scene);
        if filtered_help.get(2).map(String::as_str) != Some("All consoles") {
            differences.push(format!("filtered help is {filtered_help:?}"));
        }
        press(scene, sdl, pad, PadControl::RightShoulder);
        settle(scene);
        press(scene, sdl, pad, PadControl::LeftShoulder);
        settle(scene);
        if read(&scene.tree(), "consolesSubtitle", "text") != "14 consoles. Each one's core, firmware, memory cards and picture." {
            differences.push("leaving Consoles did not reset Y filter".to_owned());
        }

        // Pointer click on PlayStation card opens it, B returns, B leaves Settings.
        let (x, y) = centre(scene, "consoleName.5");
        scene.click(x, y);
        settle(scene);
        if at(scene).1 != "console:PlayStation" {
            differences.push(format!("click on PlayStation went to {:?}", at(scene)));
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene).1, "consoles");
        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        press(scene, sdl, pad, PadControl::East);
        wait_for(scene, "screens", "shown", "Home", 2000);
        scene.assert_quiet();
    });
}

/// Board #28: Settings -> Consoles -> PlayStation (planet card, Found 3 of 3, Memory cards Left/Right, X default, LB/RB).
pub fn console(backend: &str) {
    with_consoles("qt-app-eclipse-settings-console", |scene, sdl, pad| {
        focus_playstation(scene, sdl, pad);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "console:PlayStation".to_owned()));
        let mut differences = Vec::new();
        let (board_height, table) = rows(CONSOLE_BOARD);
        let k = DESIGN_HEIGHT / board_height;
        let want_rows = [
            ("Core", "The core that runs the PlayStation's games; A opens its own settings.", "Diamond", false),
            ("Firmware", "The firmware the PlayStation's cores use, found by its content in your folders.", "Found, 3 of 3", true),
            ("Memory cards", "The PlayStation's cards: shared by every game as the console's were, or one per game.", "One per game", true),
            ("CRT profile", "The television the PlayStation's picture is drawn on.", "Balanced", false),
            ("Shader levers", "The CRT's own levers for the PlayStation.", "", false),
            ("Crop overscan", "The whole picture, the part a television showed, or a crop of your own.", "TV overscan", true),
            ("Picture shape", "The screen the console was made for, or each pixel square.", "TV shape", false),
            ("Reset this console", "Every PlayStation setting back to its default: the core's rows, its picture and its shaders.", "", true),
        ];
        let want_cut: Vec<bool> = want_rows.iter().map(|r| r.3).collect();
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let at_size = format!("{width}x{height}");
            let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
            let screen: u64 = tree.nodes[tree.find_named("settingsScreen").unwrap() as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(screen);
            held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
            lines_held(&tree, CONSOLE_LINES, k, &at_size, &mut differences);
            check_ellipses(scene, want_rows.len(), &want_cut, k, &at_size, &mut differences);
            if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse("#4A7890") {
                differences.push(format!("{at_size}: glow is {}", read(&tree, "backdropField.0", "paint")));
            }
            let mut words = vec![
                ("settingsCrumb".to_owned(), "Consoles".to_owned()),
                ("settingsHeading".to_owned(), "PlayStation".to_owned()),
                ("settingsPlace".to_owned(), "1 of 8".to_owned()),
                ("asideCoreName".to_owned(), "Diamond".to_owned()),
                ("asideCoreSub".to_owned(), "EmuSen's own core, in use".to_owned()),
                ("asideHeading".to_owned(), "Core".to_owned()),
                ("asideSentence".to_owned(), "The core that runs the PlayStation's games. Its own settings are the rows marked with its planet.".to_owned()),
                ("asideMuted.0".to_owned(), "PlayStation · 0 games · core Diamond · CRT profile Balanced".to_owned()),
                ("asideMuted.1".to_owned(), "SwanStation and Beetle PSX can be chosen here too; this is the same setting as Cores' Make Default.".to_owned()),
            ];
            for (i, (label, about, val, _)) in want_rows.iter().enumerate() {
                words.push((format!("rowName.{i}"), (*label).to_owned()));
                words.push((format!("rowAbout.{i}"), (*about).to_owned()));
                if !val.is_empty() {
                    words.push((format!("rowWords.{i}"), (*val).to_owned()));
                }
            }
            for (name, want) in words {
                let got = read(&tree, &name, "text");
                if got != want {
                    differences.push(format!("{at_size}: {name} is {got:?} where board wants {want:?}"));
                }
            }
            let (help, x_on) = help_line(scene);
            let want_help = ["Change", "Choose or open", "Back to the default", "Back", "Console", "Home"];
            if help != want_help || x_on != "false" {
                differences.push(format!("{at_size}: help {help:?}, X enabled {x_on}"));
            }
        }
        pictures(scene, backend, "console");

        // Moving to Firmware hides the Core planet card; Memory cards steps with Left/Right, resets with X, and opens Cards with A.
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        if shown(scene, "asideConsoleCard") {
            differences.push("Core planet card still shown on Firmware row".to_owned());
        }
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        assert_eq!(at(scene).0, 2, "Memory cards focused");
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.2", "text") != "Named" {
            differences.push(format!("Memory cards Right gave {:?}", read(&scene.tree(), "rowWords.2", "text")));
        }
        press(scene, sdl, pad, PadControl::West);
        settle(scene);
        if read(&scene.tree(), "rowWords.2", "text") != "Shared" {
            differences.push(format!("Memory cards X default gave {:?}", read(&scene.tree(), "rowWords.2", "text")));
        }
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if at(scene).1 != "cards:PlayStation" {
            differences.push(format!("A on Memory cards went to {:?}", at(scene)));
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene).1, "console:PlayStation");

        // LB steps to Super Nintendo: Venus glow #7A5416, None needed firmware, cartridge Fact row on Memory cards.
        press(scene, sdl, pad, PadControl::LeftShoulder);
        settle(scene);
        let snes = scene.tree();
        if read(&snes, "settingsHeading", "text") != "Super Nintendo"
            || Colour::parse(&read(&snes, "backdropField.0", "paint")) != Colour::parse("#7A5416")
            || read(&snes, "rowWords.1", "text") != "None needed"
            || read(&snes, "rowAbout.2", "text") != "The Super Nintendo's games keep their saves in the cartridge."
        {
            differences.push("LB to Super Nintendo did not show Venus glow, None needed, and cartridge Fact".to_owned());
        }

        // RB to Dreamcast (index 7): Flycast libretro card subtitle and Video group heading as Fact.
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::RightShoulder);
            settle(scene);
        }
        let dc = scene.tree();
        if read(&dc, "settingsHeading", "text") != "Dreamcast"
            || read(&dc, "asideCoreSub", "text") != "Libretro core, in use"
            || read(&dc, "rowName.3", "text") != "Video"
            || shown(scene, "rowDoor.3")
        {
            differences.push(format!("Dreamcast libretro/group check failed: heading={:?}, sub={:?}, row3={:?}", read(&dc, "settingsHeading", "text"), read(&dc, "asideCoreSub", "text"), read(&dc, "rowName.3", "text")));
        }

        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene).1, "consoles");
        scene.assert_quiet();
    });
}

/// Board #57: Settings -> Consoles -> PlayStation -> Core (3 cores, Default/Installed, choosing a core, single-core console).
pub fn core(backend: &str) {
    with_consoles("qt-app-eclipse-settings-core", |scene, sdl, pad| {
        focus_playstation(scene, sdl, pad);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "engine:PlayStation".to_owned()));
        let mut differences = Vec::new();
        let (board_height, table) = rows(CORE_BOARD);
        let k = DESIGN_HEIGHT / board_height;
        let want_rows = [
            ("Diamond", "Diamond runs every PlayStation game.", "● Default"),
            ("SwanStation", "SwanStation would run every PlayStation game.", "✓ Installed"),
            ("Beetle PSX", "Beetle PSX would run every PlayStation game.", "✓ Installed"),
        ];
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let at_size = format!("{width}x{height}");
            let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
            let screen: u64 = tree.nodes[tree.find_named("settingsScreen").unwrap() as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(screen);
            held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
            lines_held(&tree, CORE_LINES, k, &at_size, &mut differences);
            check_ellipses(scene, want_rows.len(), &[false, false, false], k, &at_size, &mut differences);
            if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse("#4A7890") {
                differences.push(format!("{at_size}: glow is {}", read(&tree, "backdropField.0", "paint")));
            }
            let mut words = vec![
                ("settingsCrumb".to_owned(), "Consoles  ›  PlayStation".to_owned()),
                ("settingsHeading".to_owned(), "Core".to_owned()),
                ("settingsPlace".to_owned(), "1 of 3".to_owned()),
                ("asideHeading".to_owned(), "Diamond".to_owned()),
                ("asideSentence".to_owned(), "Diamond, the PlayStation's core: every PlayStation game opens on it unless the game's own options choose another.".to_owned()),
                ("asideMuted.0".to_owned(), "SwanStation and Beetle PSX can be chosen here too; this is the same setting as Cores' Make Default.".to_owned()),
                ("asideMuted.1".to_owned(), "A chooses; B keeps what was chosen.".to_owned()),
            ];
            for (i, (label, about, val)) in want_rows.iter().enumerate() {
                words.push((format!("rowName.{i}"), (*label).to_owned()));
                words.push((format!("rowAbout.{i}"), (*about).to_owned()));
                words.push((format!("rowWords.{i}"), (*val).to_owned()));
            }
            for (name, want) in words {
                let got = read(&tree, &name, "text");
                if got != want {
                    differences.push(format!("{at_size}: {name} is {got:?} where board wants {want:?}"));
                }
            }
            let (help, _) = help_line(scene);
            let want_help = ["Choose", "Back", "Home"];
            if help != want_help {
                differences.push(format!("{at_size}: help is {help:?} where board wants {want_help:?}"));
            }
        }
        pictures(scene, backend, "core");

        // Moving Down to SwanStation updates About panel; B keeps Diamond; re-opening and pressing A on SwanStation switches PlayStation's core.
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let sw = scene.tree();
        if read(&sw, "asideHeading", "text") != "SwanStation"
            || read(&sw, "asideSentence", "text") != "SwanStation would run every PlayStation game."
            || read(&sw, "asideMuted.0", "text") != "Diamond and Beetle PSX can be chosen here too; this is the same setting as Cores' Make Default."
            || read(&sw, "asideMuted.1", "text") != "A chooses; B keeps what was chosen."
        {
            differences.push("non-default core About panel mismatch".to_owned());
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "Diamond" {
            differences.push("B did not keep Diamond".to_owned());
        }
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        let psx_after = scene.tree();
        if at(scene).1 != "console:PlayStation" || read(&psx_after, "rowWords.0", "text") != "SwanStation" || read(&psx_after, "asideCoreSub", "text") != "Libretro core, in use" {
            differences.push("A on SwanStation did not switch PlayStation core to Libretro SwanStation".to_owned());
        }

        // Single-core console (Super Nintendo): 1 of 1, A re-affirms and returns.
        press(scene, sdl, pad, PadControl::LeftShoulder);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "settingsPlace", "text") != "1 of 1" || read(&scene.tree(), "rowWords.0", "text") != "● Default" {
            differences.push("single-core Super Nintendo Core page mismatch".to_owned());
        }
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if at(scene).1 != "console:SNES" {
            differences.push(format!("A on single-core console went to {:?}", at(scene)));
        }

        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        scene.assert_quiet();
    });
}

/// Board #29: Settings -> Consoles -> PlayStation -> Firmware (5 left rows, Found 3 of 3 card, PS2 missing/optional/wrong-size, SNES M==0).
pub fn firmware(backend: &str) {
    with_consoles("qt-app-eclipse-settings-firmware", |scene, sdl, pad| {
        assert!(scene.set_property("folders", "catching", "true"));
        focus_playstation(scene, sdl, pad);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (1, "firmware:PlayStation".to_owned()), "Firmware focuses first control row after split need");
        let mut differences = Vec::new();
        let (board_height, table) = rows(FIRMWARE_BOARD);
        let k = DESIGN_HEIGHT / board_height;
        let want_rows = [
            ("Open replacement", "Whether the console's open replacement runs in place of firmware of your own.", "In use when none is found", true),
            ("Folder", "Your firmware folder, searched before EmuSen's; only ever read.", "/home/deck/Firmware", true),
            ("Use EmuSen's folder alone", "Forgets your folder; firmware is then looked for in EmuSen's own.", "", false),
            ("Open EmuSen's folder", "EmuSen's own firmware folder in the file manager, made if it is not there yet.", "", false),
            ("Check again", "Reads the folders again, as after a file is put in one.", "", false),
        ];
        let want_cut: Vec<bool> = want_rows.iter().map(|r| r.3).collect();
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let at_size = format!("{width}x{height}");
            let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
            let screen: u64 = tree.nodes[tree.find_named("settingsScreen").unwrap() as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(screen);
            held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
            lines_held(&tree, FIRMWARE_LINES, k, &at_size, &mut differences);
            check_ellipses(scene, want_rows.len(), &want_cut, k, &at_size, &mut differences);
            if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse("#4A7890") {
                differences.push(format!("{at_size}: glow is {}", read(&tree, "backdropField.0", "paint")));
            }
            let mut words = vec![
                ("settingsCrumb".to_owned(), "Consoles  ›  PlayStation".to_owned()),
                ("settingsHeading".to_owned(), "Firmware".to_owned()),
                ("settingsPlace".to_owned(), "1 of 5".to_owned()),
                ("asideHeading".to_owned(), "Found, 3 of 3".to_owned()),
                ("firmwareName.0".to_owned(), "PlayStation BIOS, America".to_owned()),
                ("firmwareStatus.0".to_owned(), "Found in your folder".to_owned()),
                ("firmwareName.1".to_owned(), "PlayStation BIOS, Europe".to_owned()),
                ("firmwareStatus.1".to_owned(), "Found in your folder".to_owned()),
                ("firmwareName.2".to_owned(), "PlayStation BIOS, Japan".to_owned()),
                ("firmwareStatus.2".to_owned(), "Found in EmuSen's folder".to_owned()),
                ("asideMuted.0".to_owned(), CONTENT_NOTE.to_owned()),
            ];
            for (i, (label, about, val, _)) in want_rows.iter().enumerate() {
                words.push((format!("rowName.{i}"), (*label).to_owned()));
                words.push((format!("rowAbout.{i}"), (*about).to_owned()));
                if !val.is_empty() {
                    words.push((format!("rowWords.{i}"), (*val).to_owned()));
                }
            }
            for (name, want) in words {
                let got = read(&tree, &name, "text");
                if got != want {
                    differences.push(format!("{at_size}: {name} is {got:?} where board wants {want:?}"));
                }
            }
            for i in 0..3 {
                if !shown(scene, &format!("firmwareCheck.{i}")) || shown(scene, &format!("firmwareRing.{i}")) {
                    differences.push(format!("{at_size}: firmware item {i} checkmark visibility wrong"));
                }
            }
            let (help, x_on) = help_line(scene);
            let want_help = ["Change", "Choose or open", "Back to the default", "Back", "Home"];
            if help != want_help || x_on != "true" {
                differences.push(format!("{at_size}: help {help:?}, X enabled {x_on}"));
            }
        }
        pictures(scene, backend, "firmware");

        // Open replacement steps and resets with X; Use EmuSen's folder alone clears Folder; Check again stays on page.
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "Always" {
            differences.push(format!("Open replacement Right gave {:?}", read(&scene.tree(), "rowWords.0", "text")));
        }
        press(scene, sdl, pad, PadControl::West);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "In use when none is found" {
            differences.push(format!("Open replacement X default gave {:?}", read(&scene.tree(), "rowWords.0", "text")));
        }
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "rowWords.1", "text") != "EmuSen's folder" {
            differences.push(format!("Use EmuSen's folder alone left Folder at {:?}", read(&scene.tree(), "rowWords.1", "text")));
        }

        // PS2 Firmware: Found, 0 of 3 with required missing (rose), optional (muted), and wrong size (rose).
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        for _ in 0..4 {
            press(scene, sdl, pad, PadControl::RightShoulder);
            settle(scene);
        }
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        let ps2 = scene.tree();
        if read(&ps2, "asideHeading", "text") != "Found, 0 of 3"
            || read(&ps2, "firmwareStatus.0", "text") != "Not found"
            || read(&ps2, "firmwareStatus.1", "text") != "Not found · open replacement"
            || read(&ps2, "firmwareStatus.2", "text") != "Wrong size in your folder"
            || !shown(scene, "firmwareRing.0")
            || !shown(scene, "firmwareRing.1")
            || !shown(scene, "firmwareRing.2")
        {
            differences.push(format!("PS2 firmware states mismatch: heading={:?}, 0={:?}, 1={:?}, 2={:?}", read(&ps2, "asideHeading", "text"), read(&ps2, "firmwareStatus.0", "text"), read(&ps2, "firmwareStatus.1", "text"), read(&ps2, "firmwareStatus.2", "text")));
        }

        // Super Nintendo (M == 0): "No firmware needed" and file card hidden.
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        for _ in 0..5 {
            press(scene, sdl, pad, PadControl::LeftShoulder);
            settle(scene);
        }
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        let snes_fw = scene.tree();
        if read(&snes_fw, "asideHeading", "text") != "No firmware needed" || shown(scene, "asideFirmwareCard") {
            differences.push(format!("SNES M==0 firmware mismatch: heading={:?}, card shown={}", read(&snes_fw, "asideHeading", "text"), shown(scene, "asideFirmwareCard")));
        }

        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        scene.assert_quiet();
    });
}
