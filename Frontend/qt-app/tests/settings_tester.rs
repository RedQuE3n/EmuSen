//! Settings ▸ Controllers' input tester in the program (EmuSen_Qt.md §7.7.H), over the scripted world of
//! `settings_controllers.json` with an SDL virtual pad in this process: what is pressed on the pad or held on the
//! keyboard lit on the drawing, the stick's knob moved, the interface hearing nothing, and B held a second or Escape
//! leaving it.

mod bindings;
mod common;

use bindings::{lines, open_settings, tabs};
use common::{sandbox, wait_for};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::virtual_pad::Sdl;

#[test]
fn qt_7_7_h_the_tester_lights_what_is_pressed_moves_the_stick_and_leaves_on_b_held_a_second() {
    let _world = sandbox("qt-app-settings-tester", &[("snes", "Hero.sfc")]);
    let sdl = Sdl::open();
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_controllers.json"), pad.id);
    wait_for(&mut scene, "settingsModel", "focused", "0", 3000);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "true", 1000);
    // The PlayStation's DualShock, two shoulders on, for its sticks; Select runs the tester.
    scene.pad(UiButton::PageDown);
    scene.pad(UiButton::PageDown);
    assert_eq!(tabs(&mut scene), "PLAYER 1, PLAYSTATION, SUPER NINTENDO, SEGA CD");
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "settingsModel", "testing", "true", 1000);
    assert_eq!(lines(&mut scene), "Press anything on the pad or the keyboard. | Every press lights its button on the drawing and does nothing else. Hold B for a second, or press Escape, to stop.");

    // South on the pad lights its button and the interface hears nothing of it: no listening begins.
    sdl.press(&pad, 0, true);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "B", 2000);
    assert_eq!(scene.property("settingsModel", "listeningPad"), "false", "the press stayed the tester's");
    sdl.press(&pad, 0, false);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "", 2000);
    // A key held lights the control the console's keyboard map gives it.
    scene.key_hold(0x58, true);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "A", 1000);
    scene.key_hold(0x58, false);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "", 1000);
    // The left stick: under the deadzone its knob moves and nothing lights; past it, right lights.
    let rest = scene.property("controllerLayout", "picture");
    sdl.axis(&pad, 0, 13000);
    let mut waited = 0;
    while scene.property("controllerLayout", "picture") == rest && waited < 2000 {
        scene.wait(50);
        waited += 50;
    }
    assert_ne!(scene.property("controllerLayout", "picture"), rest, "the knob follows the stick");
    assert_eq!(scene.property("settingsModel", "drawingHeld"), "");
    sdl.axis(&pad, 0, 30000);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "LeftX+", 2000);
    sdl.axis(&pad, 0, 0);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "", 2000);

    // A short B is a button tried; B held a second leaves.
    sdl.press(&pad, 1, true);
    scene.wait(300);
    sdl.press(&pad, 1, false);
    scene.wait(1200);
    assert_eq!(scene.property("settingsModel", "testing"), "true", "a short B stays");
    sdl.press(&pad, 1, true);
    scene.wait(600);
    assert_eq!(scene.property("settingsModel", "testing"), "true", "not yet a second");
    wait_for(&mut scene, "settingsModel", "testing", "false", 2000);
    sdl.press(&pad, 1, false);
    scene.wait(200);
    assert_eq!(scene.property("settingsModel", "bindingsShown"), "true", "B's release does not leave the page");
    // Escape on the keyboard stops it too.
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "settingsModel", "testing", "true", 1000);
    scene.key(0x0100_0000, 0);
    wait_for(&mut scene, "settingsModel", "testing", "false", 1000);
    assert_eq!(scene.property("settingsModel", "bindingsShown"), "true");
    sdl.detach(pad);
    scene.assert_quiet();
}
