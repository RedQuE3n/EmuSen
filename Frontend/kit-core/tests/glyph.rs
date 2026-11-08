// EmuSen_Kit.md §7.12: every control of every family has a glyph, inside its box, lettered and named as the pad prints it, and each press of the interface has its control.

use emusen_kit_core::glyph::{self, PadFamily};
use emusen_kit_core::input::{PadControl, UiButton};

#[test]
fn every_control_of_every_family_draws_something_inside_its_box() {
    for family in PadFamily::ALL {
        for control in PadControl::ALL {
            let g = glyph::glyph(family, control);
            assert!(!g.marks.is_empty(), "{family:?} {control:?} draws no mark");
            for mark in &g.marks {
                assert!(mark.points.len() >= 2, "{family:?} {control:?}: a mark of {} points", mark.points.len());
                assert!(mark.filled || mark.stroke > 0.0, "{family:?} {control:?}: a mark neither filled nor stroked");
                let reach = mark.stroke / 2.0;
                for p in &mark.points {
                    assert!(
                        p.iter().all(|v| *v >= reach - 1e-9 && *v <= 1.0 - reach + 1e-9),
                        "{family:?} {control:?}: {p:?} leaves the box"
                    );
                }
                if mark.filled {
                    assert_eq!(mark.points.first(), mark.points.last(), "{family:?} {control:?}: a filled mark is closed");
                }
            }
            assert!(g.marks.iter().filter(|m| m.cut).count() <= 1, "{family:?} {control:?}: one mark at most holds the letter");
            if g.marks.iter().any(|m| m.cut) {
                assert!(g.letter.is_some(), "{family:?} {control:?}: a cut with no letter");
            }
            assert!(!glyph::name(family, control).is_empty(), "{family:?} {control:?} has a name");
        }
    }
}

#[test]
fn face_buttons_are_lettered_and_named_by_position() {
    let letter = |f, c| glyph::glyph(f, c).letter.map(|l| l.text);
    assert_eq!(letter(PadFamily::Xbox, PadControl::South), Some("A"));
    assert_eq!(letter(PadFamily::Xbox, PadControl::North), Some("Y"));
    assert_eq!(letter(PadFamily::Nintendo, PadControl::South), Some("B"));
    assert_eq!(letter(PadFamily::Nintendo, PadControl::East), Some("A"));
    assert_eq!(letter(PadFamily::PlayStation, PadControl::South), None);
    assert_eq!(letter(PadFamily::Generic, PadControl::South), None);
    assert!(glyph::glyph(PadFamily::Nintendo, PadControl::West).marks[0].cut, "Nintendo's letter is cut out of its disc");
    assert!(glyph::glyph(PadFamily::Xbox, PadControl::West).marks.iter().all(|m| !m.cut), "Xbox's letter is drawn in its ring");
    assert_eq!(glyph::name(PadFamily::PlayStation, PadControl::South), "Cross");
    assert_eq!(glyph::name(PadFamily::PlayStation, PadControl::North), "Triangle");
    assert_eq!(glyph::name(PadFamily::Generic, PadControl::East), "East button");
    assert_eq!(glyph::name(PadFamily::Xbox, PadControl::Start), "Menu");
    assert_eq!(glyph::name(PadFamily::Nintendo, PadControl::Select), "Minus");
    assert_eq!(glyph::name(PadFamily::Nintendo, PadControl::LeftTrigger), "ZL");
    assert_eq!(glyph::name(PadFamily::Generic, PadControl::Up), "D-pad up");

    // The generic face: four dots, the meant one filled, at its own place.
    for control in [PadControl::South, PadControl::East, PadControl::West, PadControl::North] {
        let g = glyph::glyph(PadFamily::Generic, control);
        let filled: Vec<_> = g.marks.iter().filter(|m| m.filled).collect();
        assert_eq!((g.marks.len(), filled.len()), (4, 1), "{control:?}");
        let n = filled[0].points.len() as f64;
        let (x, y) = filled[0].points.iter().fold((0.0, 0.0), |(x, y), p| (x + p[0] / n, y + p[1] / n));
        let expected = match control {
            PadControl::South => (0.5, 0.79),
            PadControl::East => (0.79, 0.5),
            PadControl::West => (0.21, 0.5),
            _ => (0.5, 0.21),
        };
        assert!((x - expected.0).abs() < 0.01 && (y - expected.1).abs() < 0.01, "{control:?}'s dot at {x:.2},{y:.2}");
    }
}

#[test]
fn each_press_has_its_control_and_the_swap_trades_south_and_east_and_north_and_west() {
    assert_eq!(UiButton::Accept.control(false), PadControl::South);
    assert_eq!(UiButton::Back.control(false), PadControl::East);
    assert_eq!(UiButton::Accept.control(true), PadControl::East);
    assert_eq!(UiButton::Back.control(true), PadControl::South);
    assert_eq!(UiButton::Menu.control(true), PadControl::Start);
    assert_eq!(UiButton::Options.control(false), PadControl::Select);
    assert_eq!(UiButton::Up.control(false), PadControl::Up);
    assert_eq!(UiButton::Search.control(false), PadControl::North);
    assert_eq!(UiButton::Search.control(true), PadControl::West, "North and West trade too (EmuSen_Kit.md §2.2)");
    assert_eq!(UiButton::Screensaver.control(true), PadControl::North);
    for swap in [false, true] {
        let mut controls: Vec<PadControl> = UiButton::ALL.iter().map(|b| b.control(swap)).collect();
        controls.sort_by_key(|c| c.index());
        controls.dedup();
        assert_eq!(controls.len(), UiButton::ALL.len(), "two presses share a control (swap {swap})");
    }
    for (i, c) in PadControl::ALL.iter().enumerate() {
        assert_eq!(c.index(), i);
    }
}

#[test]
fn every_control_gives_back_the_press_whose_control_it_is_and_the_router_s_swap_restores_it() {
    for swap in [false, true] {
        for button in UiButton::ALL {
            let given = button.control(swap).press().map(|press| press.swapped(swap));
            assert_eq!(given, Some(button), "{button:?}'s control read back (swap {swap})");
        }
    }
    assert_eq!(PadControl::RightStick.press(), Some(UiButton::Random), "either stick's click is Random");
    let pressed: Vec<UiButton> = PadControl::ALL.iter().filter_map(|c| c.press()).collect();
    assert_eq!(pressed.len(), PadControl::ALL.len(), "every control gives a press");
}
