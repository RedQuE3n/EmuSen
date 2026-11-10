//! Settings ▸ Controllers in the program (EmuSen_Qt.md §7.7.H), its consoles' pads declared in a scripted world
//! (`settings_controllers.json`) and its pad an SDL virtual pad in this process: the section with the seats, the
//! bindings page with the drawing, a button rebound by a press on the pad and reset, and Keyboard keys and Hotkeys
//! taking a key, each state held at the gate's sizes and recorded for the comparison.

mod bindings;
mod common;

use bindings::{lines, named, open_settings, sentence, tabs};
use common::{hold, sandbox, wait_for};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::mockup;
use emusen_qt_test::virtual_pad::Sdl;

#[test]
fn qt_7_7_h_controllers_the_bindings_page_keys_and_hotkeys_hold_their_states_and_bind_what_is_pressed() {
    let _world = sandbox("qt-app-settings-controllers", &[("snes", "Hero.sfc")]);
    let sdl = Sdl::open();
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_controllers.json"), pad.id);
    let mut report = Vec::new();

    // The section: the seats counted, the pad on player 1 with the console's profile, as Mistress's recording.
    let mut waited = 0;
    while sentence(&mut scene) != "Player 1 · Simulated pad · Default profile" && waited < 3000 {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(sentence(&mut scene), "Player 1 · Simulated pad · Default profile");
    assert_eq!(lines(&mut scene), "Players 2 and 3 and 4 · none | Left and Right on a controller's row pick its player.");
    hold(&mut scene, "SettingsControllers", &mockup::Mockup::default(), named, &mut report);

    // The bindings page on the Super Nintendo, Y focused as the recording has it.
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "true", 1000);
    assert_eq!(tabs(&mut scene), "PLAYER 1, SUPER NINTENDO, SEGA CD, PLAYSTATION");
    assert_eq!(scene.property("controllerLayout", "generic"), "false", "the SNES pad's own drawing");
    scene.pad(UiButton::Left);
    wait_for(&mut scene, "settingsModel", "drawingFocus", "1", 1000);
    assert_eq!(scene.property("settingsModel", "caption"), "Y");
    assert_eq!(sentence(&mut scene), "Super Nintendo · Y");
    assert_eq!(lines(&mut scene), "Bound to the pad's West button. | A listens for a press and rebinds it; Y resets this button; the profile saves at once.");
    assert_eq!(scene.property("controllerLayout", "fits"), "true");
    hold(&mut scene, "SettingsBindings", &mockup::Mockup::default(), named, &mut report);

    // A listens; North pressed on the pad binds Y to it, and the interface hears nothing of the press.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "listeningPad", "true", 1000);
    sdl.press(&pad, 3, true);
    wait_for(&mut scene, "settingsModel", "listeningPad", "false", 2000);
    sdl.press(&pad, 3, false);
    scene.wait(200);
    assert!(lines(&mut scene).starts_with("Bound to the pad's North button."), "{}", lines(&mut scene));
    assert_eq!(scene.property("settingsModel", "bindingsShown"), "true", "the press stayed the binding's");
    // Y (the pad's North on the help bar) resets it.
    scene.pad(UiButton::Search);
    scene.wait(300);
    assert!(lines(&mut scene).starts_with("Bound to the pad's West button."), "{}", lines(&mut scene));
    // RB: the Sega CD's three-button pad, its own drawing.
    scene.pad(UiButton::PageDown);
    scene.wait(200);
    assert_eq!(tabs(&mut scene), "PLAYER 1, SEGA CD, PLAYSTATION, SUPER NINTENDO");
    assert_eq!(scene.property("controllerLayout", "name"), "Genesis 3-button controller");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "false", 1000);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);

    // Keyboard keys: A listens, F2 typed binds the console's first control to it.
    for _ in 0..5 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "6", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "true", 1000);
    assert!(tabs(&mut scene).starts_with("KEYBOARD, "), "{}", tabs(&mut scene));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "listening", "true", 1000);
    scene.key(0x0100_0031, 0);
    wait_for(&mut scene, "settingsModel", "listening", "false", 1000);
    assert!(lines(&mut scene).starts_with("Bound to the key F2."), "{}", lines(&mut scene));
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "false", 1000);

    // Hotkeys: A on Fast Forward listens, F3 typed is its key.
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "7", 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "listening", "true", 1000);
    scene.key(0x0100_0032, 0);
    wait_for(&mut scene, "settingsModel", "listening", "false", 1000);
    assert!(lines(&mut scene).starts_with("Bound to the key F3."), "{}", lines(&mut scene));
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    sdl.detach(pad);
    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}

