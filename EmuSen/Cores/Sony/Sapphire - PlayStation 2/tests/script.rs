//! The press script the game runner and the bench share (`examples/common/script.rs`, Sapphire_Speed.md §51): which
//! buttons and stick values each frame holds.

#[path = "../examples/common/script.rs"]
mod script;

use script::Script;

#[test]
fn a_press_holds_its_buttons_from_its_frame_for_six_frames_or_its_count() {
    let s = Script::parse("press 100 14\npress 200 3+14 2\n# a note\nshot 150\n");
    assert_eq!((99..=106).map(|f| s.buttons(f)).collect::<Vec<_>>(), [0, 1 << 14, 1 << 14, 1 << 14, 1 << 14, 1 << 14, 1 << 14, 0]);
    assert_eq!([s.buttons(199), s.buttons(200), s.buttons(201), s.buttons(202)], [0, 1 << 3 | 1 << 14, 1 << 3 | 1 << 14, 0]);
}

#[test]
fn presses_that_overlap_are_held_together() {
    let s = Script::parse("press 10 14 10\npress 12 3 2\n");
    assert_eq!([s.buttons(11), s.buttons(12), s.buttons(13), s.buttons(14), s.buttons(19), s.buttons(20)], [1 << 14, 1 << 14 | 1 << 3, 1 << 14 | 1 << 3, 1 << 14, 1 << 14, 0]);
}

#[test]
fn a_stick_holds_its_value_and_the_later_begun_line_holds_an_axis() {
    let s = Script::parse("stick 50 1 -1.0 100\nstick 60 1 0.5 10\nstick 60 2 1.0\n");
    assert!(s.has_sticks());
    assert_eq!([s.axis(49, 1), s.axis(50, 1), s.axis(59, 1), s.axis(60, 1), s.axis(69, 1), s.axis(70, 1), s.axis(149, 1), s.axis(150, 1)], [0.0, -1.0, -1.0, 0.5, 0.5, -1.0, -1.0, 0.0]);
    assert_eq!([s.axis(59, 2), s.axis(60, 2), s.axis(65, 2), s.axis(66, 2)], [0.0, 1.0, 1.0, 0.0]);
    assert_eq!(s.axis(60, 0), 0.0, "an axis no line holds is centred");
    assert!(!Script::parse("press 1 14\nstick 1 4 1.0\n").has_sticks(), "an axis past the fourth is no stick");
}
