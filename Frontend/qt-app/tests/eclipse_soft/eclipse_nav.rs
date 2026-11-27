//! Eclipse's nav on the real program (EmuSen_Qt.md §3.4.8, R-20261115-7): from Home and from Choose a console the shoulders
//! of an in-process SDL pad step to the sections either side and a tapped section opens, each opening the program's screen
//! for it, B coming back; the nav marks the section shown; under the reference theme the shoulders on Home open nothing.
//! One test, as the harness is one per process.

use crate::common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::{PadControl, UiButton};
use emusen_qt::MAIN_URL;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_models::{library as games, sections, themes};
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

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

const LB: (PadControl, UiButton) = (PadControl::LeftShoulder, UiButton::PageUp);
const RB: (PadControl, UiButton) = (PadControl::RightShoulder, UiButton::PageDown);

/// Which of the nav's sections is marked: the one whose underline is drawn.
fn marked(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    sections::all().iter().filter(|s| tree.nodes.iter().any(|n| n.name == format!("navUnderline.{}", s.key) && n.visible)).map(|s| s.key.to_owned()).collect()
}

/// The screen settled: the pad's clock run and the theme's screen placed.
fn settle(scene: &mut Scene<'_>) {
    scene.run_pad_clock();
    scene.wait(400);
}

/// B, waiting for the screen it returns to.
fn back(scene: &mut Scene<'_>, to: &str) {
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", to, 2000);
    settle(scene);
}

/// The section drawn on the nav, tapped in its middle.
fn tap(scene: &mut Scene<'_>, key: &str) {
    let tree = scene.tree();
    let n = tree.nodes.iter().rev().find(|n| n.name == format!("nav.{key}") && n.visible).cloned().unwrap_or_else(|| panic!("the nav's {key} is drawn"));
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

/// Where a section left the program: the screen shown, the grid's key, the Home card's page, and the List card's page.
fn at(scene: &mut Scene<'_>) -> [String; 4] {
    ["shown", "key", "page", "listPage"].map(|p| scene.property("screens", p))
}

#[test]
fn qt_3_4_8_the_shoulders_and_a_tap_open_eclipse_s_sections_from_home_and_consoles() {
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
    settle(&mut scene);
    assert_eq!(scene.property("screens", "section"), "home", "Home is the program's first section");

    // Under the reference theme the shoulders open no section, and no nav is drawn.
    for (control, button) in [RB, LB] {
        shoulder(&mut scene, &sdl, &pad, control, button);
    }
    scene.wait(300);
    assert_eq!(scene.property("screens", "shown"), "Home", "the reference's Home keeps its shoulders");
    assert!(marked(&mut scene).is_empty(), "the reference draws no nav");

    // Eclipse: its Home draws the nav, Home marked.
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    wait_for(&mut scene, "theme", "home", "true", 3000);
    settle(&mut scene);
    assert_eq!(scene.property("host.theme", "failed"), "false", "{}", scene.property("host.theme", "warnings"));
    assert_eq!(marked(&mut scene), ["home"], "the nav marks Home");

    let all = |s: &str| s.to_owned();
    // Home's LB: Tools hub; B back to Home.
    shoulder(&mut scene, &sdl, &pad, LB.0, LB.1);
    wait_for(&mut scene, "screens", "shown", "Tools", 2000);
    settle(&mut scene);
    assert_eq!(marked(&mut scene), ["tools"], "LB on Home opens Tools");
    back(&mut scene, "Home");
    assert_eq!(marked(&mut scene), ["home"], "back on Home, the nav marks Home");

    // Home's RB: Library, the games view at every game; RB goes to Consoles, B returns to Home.
    shoulder(&mut scene, &sdl, &pad, RB.0, RB.1);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    settle(&mut scene);
    assert_eq!(at(&mut scene)[..2], [all("Grid"), all(games::ALL)], "RB on Home opens Library: every game");
    assert_eq!(scene.property("screens", "section"), "library");
    assert_eq!(marked(&mut scene), ["library"], "Library's nav marks Library");
    back(&mut scene, "Home");
    assert_eq!(marked(&mut scene), ["home"], "B from top-level Library goes Home");
    shoulder(&mut scene, &sdl, &pad, RB.0, RB.1);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    settle(&mut scene);
    shoulder(&mut scene, &sdl, &pad, RB.0, RB.1);
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    settle(&mut scene);
    assert_eq!(scene.property("screens", "section"), "consoles");
    assert_eq!(marked(&mut scene), ["consoles"], "Choose a console's nav marks Consoles");

    // Choose a console's RB: Collections hub; LB back to Consoles.
    shoulder(&mut scene, &sdl, &pad, RB.0, RB.1);
    wait_for(&mut scene, "screens", "shown", "Collections", 2000);
    settle(&mut scene);
    assert_eq!(marked(&mut scene), ["collections"], "RB on Consoles opens Collections");
    shoulder(&mut scene, &sdl, &pad, LB.0, LB.1);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    settle(&mut scene);

    // Choose a console's LB: Library; RB back to Consoles.
    shoulder(&mut scene, &sdl, &pad, LB.0, LB.1);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    settle(&mut scene);
    assert_eq!(at(&mut scene)[..2], [all("Grid"), all(games::ALL)], "LB on Consoles opens Library");
    assert_eq!(marked(&mut scene), ["library"], "LB from Consoles marks Library");
    shoulder(&mut scene, &sdl, &pad, RB.0, RB.1);
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    settle(&mut scene);

    // A tap: Tools from Choose a console, Home on Back, Collections tapped, and Consoles from Home.
    tap(&mut scene, "tools");
    wait_for(&mut scene, "screens", "shown", "Tools", 2000);
    settle(&mut scene);
    assert_eq!(marked(&mut scene), ["tools"], "Tools tapped on Consoles");
    tap(&mut scene, "collections");
    wait_for(&mut scene, "screens", "shown", "Collections", 2000);
    settle(&mut scene);
    assert_eq!(marked(&mut scene), ["collections"], "Collections tapped on Tools");
    back(&mut scene, "Home");
    tap(&mut scene, "consoles");
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    settle(&mut scene);
    assert_eq!(marked(&mut scene), ["consoles"], "Consoles tapped on Home");
    tap(&mut scene, "home");
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    settle(&mut scene);
    assert_eq!(marked(&mut scene), ["home"], "Home tapped on Consoles");
    // The section shown, tapped, stays.
    tap(&mut scene, "home");
    scene.wait(400);
    assert_eq!(scene.property("screens", "shown"), "Home", "the section shown stays");
    scene.assert_quiet();
}

#[test]
fn qt_3_4_10_question_eclipse_rearms_multistep_and_reopens_on_showing_back() {
    let dir = std::env::temp_dir().join(format!("emusen-question-eclipse-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let qml = dir.join("QuestionScene.qml");
    let _link = MAIN_URL;
    std::fs::write(
        &qml,
        r#"import QtQuick
import EmuSen.App
import EmuSen.Kit as Kit

Window {
    id: root
    objectName: "scene"
    width: 1280
    height: 800
    visible: true
    property int step: 0
    property int picked: -99
    property bool showing: false
    property bool keepShowingOnBack: false
    Kit.Frame {
        id: frame
        objectName: "frame"
        anchors.fill: parent
        focus: true
        QuestionHost {
            id: host
            objectName: "host"
            controller: frame.controller
            title: "Confirm action"
            sentences: root.step === 0 ? ["Step 1"] : ["Step 2"]
            choices: root.step === 0 ? ["First", "Next"] : ["Keep", "Done"]
            backIndex: root.step === 0 ? 0 : 1
            showing: root.showing
            onChosen: (index) => {
                root.picked = index
                if (root.step === 0)
                    root.step = 1
                else if (index === 1 && !root.keepShowingOnBack)
                    root.showing = false
            }
        }
    }
}
"#,
    )
    .unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(&format!("file://{}", qml.display()), 1280, 800);
    scene.kit_tokens().expect("kit tokens").set_overrides(vec![(emusen_kit_core::tokens::Variant::Dark, "coronaRim".into(), "2".into())]);
    wait_for(&mut scene, "host", "eclipse", "true", 2000);
    assert!(scene.set_property("scene", "showing", "true"));
    wait_for(&mut scene, "host", "opened", "true", 2000);
    // Advance selection off index 0 then choose item 1; showing stays true with the same title.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "scene", "step", "1", 1000);
    assert_eq!(scene.property("scene", "picked"), "1");
    scene.wait(100);
    // Step 2 must reset choiceList to 0 while keeping showing true.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "scene", "picked", "0", 1000);
    scene.wait(100);
    // Keeping showing true on Back (backIndex = 1) must re-open cleanly rather than wedging the popup closed.
    assert!(scene.set_property("scene", "keepShowingOnBack", "true"));
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "scene", "picked", "1", 1000);
    wait_for(&mut scene, "host", "opened", "true", 2000);
    assert!(scene.set_property("scene", "keepShowingOnBack", "false"));
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "scene", "showing", "false", 1000);
    scene.assert_quiet();
    let _ = std::fs::remove_dir_all(&dir);
}
