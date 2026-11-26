// EmuSen_Kit.md: Sheet uses NoAutoClose with Escape shortcut, Question handles synchronous Accept transitions cleanly and emits chosen(backIndex) once via onClosed fallback.

use emusen_kit_core::input::{Key, UiButton};
use emusen_kit_qt::qt_keys::qt_key;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/QuestionTransitionScene.qml");

fn prop(scene: &mut Scene<'_>, object: &str, name: &str) -> String {
    scene.property(object, name)
}

#[test]
fn question_escape_fallback_and_synchronous_step_transitions() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    assert!(scene.set_property("scene", "separator", "|"));
    assert!(scene.set_property("scene", "titleText", "Step 1"));
    assert!(scene.set_property("scene", "choicesJoined", "Cancel|Continue"));
    assert!(scene.set_property("scene", "stepTwoTitle", "Step 2"));
    assert!(scene.set_property("scene", "stepTwoChoices", "Back|Apply"));

    // Escape closes the sheet via NoAutoClose + Shortcut and emits chosen(backIndex) once.
    assert!(scene.set_property("scene", "showing", "true"));
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "true");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "true");
    scene.key(qt_key(Key::Escape).unwrap(), 0);
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "false");
    assert_eq!(prop(&mut scene, "question", "visible"), "false");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "false");
    assert_eq!(prop(&mut scene, "scene", "chosen"), "0");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "1");

    // UiButton::Back closes the sheet and clears sheetOpen cleanly.
    assert!(scene.set_property("scene", "chosen", "-2"));
    assert!(scene.set_property("scene", "chosenCount", "0"));
    assert!(scene.set_property("scene", "showing", "true"));
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "true");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "true");
    scene.pad(UiButton::Back);
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "false");
    assert_eq!(prop(&mut scene, "question", "visible"), "false");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "false");
    assert_eq!(prop(&mut scene, "scene", "chosen"), "0");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "1");

    // Programmatic close() while unanswered triggers the onClosed fallback once.
    assert!(scene.set_property("scene", "chosen", "-2"));
    assert!(scene.set_property("scene", "chosenCount", "0"));
    assert!(scene.set_property("scene", "showing", "true"));
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "true");
    assert!(scene.set_property("scene", "dismissExternally", "true"));
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "false");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "false");
    assert_eq!(prop(&mut scene, "scene", "chosen"), "0");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "1");
    assert!(scene.set_property("scene", "dismissExternally", "false"));

    // Synchronous Accept transition toggling showing: true -> false -> true leaves Step 2 open.
    assert!(scene.set_property("scene", "step", "1"));
    assert!(scene.set_property("scene", "chosen", "-2"));
    assert!(scene.set_property("scene", "chosenCount", "0"));
    assert!(scene.set_property("scene", "autoAdvance", "true"));
    assert!(scene.set_property("scene", "reopenViaToggle", "true"));
    assert!(scene.set_property("scene", "showing", "true"));
    scene.rest();
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.rest();
    assert_eq!(prop(&mut scene, "scene", "step"), "2");
    assert_eq!(prop(&mut scene, "scene", "chosen"), "1");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "1");
    assert_eq!(prop(&mut scene, "question", "opened"), "true");
    assert_eq!(prop(&mut scene, "question", "visible"), "true");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "true");
    assert_eq!(prop(&mut scene, "questionTitle", "text"), "Step 2");
    assert_eq!(prop(&mut scene, "choices", "currentIndex"), "0");

    // Step 2 remains operable and completes cleanly on Accept without extra fallback signals.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.rest();
    assert_eq!(prop(&mut scene, "scene", "chosen"), "1");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "2");
    assert_eq!(prop(&mut scene, "question", "opened"), "false");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "false");

    // Synchronous Accept transition keeping showing: true resets state and still handles Escape on Step 2.
    assert!(scene.set_property("scene", "step", "1"));
    assert!(scene.set_property("scene", "titleText", "Step 1b"));
    assert!(scene.set_property("scene", "choicesJoined", "No|Yes"));
    assert!(scene.set_property("scene", "chosen", "-2"));
    assert!(scene.set_property("scene", "chosenCount", "0"));
    assert!(scene.set_property("scene", "reopenViaToggle", "false"));
    assert!(scene.set_property("scene", "showing", "true"));
    scene.rest();
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.rest();
    assert_eq!(prop(&mut scene, "scene", "step"), "2");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "1");
    assert_eq!(prop(&mut scene, "question", "opened"), "true");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "true");
    assert_eq!(prop(&mut scene, "questionTitle", "text"), "Step 2");
    assert_eq!(prop(&mut scene, "choices", "currentIndex"), "0");
    scene.key(qt_key(Key::Escape).unwrap(), 0);
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "false");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "false");
    assert_eq!(prop(&mut scene, "scene", "chosen"), "0");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "2");

    // Step 2 after a non-toggling Accept transition also handles UiButton::Back cleanly.
    assert!(scene.set_property("scene", "step", "1"));
    assert!(scene.set_property("scene", "chosen", "-2"));
    assert!(scene.set_property("scene", "chosenCount", "0"));
    assert!(scene.set_property("scene", "showing", "true"));
    scene.rest();
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.rest();
    assert_eq!(prop(&mut scene, "scene", "step"), "2");
    assert_eq!(prop(&mut scene, "question", "opened"), "true");
    scene.pad(UiButton::Back);
    scene.rest();
    assert_eq!(prop(&mut scene, "question", "opened"), "false");
    assert_eq!(prop(&mut scene, "scene", "sheetOpen"), "false");
    assert_eq!(prop(&mut scene, "scene", "chosen"), "0");
    assert_eq!(prop(&mut scene, "scene", "chosenCount"), "2");

    scene.assert_quiet();
}
