//! Cores in the program (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7): reached from Home's place, over the world
//! Mistress's recordings script (qt-models/tests/cores), walked by the pad into the five recorded states, each held at
//! the four sizes with the gate's audit (fit, pad reach, pointer reach, no chrome) and recorded beside Mistress's.

mod common;

use common::{hold, hold_walking, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::WITH_SHOULDERS;
use emusen_qt::MAIN_URL;
use emusen_qt_test::excuse::Excuses;
use emusen_qt_test::{mockup, Scene};

const WORLDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qt-models/tests/cores");

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// The program on Home with Cores' world named, then Cores opened from its place.
fn open<'a>(harness: &'a mut emusen_qt_test::Harness, world: &str) -> Scene<'a> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "coresWorld", &format!("{WORLDS}/{world}")));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..4 {
        scene.pad(UiButton::Right);
    }
    assert_eq!(focused(&mut scene), "place.4");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Cores", 2000);
    wait_for(&mut scene, "coresModel", "busy", "false", 5000);
    scene.wait(200);
    scene
}

fn section(scene: &mut Scene<'_>, name: &str, by: UiButton) {
    for _ in 0..4 {
        if scene.property("coresModel", "section") == name {
            break;
        }
        scene.pad(by);
        scene.wait(50);
    }
    wait_for(scene, "coresModel", "section", name, 1000);
}

/// Up the list, then down it, until the pane names the core; the focus keeps its core across the shoulders.
fn down_to(scene: &mut Scene<'_>, core: &str) {
    for _ in 0..10 {
        if scene.property("coresModel", "paneTitle") == core || focused(scene) == "toolSearch" {
            break;
        }
        scene.pad(UiButton::Up);
        scene.wait(50);
    }
    for _ in 0..10 {
        if scene.property("coresModel", "paneTitle") == core {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(50);
    }
    wait_for(scene, "coresModel", "paneTitle", core, 1000);
}

/// X's page on the focused row, its first row chosen.
fn mark(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Screensaver);
    wait_for(scene, "coresModel", "cardOpen", "true", 1000);
    scene.wait(300);
    scene.pad(UiButton::Accept);
    wait_for(scene, "coresModel", "cardOpen", "false", 1000);
    scene.wait(300);
}

#[test]
fn qt_7_7_cores_walks_into_mistress_s_five_recorded_states() {
    let _world = sandbox("qt-app-cores", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut report = Vec::new();
    let mut excuses = Excuses::none();
    let none = mockup::Mockup::default();
    {
        let mut scene = open(&mut harness, "world.json");
        // Cores: the PlayStation 2 on the shoulders, LRPS2 focused, A naming its install.
        section(&mut scene, "PlayStation 2", UiButton::PageDown);
        down_to(&mut scene, "LRPS2");
        assert_eq!(scene.property("coresModel", "facts").matches("words").count(), 5);
        assert_eq!(scene.property("coresModel", "meterWords"), "/Cores 51 MB · 410 GB free");
        hold(&mut scene, "Cores", &none, &mut excuses, &mut report);

        // CoresDetail: All, the core waiting for approval focused, A asking to approve it.
        section(&mut scene, "All", UiButton::PageUp);
        down_to(&mut scene, "Extra PS2 Core");
        assert!(scene.property("coresModel", "help").contains("Approve…"));
        hold(&mut scene, "CoresDetail", &none, &mut excuses, &mut report);

        // CoresPlan: LRPS2 marked to install and Beetle PSX to update on X's pages, then Start's plan.
        section(&mut scene, "PlayStation 2", UiButton::PageDown);
        down_to(&mut scene, "LRPS2");
        mark(&mut scene);
        section(&mut scene, "PlayStation", UiButton::PageUp);
        down_to(&mut scene, "Beetle PSX");
        mark(&mut scene);
        wait_for(&mut scene, "coresModel", "summary", "2 marked: install 1, update 1, about 8 MB to download", 1000);
        section(&mut scene, "PlayStation 2", UiButton::PageDown);
        down_to(&mut scene, "LRPS2");
        scene.pad(UiButton::Menu);
        wait_for(&mut scene, "coresModel", "asking", "true", 1000);
        assert_eq!(scene.property("coresModel", "questionTitle"), "Apply 2 changes?");
        scene.wait(400);
        hold(&mut scene, "CoresPlan", &none, &mut excuses, &mut report);

        // CoresRunning: Yes, and the install runs and does not end.
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "asking", "false", 1000);
        wait_for(&mut scene, "coresModel", "summary", "Installing LRPS2: 18 MB of 41 MB", 2000);
        assert_eq!(scene.property("coresModel", "busy"), "true");
        scene.wait(400);
        hold(&mut scene, "CoresRunning", &none, &mut excuses, &mut report);

        // B goes Home.
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "screens", "shown", "Home", 2000);
    }
    {
        // CoresNoCatalogue: the runtime's four cores, and the fetch's refusal said.
        let mut scene = open(&mut harness, "no-catalogue.json");
        assert_eq!(scene.property("coresModel", "summary"), "Buildbot.libretro.com could not be reached.");
        hold(&mut scene, "CoresNoCatalogue", &none, &mut excuses, &mut report);

        // Search: Select's List card, its first row docks the keyboard; A types, X keeps the words, B deletes and closes.
        scene.pad(UiButton::Options);
        wait_for(&mut scene, "coresModel", "cardOpen", "true", 1000);
        scene.wait(300);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "searching", "true", 1000);
        scene.wait(300);
        let holder = focused(&mut scene);
        assert!(holder == "coresKeyboard" || holder.starts_with("key."), "the keyboard holds the focus: {holder}");
        // The gate's walk presses the shoulders too, which move the focus between the keys and the rows (R-20261111-31, -32).
        hold_walking(&mut scene, "CoresSearch", &none, &mut excuses, &mut report, Variant::Dark, &WITH_SHOULDERS);
        wait_for(&mut scene, "coresModel", "searchKeys", "true", 1000);
        // The right shoulder gives the rows the focus, the keyboard staying docked; the left gives it back to the keys.
        scene.pad(UiButton::PageDown);
        wait_for(&mut scene, "coresModel", "searchKeys", "false", 1000);
        scene.wait(100);
        let row = focused(&mut scene);
        assert!(row != "coresKeyboard" && !row.starts_with("key."), "a row holds the focus: {row}");
        assert_eq!(scene.property("coresKeyboard", "visible"), "true", "the keyboard stays docked");
        assert_eq!(scene.property("coresModel", "searching"), "true");
        let shelf = scene.property("coresModel", "section");
        scene.pad(UiButton::PageUp);
        wait_for(&mut scene, "coresModel", "searchKeys", "true", 1000);
        scene.wait(100);
        let holder = focused(&mut scene);
        assert!(holder == "coresKeyboard" || holder.starts_with("key."), "the keys hold the focus again: {holder}");
        assert_eq!(scene.property("coresModel", "section"), shelf, "the shoulder moved the focus, not the section");
        // The walk left the current key where its presses took it; Up to the top row, of letters and digits, then A types it.
        for _ in 0..5 {
            scene.pad(UiButton::Up);
        }
        scene.pad(UiButton::Accept);
        scene.wait(100);
        assert_eq!(scene.property("coresModel", "searchText").chars().count(), 1, "A typed the focused key");
        scene.pad(UiButton::Screensaver);
        wait_for(&mut scene, "coresModel", "searching", "false", 1000);
        scene.pad(UiButton::Options);
        wait_for(&mut scene, "coresModel", "cardOpen", "true", 1000);
        scene.wait(300);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "searching", "true", 1000);
        scene.wait(200);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "searchText", "", 1000);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "searching", "false", 1000);
        scene.wait(300);

        // Options: A on Diamond, its console's default, steps its option and keeps the page.
        section(&mut scene, "PlayStation", UiButton::PageDown);
        down_to(&mut scene, "Diamond");
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "cardTitle", "Diamond · Options", 1000);
        scene.wait(300);
        hold(&mut scene, "CoresOptions", &none, &mut excuses, &mut report);
        scene.pad(UiButton::Right);
        scene.wait(200);
        assert!(scene.property("coresModel", "cardRows").contains("Yours"), "{}", scene.property("coresModel", "cardRows"));
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "cardOpen", "false", 1000);
        scene.wait(300);

        // Firmware: X's page, its second row.
        scene.pad(UiButton::Screensaver);
        wait_for(&mut scene, "coresModel", "cardOpen", "true", 1000);
        scene.wait(300);
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "cardTitle", "Diamond · Firmware", 1000);
        scene.wait(300);
        assert!(scene.property("coresModel", "cardRows").contains("✓ Found"));
        hold(&mut scene, "CoresFirmware", &none, &mut excuses, &mut report);
        scene.pad(UiButton::Back);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "cardOpen", "false", 1000);
    }
    {
        // CoresUpdateAll: Select's List card, Update All…, its plan; Yes, and Beetle PSX's update runs to its end.
        let mut scene = open(&mut harness, "world.json");
        scene.pad(UiButton::Options);
        wait_for(&mut scene, "coresModel", "cardOpen", "true", 1000);
        scene.wait(300);
        let rows: serde_json::Value = serde_json::from_str(&scene.property("coresModel", "cardRows")).unwrap();
        let at = rows.as_array().unwrap().iter().position(|r| r["label"] == "Update All…").expect("Update All… on the List card");
        for _ in 0..at {
            scene.pad(UiButton::Down);
        }
        scene.wait(100);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "questionTitle", "Update Beetle PSX?", 1000);
        scene.wait(400);
        hold(&mut scene, "CoresUpdateAll", &none, &mut excuses, &mut report);
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "summary", "Installed Beetle PSX. Nothing downloaded runs until you approve it.", 3000);
        wait_for(&mut scene, "coresModel", "busy", "false", 3000);

        // Roll Back: PCEE2's page, the question, Yes, and the kept build put back.
        section(&mut scene, "PlayStation 2", UiButton::PageDown);
        down_to(&mut scene, "PCEE2");
        scene.pad(UiButton::Screensaver);
        wait_for(&mut scene, "coresModel", "cardOpen", "true", 1000);
        scene.wait(300);
        let rows: serde_json::Value = serde_json::from_str(&scene.property("coresModel", "cardRows")).unwrap();
        let at = rows.as_array().unwrap().iter().position(|r| r["label"] == "Roll Back…").expect("Roll Back… on PCEE2's page");
        for _ in 0..at {
            scene.pad(UiButton::Down);
        }
        scene.wait(100);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "questionTitle", "Roll Back PCEE2?", 1000);
        scene.wait(300);
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "summary", "Rolled back PCEE2 to the build of 24 October 2026.", 3000);
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}
