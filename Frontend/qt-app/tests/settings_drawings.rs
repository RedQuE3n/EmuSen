//! Every declared controller's drawing on the bindings page at the gate's sizes (EmuSen_Qt.md §7.7.H, the port of
//! Mistress's `Every_consoles_bindings_page_fits`), each controller id of the in-tree cores declared by a console of a
//! scripted world (`settings_drawings.json`), and a synthetic core's controller with no drawing drawn generic.

mod bindings;
mod common;

use bindings::{named, open_settings, tabs};
use common::{hold, sandbox, wait_for};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::mockup;
use emusen_qt_test::virtual_pad::Sdl;

#[test]
fn qt_7_7_h_every_declared_controllers_drawing_fits_and_a_controller_with_none_is_drawn_generic() {
    let _world = sandbox("qt-app-settings-drawings", &[("snes", "Hero.sfc")]);
    let sdl = Sdl::open();
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_drawings.json"), pad.id);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "true", 1000);
    let mut report = Vec::new();
    let mut seen = Vec::new();
    for _ in 0..13 {
        let id = tabs(&mut scene).split(", ").nth(1).unwrap_or_default().to_lowercase();
        assert!(!seen.contains(&id), "{id} twice");
        let generic = scene.property("controllerLayout", "generic");
        assert_eq!(generic, if id == "test.pad" { "true" } else { "false" }, "{id}: only the controller with no drawing is drawn generic");
        assert_eq!(scene.property("controllerLayout", "fits"), "true", "{id}: every label beside the drawing");
        hold(&mut scene, &format!("Bindings-{}", id.replace('.', "-")), &mockup::Mockup::default(), named, &mut report);
        seen.push(id);
        scene.pad(UiButton::PageDown);
        scene.wait(200);
    }
    assert_eq!(seen.len(), 13);
    sdl.detach(pad);
    assert!(report.is_empty(), "{}", report.join("\n"));
}

