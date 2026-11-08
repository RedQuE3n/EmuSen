// EmuSen_Kit.md §7.12: every control of every family has a glyph, inside its box, lettered and named as the pad prints it, and each press of the interface has its control.

use emusen_kit_core::glyph::{self, GlyphStyle, PadFamily};
use emusen_kit_core::world::Point;
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

fn inside(p: Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    for pair in polygon.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    inside
}

// EmuSen_Kit.md §7.28: the filled set, one solid shape a control with its letter or its mark cut out, in every family.
#[test]
fn kit_7_28_every_control_of_every_family_draws_a_filled_shape_with_a_cut_mark() {
    for family in PadFamily::ALL {
        let mut drawn = Vec::new();
        for control in PadControl::ALL {
            let g = glyph::styled(family, control, GlyphStyle::Filled);
            assert_eq!(g.marks.len(), 1, "{family:?} {control:?}: one solid shape");
            let m = &g.marks[0];
            assert!(m.filled && m.stroke == 0.0, "{family:?} {control:?}: filled, not stroked");
            assert_eq!(m.points.first(), m.points.last(), "{family:?} {control:?}: the shape is closed");
            assert!(m.points.iter().flatten().all(|v| (0.0..=1.0).contains(v)), "{family:?} {control:?} leaves the box");
            assert!(m.cut != !m.holes.is_empty(), "{family:?} {control:?}: a letter or holes cut out, one of the two");
            assert_eq!(m.cut, g.letter.is_some(), "{family:?} {control:?}: a cut letter and a letter go together");
            assert!(g.letter.is_none_or(|l| l.bold), "{family:?} {control:?}: a letter cut out of a filled shape is bold");
            for hole in &m.holes {
                assert!(hole.len() >= 4 && hole.first() == hole.last(), "{family:?} {control:?}: a hole is a closed polygon");
                assert!(hole.iter().all(|p| inside(*p, &m.points)), "{family:?} {control:?}: a hole leaves its shape");
            }
            let name = glyph::name(glyph::lettered(family, GlyphStyle::Filled), control);
            assert!(!name.is_empty(), "{family:?} {control:?} has a name");
            drawn.push((control, g));
        }
        for (i, (a, ga)) in drawn.iter().enumerate() {
            for (b, gb) in &drawn[i + 1..] {
                let sticks = matches!((a, b), (PadControl::LeftStick, PadControl::RightStick));
                assert!(sticks || ga != gb, "{family:?}: {a:?} and {b:?} draw the same glyph");
            }
        }
    }
}

#[test]
fn kit_7_28_an_unknown_pad_is_lettered_as_an_xbox_pad_in_the_filled_set_only() {
    assert_eq!(glyph::lettered(PadFamily::Generic, GlyphStyle::Filled), PadFamily::Xbox);
    assert_eq!(glyph::lettered(PadFamily::Generic, GlyphStyle::Outlined), PadFamily::Generic);
    assert_eq!(glyph::lettered(PadFamily::Nintendo, GlyphStyle::Filled), PadFamily::Nintendo);
    let filled = |f, c| glyph::styled(f, c, GlyphStyle::Filled);
    assert_eq!(filled(PadFamily::Generic, PadControl::South), filled(PadFamily::Xbox, PadControl::South));
    assert_eq!(filled(PadFamily::Xbox, PadControl::South).letter.map(|l| l.text), Some("A"));
    assert_eq!(filled(PadFamily::Nintendo, PadControl::South).letter.map(|l| l.text), Some("B"), "Nintendo letters by position");
    assert_eq!(filled(PadFamily::PlayStation, PadControl::South).letter, None, "PlayStation's shapes are cut, not lettered");
    assert_eq!(filled(PadFamily::Xbox, PadControl::RightTrigger).letter.map(|l| l.text), Some("RT"));
    for family in PadFamily::ALL {
        for control in PadControl::ALL {
            assert_eq!(glyph::styled(family, control, GlyphStyle::Outlined), glyph::glyph(family, control), "the outlined set is unchanged");
        }
    }
}
