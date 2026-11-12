//! An unchanged screen is not re-rendered (EmuSen_Qt.md §9.3, §9.3.2): Home, Choose a console and the grid swap no frame
//! once the focus has settled, and the sky's glow moves only while the focus moves. A failure says what the program held
//! then: the screen, the models' problems, the focus.

mod common;

use common::{assert_still_saying, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;

/// A press, then frames while the glow eases to the new focus, then none once it settles.
fn moves_then_stills(scene: &mut emusen_qt_test::Scene<'_>, button: UiButton, what: &str) {
    let before = scene.frames();
    scene.pad(button);
    scene.wait(100);
    let sky = scene.running();
    assert!(scene.frames() > before && !sky.is_empty(), "{what}: the glow eases after the press ({sky:?})");
    assert_still_saying(scene, what);
}

#[test]
fn the_screens_render_nothing_once_the_focus_settles() {
    let _world = sandbox("qt-app-idle", &[("NES", "One.nes"), ("NES", "Two.nes"), ("GB", "Three.gb")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert_eq!(scene.property("screens", "shown"), "Home");
    assert_still_saying(&mut scene, "Home");

    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("screens", "shown"), "Consoles");
    assert_still_saying(&mut scene, "Choose a console");
    moves_then_stills(&mut scene, UiButton::Up, "Choose a console, Up to the nearest place");
    moves_then_stills(&mut scene, UiButton::Down, "Choose a console, Down to the nearest console");
    moves_then_stills(&mut scene, UiButton::Right, "Choose a console, Right along the orbit");

    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    assert_still_saying(&mut scene, "the grid");
    scene.pad(UiButton::Right);
    assert_still_saying(&mut scene, "the grid after Right");
    scene.assert_quiet();
}
