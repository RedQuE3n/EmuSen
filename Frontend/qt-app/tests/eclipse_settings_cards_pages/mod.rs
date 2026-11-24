//! The oracle of Eclipse's Settings on Memory cards, Saves on the cards, One save, Drawer and Delete, by pad and pointer (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{held, read, rows};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, centre, help_line, lines_held, pictures, settle, shown};
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

const WORLD_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/eclipse/settings_cards_world.json");
const CARDS_BOARD: &str = include_str!("../eclipse/settings_cards.tsv");
const CARDS_LINES: &str = include_str!("../eclipse/settings_cards_lines.tsv");
const SAVES_BOARD: &str = include_str!("../eclipse/settings_card_saves.tsv");
const SAVES_LINES: &str = include_str!("../eclipse/settings_card_saves_lines.tsv");
const SAVE_BOARD: &str = include_str!("../eclipse/settings_card_save.tsv");
const SAVE_LINES: &str = include_str!("../eclipse/settings_card_save_lines.tsv");

/// The program in a scripted PlayStation cards world, Eclipse on Settings -> Consoles -> PlayStation -> Memory cards.
fn with_cards(tag: &str, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad)) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new(tag);
    let mut psx = vec![0u8; 4096];
    psx[..8].copy_from_slice(b"PS-X EXE");
    std::fs::write(world.dir("Roms/psx").join("One.psexe"), &psx).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut stored = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    stored["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &stored).unwrap();
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
    press(&mut scene, &sdl, &pad, PadControl::South);
    settle(&mut scene);
    assert_eq!(at(&mut scene).1, "console:PlayStation");
    press(&mut scene, &sdl, &pad, PadControl::Down);
    press(&mut scene, &sdl, &pad, PadControl::Down);
    settle(&mut scene);
    press(&mut scene, &sdl, &pad, PadControl::South);
    settle(&mut scene);
    assert_eq!(at(&mut scene), (0, "cards:PlayStation".to_owned()), "opens PlayStation Memory cards");
    f(&mut scene, &sdl, &pad);
}

/// Checks a board's table, line metrics, glow, help bar and expected text across every gate size.
fn check_board(
    scene: &mut Scene<'_>,
    board_tsv: &str,
    lines_tsv: &str,
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
        if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse("#4A7890") {
            differences.push(format!("{at_size}: glow is {}", read(&tree, "backdropField.0", "paint")));
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
        let want_help = ["Value", "Change", "Back", "Home"];
        if help != want_help {
            differences.push(format!("{at_size}: help is {help:?} where board wants {want_help:?}"));
        }
    }
}

/// Boards #58 (Memory cards), #63 (Saves on the cards), #64 (One save), plus Drawer and Delete confirmation.
pub fn cards(backend: &str) {
    with_cards("qt-app-eclipse-settings-cards", |scene, sdl, pad| {
        let mut differences = Vec::new();

        // Board #58: Memory cards (Mode focused, 2-line wrapped Mode short line, 3 cards in drawer).
        let cards_words = vec![
            ("settingsCrumb".into(), "Consoles  ›  PlayStation".into()),
            ("settingsHeading".into(), "Memory cards".into()),
            ("settingsPlace".into(), "1 of 6".into()),
            ("asideHeading".into(), "Mode".into()),
            ("asideSentence".into(), "Named: every PlayStation game sees the same cards when they are shared, as the console did; per game gives each game its own.".into()),
            ("asideMuted.0".into(), "3 cards in the drawer".into()),
            ("rowName.0".into(), "Mode".into()),
            ("rowAbout.0".into(), "Per game, shared by every game as the console's were, or a drawer of named cards.".into()),
            ("rowWords.0".into(), "Named".into()),
            ("rowName.1".into(), "Slot 1".into()),
            ("rowAbout.1".into(), "The card in this slot when a game starts.".into()),
            ("rowWords.1".into(), "Card 1".into()),
            ("rowName.2".into(), "Slot 2".into()),
            ("rowAbout.2".into(), "The card in this slot when a game starts.".into()),
            ("rowWords.2".into(), "Card 2".into()),
            ("rowName.3".into(), "Cards".into()),
            ("rowAbout.3".into(), "Every card in the drawer, and the saves on each.".into()),
            ("rowWords.3".into(), "3 cards".into()),
            ("rowName.4".into(), "Saves on the cards".into()),
            ("rowAbout.4".into(), "Each card's saves, copied, moved or deleted.".into()),
            ("rowName.5".into(), "New card".into()),
            ("rowAbout.5".into(), "A blank card in the drawer, named for you.".into()),
            ("rowWords.5".into(), "…".into()),
        ];
        check_board(scene, CARDS_BOARD, CARDS_LINES, &cards_words, &[(0, 2), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)], &mut differences);
        pictures(scene, backend, "cards");

        // Stepping Mode Right/Left updates value and aside sentence; Drawer opens cleanly and keeps glow.
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "Shared" {
            differences.push(format!("Mode Right gave {:?}", read(&scene.tree(), "rowWords.0", "text")));
        }
        press(scene, sdl, pad, PadControl::Left);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "Named" {
            differences.push(format!("Mode Left gave {:?}", read(&scene.tree(), "rowWords.0", "text")));
        }
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        let drawer = scene.tree();
        if at(scene).1 != "drawer:PlayStation:psx.card"
            || read(&drawer, "settingsHeading", "text") != "Memory Cards"
            || read(&drawer, "rowName.0", "text") != "Card 1"
            || read(&drawer, "rowAbout.0", "text") != "A card in the drawer."
            || shown(scene, "rowDoor.0")
            || Colour::parse(&read(&drawer, "backdropField.0", "paint")) != Colour::parse("#4A7890")
        {
            differences.push(format!("Drawer page check failed: at={:?}, heading={:?}, door0={}", at(scene), read(&drawer, "settingsHeading", "text"), shown(scene, "rowDoor.0")));
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene), (3, "cards:PlayStation".to_owned()), "B returns to Cards row");

        // New card stays cleanly on Memory cards; Saves on the cards opens board #63.
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (5, "cards:PlayStation".to_owned()), "New card stays on page in scripted world");
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "saves:PlayStation:psx.card".to_owned()));

        // Board #63: Saves on the cards (Tower Set focused, 2-line wrapped Import short line, Card 1 aside).
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let saves_words = vec![
            ("settingsCrumb".into(), "Consoles  ›  PlayStation  ›  Memory cards".into()),
            ("settingsHeading".into(), "Memory Card".into()),
            ("settingsPlace".into(), "2 of 5".into()),
            ("asideHeading".into(), "Card 1".into()),
            ("asideSentence".into(), "Shared: Card 1 in Slot 1, Card 2 in Slot 2.".into()),
            ("asideMuted.0".into(), "3 saves, 11 of 15 blocks free.".into()),
            ("rowName.0".into(), "Card".into()),
            ("rowAbout.0".into(), "Left and Right step every card, the drawer's and each game's own.".into()),
            ("rowWords.0".into(), "Card 1".into()),
            ("rowName.1".into(), "Tower Set".into()),
            ("rowAbout.1".into(), "Choose a save to copy, move or delete it.".into()),
            ("rowWords.1".into(), "1 block".into()),
            ("rowName.2".into(), "Glass Orchard".into()),
            ("rowAbout.2".into(), "Choose a save to copy, move or delete it.".into()),
            ("rowWords.2".into(), "2 blocks".into()),
            ("rowName.3".into(), "Ninth Lantern".into()),
            ("rowAbout.3".into(), "Choose a save to copy, move or delete it.".into()),
            ("rowWords.3".into(), "1 block · damaged".into()),
            ("rowName.4".into(), "Import Every Game's Saves".into()),
            ("rowAbout.4".into(), "Every save on every game's own card copied onto the chosen card, all or none; each game's own card stays as it is.".into()),
        ];
        check_board(scene, SAVES_BOARD, SAVES_LINES, &saves_words, &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 2)], &mut differences);
        pictures(scene, backend, "card-saves");

        // Stepping Card cycles Card 2 (1 save), Card 3 (0 saves), One (game's own), and back to Card 1.
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        let c2 = scene.tree();
        if read(&c2, "rowWords.0", "text") != "Card 2" || read(&c2, "asideHeading", "text") != "Card 2" || read(&c2, "asideMuted.0", "text") != "1 save, 14 of 15 blocks free." {
            differences.push(format!("Card 2 state mismatch: val={:?}, heading={:?}, muted={:?}", read(&c2, "rowWords.0", "text"), read(&c2, "asideHeading", "text"), read(&c2, "asideMuted.0", "text")));
        }
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "asideMuted.0", "text") != "0 saves, 15 of 15 blocks free." {
            differences.push(format!("Card 3 muted is {:?}", read(&scene.tree(), "asideMuted.0", "text")));
        }
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        let own = scene.tree();
        if read(&own, "asideHeading", "text") != "One" || read(&own, "asideMuted.1", "text") != "One" {
            differences.push(format!("Game's own card mismatch: heading={:?}, muted1={:?}", read(&own, "asideHeading", "text"), read(&own, "asideMuted.1", "text")));
        }
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        assert_eq!(read(&scene.tree(), "rowWords.0", "text"), "Card 1");

        // Board #64: One save on a card (Tower Set).
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "save:PlayStation:psx.card:BASLUS-00101".to_owned()));
        let save_words = vec![
            ("settingsCrumb".into(), "Consoles  ›  PlayStation  ›  Memory cards  ›  Memory Card".into()),
            ("settingsHeading".into(), "Tower Set".into()),
            ("settingsPlace".into(), "1 of 4".into()),
            ("asideHeading".into(), "Tower Set".into()),
            ("asideSentence".into(), "Tower Set on Card 1.".into()),
            ("rowName.0".into(), "Copy or move to".into()),
            ("rowAbout.0".into(), "The card the save is copied or moved to.".into()),
            ("rowWords.0".into(), "Card 2".into()),
            ("rowName.1".into(), "Copy Save".into()),
            ("rowName.2".into(), "Move Save".into()),
            ("rowName.3".into(), "Delete Save…".into()),
        ];
        check_board(scene, SAVE_BOARD, SAVE_LINES, &save_words, &[(0, 1)], &mut differences);
        for i in 1..=3 {
            if shown(scene, &format!("rowAbout.{i}")) {
                differences.push(format!("row {i} on One save unexpectedly drew rowAbout"));
            }
        }
        if shown(scene, "asideMuted.0") {
            differences.push("One save unexpectedly drew asideMuted.0".into());
        }
        pictures(scene, backend, "card-save");

        // Target card stepping by pad and pointer chevron clicks.
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "Card 3" {
            differences.push(format!("Target Right gave {:?}", read(&scene.tree(), "rowWords.0", "text")));
        }
        let (x, y) = centre(scene, "rowLess.0");
        scene.click(x, y);
        settle(scene);
        if read(&scene.tree(), "rowWords.0", "text") != "Card 2" {
            differences.push(format!("Target chevron click gave {:?}", read(&scene.tree(), "rowWords.0", "text")));
        }

        // Delete confirmation page renders cleanly on the Settings frame with console glow.
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        let del = scene.tree();
        if at(scene) != (0, "delete:PlayStation:psx.card:BASLUS-00101".to_owned())
            || read(&del, "settingsCrumb", "text") != "Consoles  ›  PlayStation  ›  Memory cards  ›  Memory Card  ›  Tower Set"
            || read(&del, "settingsHeading", "text") != "Delete Save"
            || read(&del, "settingsPlace", "text") != "1 of 2"
            || read(&del, "asideHeading", "text") != "Delete Save"
            || read(&del, "asideSentence", "text") != "Delete Tower Set from Card 1? It cannot be brought back."
            || read(&del, "rowName.0", "text") != "No"
            || read(&del, "rowName.1", "text") != "Yes"
            || Colour::parse(&read(&del, "backdropField.0", "paint")) != Colour::parse("#4A7890")
        {
            differences.push(format!("Delete confirmation mismatch: at={:?}, heading={:?}, sentence={:?}", at(scene), read(&del, "settingsHeading", "text"), read(&del, "asideSentence", "text")));
        }
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene).1, "save:PlayStation:psx.card:BASLUS-00101", "No returns to save page");

        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        for _ in 0..5 {
            press(scene, sdl, pad, PadControl::East);
            settle(scene);
        }
        wait_for(scene, "screens", "shown", "Home", 2000);
        scene.assert_quiet();
    });
}
