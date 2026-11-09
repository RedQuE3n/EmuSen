//! Cores in the program (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7): reached from Home's place, over the world
//! Mistress's recordings script (qt-models/tests/cores), walked by the pad into the five recorded states, each held at
//! the four sizes with the gate's audit (fit, pad reach, pointer reach, no chrome) and recorded beside Mistress's.

mod common;

use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};

const WORLDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qt-models/tests/cores");

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// The differences the tester has been told of, which a held state does not count as faults (EmuSen_Qt.md §7.7).
fn named(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    // Under a question the screen beneath is not walked: the question is modal, as Mistress's plan sheet is.
    let asked = tree.nodes.iter().any(|n| n.ty == "QQuickPopupItem" && n.visible);
    (size == (1280, 720) && fault.contains("too small to read")) || (asked && fault.contains("the pad does not reach")) || scrolled_out(fault, tree)
}

/// An overlap with a row the list has scrolled wholly out of its box, which Qt culls and so draws nowhere: the audit
/// reads the row's box, not its list's clip (EmuSen_Qt.md §7.7.F).
fn scrolled_out(fault: &str, tree: &emusen_qt_test::tree::Tree) -> bool {
    let Some(pair) = fault.strip_prefix("gate: overlap: ") else { return false };
    let Some(list) = tree.nodes.iter().find(|n| n.name == "toolRows") else { return false };
    let inside = |n: &emusen_qt_test::tree::Node| {
        let mut at = Some(n);
        while let Some(node) = at {
            if node.id == list.id {
                return true;
            }
            at = tree.nodes.iter().find(|p| p.id == node.parent);
        }
        false
    };
    pair.split(" and ").filter_map(|i| i.trim().parse::<usize>().ok()).filter_map(|i| tree.nodes.get(i)).any(|n| {
        inside(n) && n.id != list.id && (n.y + n.h <= list.y + 1.0 || n.y >= list.y + list.h - 1.0)
    })
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
    let none = mockup::Mockup::default();
    {
        let mut scene = open(&mut harness, "world.json");
        // Cores: the PlayStation 2 on the shoulders, LRPS2 focused, A naming its install.
        section(&mut scene, "PlayStation 2", UiButton::PageDown);
        down_to(&mut scene, "LRPS2");
        assert_eq!(scene.property("coresModel", "facts").matches("words").count(), 5);
        assert_eq!(scene.property("coresModel", "meterWords"), "/Cores 51 MB · 410 GB free");
        hold(&mut scene, "Cores", &none, named, &mut report);

        // CoresDetail: All, the core waiting for approval focused, A asking to approve it.
        section(&mut scene, "All", UiButton::PageUp);
        down_to(&mut scene, "Extra PS2 Core");
        assert!(scene.property("coresModel", "help").contains("Approve…"));
        hold(&mut scene, "CoresDetail", &none, named, &mut report);

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
        hold(&mut scene, "CoresPlan", &none, named, &mut report);

        // CoresRunning: Yes, and the install runs and does not end.
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "asking", "false", 1000);
        wait_for(&mut scene, "coresModel", "summary", "Installing LRPS2: 18 MB of 41 MB", 2000);
        assert_eq!(scene.property("coresModel", "busy"), "true");
        scene.wait(400);
        hold(&mut scene, "CoresRunning", &none, named, &mut report);

        // B goes Home.
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "screens", "shown", "Home", 2000);
    }
    {
        // CoresNoCatalogue: the runtime's four cores, and the fetch's refusal said.
        let mut scene = open(&mut harness, "no-catalogue.json");
        assert_eq!(scene.property("coresModel", "summary"), "Buildbot.libretro.com could not be reached.");
        hold(&mut scene, "CoresNoCatalogue", &none, named, &mut report);
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}
