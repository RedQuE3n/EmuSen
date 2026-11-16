//! EmuSen_Kit.md §7.121: under Eclipse, whose size tokens were read off its boards, no word of the program's own Settings
//! or Themes tab is under the legibility floor at any gate size, by the kit's gate's own rule.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::audit::Fault;
use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;

/// The words under the legibility floor in what the window shows now, at every gate size: the kit's gate's own rule
/// (`Fault::TooSmall`), each as `size where: name "words"`.
fn too_small(scene: &mut Scene<'_>, at: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (w, h) in emusen_qt_test::harness::SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let tree = scene.tree();
        let focus = scene.focus(&tree);
        let edges = scene.record_edges(&tree);
        for fault in scene.audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &edges, f64::from(w), f64::from(h)) {
            if let Fault::TooSmall { item, capitals } = fault {
                let n = &tree.nodes[item.0 as usize];
                found.push(format!("{w}x{h} {at}: {} {:?} at {} px, capitals {capitals:.2}", n.name, n.text.as_deref().unwrap_or(""), n.size));
            }
        }
    }
    let (w, h) = emusen_qt_test::harness::SIZES[0];
    scene.resize(w, h);
    scene.wait(150);
    found
}

#[test]
fn kit_7_121_under_eclipse_no_word_of_the_program_s_settings_is_under_the_floor() {
    let _world = sandbox("qt-app-eclipse-floor", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let (w, h) = emusen_qt_test::harness::SIZES[0];
    let mut scene = harness.load(MAIN_URL, w, h);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..20 {
        let tree = scene.tree();
        if scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).as_deref() == Some("place.0") {
            break;
        }
        scene.wait(50);
    }
    // Settings ▸ Library ▸ Themes, Eclipse used, as qt_3_4_7 does.
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    for _ in 0..4 {
        scene.pad(UiButton::PageDown);
    }
    wait_for(&mut scene, "settingsModel", "section", "4", 1000);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "2", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Themes", 2000);
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 5000);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "themesModel", "name", "Eclipse", 2000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    wait_for(&mut scene, "themesModel", "findings", "In use.", 2000);

    // The Themes tab, then each of Settings' six sections, at every gate size.
    let mut found = too_small(&mut scene, "Themes");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    for _ in 0..4 {
        scene.pad(UiButton::PageUp);
    }
    wait_for(&mut scene, "settingsModel", "section", "0", 1000);
    for section in 0..6 {
        wait_for(&mut scene, "settingsModel", "section", &section.to_string(), 1000);
        found.extend(too_small(&mut scene, &format!("Settings.{section}")));
        scene.pad(UiButton::PageDown);
    }
    assert!(found.is_empty(), "{} words under the floor under Eclipse:\n{}", found.len(), found.join("\n"));
}
