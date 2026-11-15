//! Eclipse's nav on the real program (EmuSen_Qt.md §3.4.8, R-20261115-7): the shoulders of an in-process SDL pad step through
//! every section the program has, each opening its screen, and the nav marks the one shown; a section tapped opens it; under
//! the reference theme the shoulders on Home open nothing. One test, as the harness is one per process.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::{PadControl, UiButton};
use emusen_qt::MAIN_URL;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_models::{screens, sections, themes};
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

/// The screen a section shows, as `Screens.shown` names it.
fn shown_of(section: &screens::Section) -> &'static str {
    match section.screen {
        screens::Shown::Home => "Home",
        screens::Shown::Consoles => "Consoles",
        other => panic!("{other:?}: a section this test does not know"),
    }
}

/// A shoulder pressed and let go as a hand does: down, then up, then held no more by the router before the next press.
fn shoulder(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, control: PadControl, button: UiButton) {
    let id = sdl_button(control).expect("an SDL button");
    sdl.press(pad, id, true);
    scene.wait(100);
    sdl.press(pad, id, false);
    for _ in 0..40 {
        scene.wait(25);
        if !scene.pad_held(button) {
            return;
        }
    }
    panic!("the router still holds {button:?}");
}

/// Which of the nav's sections is marked: the one whose underline is drawn.
fn marked(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    sections::all().iter().filter(|s| tree.nodes.iter().any(|n| n.name == format!("navUnderline.{}", s.key) && n.visible)).map(|s| s.key.to_owned()).collect()
}

fn back_home(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Home", 2000);
    scene.run_pad_clock();
    scene.wait(300);
}

#[test]
fn qt_3_4_8_the_shoulders_step_eclipse_s_sections_and_each_opens_its_screen() {
    let _world = sandbox("qt-app-eclipse-nav", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    scene.run_pad_clock();
    scene.wait(300);
    assert_eq!(scene.property("screens", "section"), "home", "Home is the program's first section");

    // Under the reference theme the shoulders open no section, and no nav is drawn.
    shoulder(&mut scene, &sdl, &pad, PadControl::RightShoulder, UiButton::PageDown);
    shoulder(&mut scene, &sdl, &pad, PadControl::LeftShoulder, UiButton::PageUp);
    scene.wait(300);
    assert_eq!(scene.property("screens", "shown"), "Home", "the reference's Home keeps its shoulders");
    assert!(marked(&mut scene).is_empty(), "the reference draws no nav");

    // Eclipse: its Home draws the nav, Home marked.
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    wait_for(&mut scene, "theme", "home", "true", 3000);
    scene.run_pad_clock();
    scene.wait(500);
    assert_eq!(scene.property("host.theme", "failed"), "false", "{}", scene.property("host.theme", "warnings"));
    assert_eq!(marked(&mut scene), ["home"], "the nav marks Home");

    // RB steps to each next section in turn, each opening its screen; Home's nav is drawn again marking Home when B returns.
    let all = sections::all();
    for (by, control, button) in [(1i32, PadControl::RightShoulder, UiButton::PageDown), (-1, PadControl::LeftShoulder, UiButton::PageUp)] {
        let next = &all[(by.rem_euclid(all.len() as i32)) as usize];
        shoulder(&mut scene, &sdl, &pad, control, button);
        wait_for(&mut scene, "screens", "shown", shown_of(next), 2000);
        assert_eq!(scene.property("screens", "shown"), shown_of(next), "{button:?} on Home opens {}", next.key);
        assert_eq!(scene.property("screens", "section"), next.key);
        back_home(&mut scene);
        assert_eq!(marked(&mut scene), ["home"], "back on Home, the nav marks Home");
    }

    // A section tapped opens its screen.
    let tree = scene.tree();
    let consoles = tree.find_named("nav.consoles").map(|i| tree.nodes[i as usize].clone()).expect("the nav's Consoles is drawn");
    scene.click(consoles.x + consoles.w / 2.0, consoles.y + consoles.h / 2.0);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    assert_eq!(scene.property("screens", "shown"), "Consoles", "tapping Consoles opens it");
    back_home(&mut scene);
    scene.assert_quiet();
}
