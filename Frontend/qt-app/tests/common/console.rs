//! Walking a console started with nothing in it, as console.rs and esde.rs do (EmuSen_Qt.md §7.9): the card's rows by
//! the pad and the pointer, the console's screen owning the pad, and the chord's Pause Card.
#![allow(dead_code)]

use emusen_kit_core::input::{PadControl, UiButton};
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

/// Waits up to `seconds` for a condition the scene answers, letting Qt run between looks.
pub fn until(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        assert!(start.elapsed() < std::time::Duration::from_secs(seconds), "{what}: not within {seconds} s");
        scene.wait(8);
    }
}

/// The words of the item holding the focus.
pub fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

/// Down on a card until the focused row reads `words`, then A.
pub fn choose_row(scene: &mut Scene<'_>, words: &str) {
    for _ in 0..12 {
        if focused_words(scene).starts_with(words) {
            scene.pad(UiButton::Accept);
            return;
        }
        scene.pad(UiButton::Down);
    }
    panic!("no row reads {words}; the focus reads {}", focused_words(scene));
}

/// The centre of the shown card's row whose label reads `label`.
pub fn row_centre(scene: &mut Scene<'_>, label: &str) -> (f64, f64) {
    let tree = scene.tree();
    tree.nodes
        .iter()
        .filter(|n| n.visible && n.ty.starts_with("MenuRow"))
        .find(|r| tree.nodes.iter().any(|n| n.parent == r.id && n.name == "label" && n.text.as_deref() == Some(label)))
        .map(|r| (r.x + r.w / 2.0, r.y + r.h / 2.0))
        .unwrap_or_else(|| panic!("no row reads {label}"))
}

/// The console's screen up, its picture drawn, the pad the console's: B and Start, pressed on the pad, reach it and
/// not the program, which stays on the console's screen.
pub fn console_owns_the_pad(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, console: &str) {
    until(scene, 20, "the console running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true" && s.property("screens", "shown") == "Game"
    });
    assert_eq!((scene.property("session", "console"), scene.property("session", "game")), (console.to_owned(), String::new()));
    until(scene, 20, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    until(scene, 5, "the console owning the pad", |s| s.pad_chord_only());
    scene.run_pad_clock();
    for control in [PadControl::East, PadControl::Start] {
        let button = emusen_qt_endymion::reading::sdl_button(control).unwrap();
        sdl.press(pad, button, true);
        scene.wait(120);
        sdl.press(pad, button, false);
        scene.wait(120);
        assert_eq!((scene.property("screens", "shown"), scene.property("session", "paused")), ("Game".into(), "false".into()), "{control:?} is the console's");
    }
}

/// The chord's Pause Card over the console, named as Mistress named its session.
pub fn pause(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, named: &str) {
    let guide = emusen_qt_endymion::reading::sdl_button(PadControl::Guide).unwrap();
    until(scene, 5, "the console owning the pad", |s| s.pad_chord_only());
    scene.run_pad_clock();
    sdl.press(pad, guide, true);
    until(scene, 10, "the chord pausing the console under the Pause Card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    sdl.press(pad, guide, false);
    until(scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(named)), "the Pause Card names {named}");
}
