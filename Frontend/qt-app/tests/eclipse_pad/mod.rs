//! The pad as a hand presses it, and the focus read back, for the oracles of Eclipse's screens (eclipse_consoles_oracle,
//! eclipse_games_oracle): an SDL pad inside the test's process, as the program reads one.
#![allow(dead_code)]

use emusen_kit_core::input::PadControl;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

/// SDL's trigger axes (SDL_GAMEPAD_AXIS_LEFT_TRIGGER and _RIGHT_TRIGGER) and a full pull.
const LEFT_TRIGGER: i32 = 4;
const RIGHT_TRIGGER: i32 = 5;
const PULLED: i16 = i16::MAX;

/// The name of the walkable element that holds the focus; empty for none.
pub fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// A control pressed and let go on the pad, as a hand does: down until the router holds it (or the screen it opens is
/// shown, whose frame holds nothing yet), up until it has let it go.
pub fn press(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, control: PadControl) {
    let button = control.press().expect("a control the interface hears");
    let shown = scene.property("screens", "shown");
    let set = |down: bool| match sdl_button(control) {
        Some(b) => sdl.press(pad, b, down),
        None => sdl.axis(pad, if control == PadControl::LeftTrigger { LEFT_TRIGGER } else { RIGHT_TRIGGER }, if down { PULLED } else { 0 }),
    };
    for down in [true, false] {
        set(down);
        let heard = |s: &mut Scene<'_>| s.pad_held(button) == down || (down && s.property("screens", "shown") != shown);
        let mut waited = 0;
        while !heard(scene) && waited < 2000 {
            scene.wait(8);
            waited += 8;
        }
        assert!(heard(scene), "the router heard {control:?} {}; shown {}, focus {}", if down { "down" } else { "up" }, scene.property("screens", "shown"), focused(scene));
    }
    scene.wait(60);
}
