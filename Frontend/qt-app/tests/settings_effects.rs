//! Settings ▸ Controllers' rows taking effect in the program (EmuSen_Qt.md §7.7.H3), over the scripted world of
//! `settings_controllers.json` with SDL virtual pads in this process: each row changed on the screen, then its effect
//! seen where Mistress's was: the menus' A and B, the help bar's family, the pads the menus read, the tester's stick
//! threshold, the notice of a pad come and gone, and the running game's rumble switch.

mod bindings;
mod common;

use bindings::{focus_row as focus, open_settings, open_settings_reading};
use common::{sandbox, wait_for};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::virtual_pad::Sdl;

const WORLD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_controllers.json");


fn qt_7_7_h3_swap_a_and_b_trades_the_menus_a_and_b_on_the_pad(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-swap", &[("snes", "Hero.sfc")]);
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, pad.id);
    focus(&mut scene, "A and B trade");
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "settingsModel", "swapButtons", "true", 2000);
    assert_eq!(scene.property("reference", "swap"), "true", "the theme's router swaps too");
    // South on the pad is now B: Settings is left. The router's clock on the wall clock, as a real pad needs it.
    scene.run_pad_clock();
    sdl.press(&pad, 0, true);
    scene.wait(100);
    sdl.press(&pad, 0, false);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
}

fn qt_7_7_h3_controller_type_chooses_the_help_bar_s_buttons(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-type", &[("snes", "Hero.sfc")]);
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, pad.id);
    // An Xbox pad in hand, so Automatic draws the Xbox's buttons (Kit.Families: Xbox 1, PlayStation 2, Nintendo 3).
    assert_eq!(scene.property("program", "padFamily"), "1");
    focus(&mut scene, "Which buttons the help bar draws");
    for _ in 0..4 {
        if scene.property("settingsModel", "controllerFamily") == "2" {
            break;
        }
        scene.pad(UiButton::Right);
        scene.wait(100);
    }
    wait_for(&mut scene, "settingsModel", "controllerFamily", "2", 1000);
    wait_for(&mut scene, "program", "padFamily", "2", 1000);
    assert_eq!(scene.property("reference", "family"), "2", "the themes draw the chosen buttons too");
}

fn qt_7_7_h3_only_the_first_controller_leaves_the_menus_one_pad(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-first", &[("snes", "Hero.sfc")]);
    let one = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let two = sdl.attach("Second pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, one.id);
    focus(&mut scene, "Only the first controller connected");
    scene.pad(UiButton::Right);
    scene.wait(300);
    // A second pad connected now is not read by the menus.
    assert!(scene.set_property("program", "onlyPads", &format!("{} {}", one.id, two.id)));
    scene.wait(500);
    assert_eq!(scene.property("pads", "connected"), "1", "the menus read the first pad alone");
    // The second pad steers nothing: its Down does not move the focus.
    scene.run_pad_clock();
    let focused = scene.property("settingsModel", "focused");
    sdl.press(&two, 12, true);
    scene.wait(150);
    sdl.press(&two, 12, false);
    scene.wait(150);
    assert_eq!(scene.property("settingsModel", "focused"), focused, "the second pad is not read");
    // Off again by the first pad's Left, both pads are read.
    sdl.press(&one, 13, true);
    scene.wait(150);
    sdl.press(&one, 13, false);
    wait_for(&mut scene, "pads", "connected", "2", 2000);
}

fn qt_7_7_h3_controller_notices_tell_of_a_pad_come_and_gone_where_on(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-notices", &[("snes", "Hero.sfc")]);
    let one = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    // SDL numbers its virtual pads in turn, so the one attached next is read too.
    open_settings_reading(&mut scene, WORLD, &format!("{} {}", one.id, one.id + 1));
    assert_eq!(scene.property("notices", "shown"), "false", "the pads present at the start are not announced");
    // A pad of its own make, since SDL names a pad by the mapping its make first made.
    let two = sdl.attach("Second pad", 0x1209, 0x0002);
    assert_eq!(two.id, one.id + 1);
    wait_for(&mut scene, "notices", "words", "Second pad: player 2", 3000);
    sdl.detach(two);
    wait_for(&mut scene, "notices", "words", "Second pad disconnected: player 2 is kept for it", 3000);
    wait_for(&mut scene, "notices", "shown", "false", 6000);
    // Turned off, a pad's going says nothing.
    focus(&mut scene, "A notice at the top of the screen");
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "settingsModel", "controllerNotices", "false", 2000);
    sdl.detach(one);
    wait_for(&mut scene, "pads", "connected", "0", 3000);
    scene.wait(300);
    assert_eq!(scene.property("notices", "shown"), "false", "notices off");
}

fn qt_7_7_h3_stick_deadzone_moves_where_the_stick_counts_as_a_direction(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-deadzone", &[("snes", "Hero.sfc")]);
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, pad.id);
    // The deadzone stepped down from 50% to 30%.
    focus(&mut scene, "How far a stick must travel");
    for _ in 0..4 {
        scene.pad(UiButton::Left);
        scene.wait(100);
    }
    // The tester on the PlayStation's DualShock: the left stick at 0.4 of its travel lights right, past 30%.
    for _ in 0..40 {
        if scene.property("settingsModel", "focused") == "1" {
            break;
        }
        scene.pad(UiButton::Up);
        scene.wait(60);
    }
    scene.wait(300);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "bindingsShown", "true", 1000);
    scene.pad(UiButton::PageDown);
    scene.pad(UiButton::PageDown);
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "settingsModel", "testing", "true", 1000);
    sdl.axis(&pad, 0, 13107);
    wait_for(&mut scene, "settingsModel", "drawingHeld", "LeftX+", 2000);
    sdl.axis(&pad, 0, 0);
}

fn qt_7_7_h3_rumble_reaches_the_session(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-rumble", &[("snes", "Hero.sfc")]);
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, pad.id);
    wait_for(&mut scene, "session", "rumble", "true", 1000);
    focus(&mut scene, "Your controller shakes");
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "settingsModel", "rumble", "false", 2000);
    wait_for(&mut scene, "session", "rumble", "false", 1000);
}

fn qt_7_7_h6_guide_chords_changed_on_the_row_hold_without_a_restart(sdl: &Sdl, harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-chords", &[("snes", "Hero.sfc")]);
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    open_settings(&mut scene, WORLD, pad.id);
    // Auto away from Steam's Game Mode: on, for the game and the router.
    wait_for(&mut scene, "game", "guideChords", "true", 1000);
    focus(&mut scene, "Guide held with Up saves");
    // Auto, On, Off: two steps to Off.
    for _ in 0..2 {
        scene.pad(UiButton::Right);
        scene.wait(100);
    }
    wait_for(&mut scene, "settingsModel", "guideChords", "false", 2000);
    wait_for(&mut scene, "game", "guideChords", "false", 1000);
    scene.pad(UiButton::Left);
    wait_for(&mut scene, "game", "guideChords", "true", 1000);
}

/// One process, one Qt and one SDL: the rows in turn, each over its own world and window; those that run the router's
/// clock for a real pad's presses last, since the harness's own presses keep it stopped.
#[test]
fn qt_7_7_h3_the_controllers_rows_take_effect() {
    let sdl = Sdl::open();
    let mut harness = emusen_qt_test::Harness::new();
    qt_7_7_h3_controller_type_chooses_the_help_bar_s_buttons(&sdl, &mut harness);
    qt_7_7_h3_stick_deadzone_moves_where_the_stick_counts_as_a_direction(&sdl, &mut harness);
    qt_7_7_h3_rumble_reaches_the_session(&sdl, &mut harness);
    qt_7_7_h6_guide_chords_changed_on_the_row_hold_without_a_restart(&sdl, &mut harness);
    qt_7_7_h3_controller_notices_tell_of_a_pad_come_and_gone_where_on(&sdl, &mut harness);
    qt_7_7_h3_only_the_first_controller_leaves_the_menus_one_pad(&sdl, &mut harness);
    qt_7_7_h3_swap_a_and_b_trades_the_menus_a_and_b_on_the_pad(&sdl, &mut harness);
}
