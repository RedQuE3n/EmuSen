//! Two of the Controllers pages' failures made to happen in the program and found on the notice layer (EmuSen_Qt.md
//! §7.7.H6), on the software backend: the pads' reader given a platform library that will not load, and the binding
//! files' folder made read-only before a key is bound on the Keyboard keys page.

mod bindings;
mod common;

use bindings::open_settings;
use common::{sandbox, wait_for};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{Harness, Scene};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const WORLD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_controllers.json");

/// The notice layer showing words that begin `start`, within five seconds; the words.
fn said(scene: &mut Scene<'_>, start: &str) -> String {
    for _ in 0..100 {
        if scene.property("notices", "shown") == "true" && scene.property("notices", "words").starts_with(start) {
            return scene.property("notices", "words");
        }
        scene.wait(50);
    }
    panic!("never said \"{start}…\"; the notice layer reads \"{}\" (shown {})", scene.property("notices", "words"), scene.property("notices", "shown"));
}

/// Every folder under `root`, itself first.
fn folders(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    let mut at = 0;
    while at < out.len() {
        if let Ok(entries) = std::fs::read_dir(&out[at]) {
            out.extend(entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()));
        }
        at += 1;
    }
    out
}

fn pads_that_will_not_open(harness: &mut Harness) {
    let _world = sandbox("qt-app-controller-failures-pads", &[]);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("pads", "platformLibrary", "/nonexistent/libemusen_platform.so"));
    let words = said(&mut scene, "The controllers could not be read: ");
    assert!(words.contains("/nonexistent/libemusen_platform.so"), "{words}");
}

fn bindings_that_cannot_be_saved(sdl: &Sdl, harness: &mut Harness) {
    let world = sandbox("qt-app-controller-failures-save", &[("snes", "Hero.sfc")]);
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, pad.id);
    // Keyboard keys, its page, A listening.
    for _ in 0..6 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "6", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "true", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "listening", "true", 1000);
    // The configuration's folders read-only: F2 typed cannot be saved.
    let config = folders(&world.dir("Config"));
    config.iter().for_each(|d| std::fs::set_permissions(d, std::fs::Permissions::from_mode(0o555)).unwrap());
    scene.key(0x0100_0031, 0);
    let words = said(&mut scene, "The bindings could not be saved: ");
    config.iter().for_each(|d| std::fs::set_permissions(d, std::fs::Permissions::from_mode(0o755)).unwrap());
    assert!(words.len() > "The bindings could not be saved: ".len(), "the reason given: {words}");
    sdl.detach(pad);
}

/// One process, one Qt and one SDL: each failure over its own world and window.
#[test]
fn qt_7_7_h6_the_pads_unopened_and_the_bindings_unsaved_are_said() {
    let sdl = Sdl::open();
    let mut harness = Harness::new();
    pads_that_will_not_open(&mut harness);
    bindings_that_cannot_be_saved(&sdl, &mut harness);
}
