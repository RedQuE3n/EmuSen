// EmuSen_Kit.md §7.121: a look's type size is the kit's scaled size, raised by half units to the smallest whose capitals pass
// the fit audit's floor, and a size already above it is kept (R-20261115-14).

use emusen_kit_core::audit::{screen_unit, smallest_capitals, smallest_capitals_in_units, CAPS_SLACK, DESKTOP_CAPS, SIZES};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::tokens::{legible_text, scaled_text};

#[test]
fn kit_7_121_a_size_under_the_floor_is_raised_to_the_smallest_that_passes() {
    // A face whose capitals are Inter's share of its size, the floor's own face.
    let caps = |size: f64| size * DESKTOP_CAPS;
    let floor = smallest_capitals_in_units();
    for base in [10.0, 13.333_333_333_333_334, 14.0, 15.5, 16.0, 17.333_333_333_333_332, 74.666_666_666_666_67] {
        let size = legible_text(base, 1.0, caps);
        let scaled = scaled_text(base, 1.0);
        if caps(scaled) >= floor {
            assert_eq!(size, scaled, "{base}: above the floor, the design's size");
        } else {
            assert!(caps(size) >= floor && caps(size - 0.5) < floor, "{base}: raised to {size}, the smallest half unit that passes");
        }
    }
    // At a larger text scale the scaled size is the start.
    assert_eq!(legible_text(20.0, 1.3, caps), scaled_text(20.0, 1.3));
    // A face that reports no capitals keeps the design's size.
    assert_eq!(legible_text(10.0, 1.0, |_| 0.0), 10.0);
}

#[test]
fn kit_7_121_the_legible_size_passes_the_floor_at_every_gate_size() {
    // At each gate size, the capitals of the size the rule gives, in the window's pixels, against the audit's floor there.
    let caps = |size: f64| size * DESKTOP_CAPS;
    for base in [10.0, 13.333_333_333_333_334, 14.0, 16.0] {
        let size = legible_text(base, 1.0, caps);
        for (_, h) in SIZES {
            let h = f64::from(h);
            let drawn = caps(size) * h / DESIGN_HEIGHT;
            assert!(drawn >= smallest_capitals() * screen_unit(h) - CAPS_SLACK, "{base} at height {h}: capitals {drawn}");
        }
    }
}

#[test]
fn kit_7_121_a_theme_s_size_tokens_are_never_under_the_floor_on_the_big_screen() {
    use emusen_kit_core::tokens::{base_type, header_base, header_size, legible_header, legible_type, type_scale, Form, TypeScale};
    let caps = |size: f64| size * DESKTOP_CAPS;
    let floor = smallest_capitals_in_units();
    // Eclipse's tokens as theme.json gives them: the row at 16, the body at 14, the caption at 15.33, read off its boards.
    let eclipse = TypeScale { row: 16.0, body: 14.0, caption: 15.333_333_333_333_334, ..base_type(Form::BigScreen) };
    for scale in [1.0, 1.15, 1.3] {
        let drawn = legible_type(Form::BigScreen, &eclipse, scale, caps);
        for (given, size) in [(eclipse.row, drawn.row), (eclipse.body, drawn.body), (eclipse.caption, drawn.caption)] {
            assert!(caps(size.floor()) >= floor, "{given} at {scale}: {size}, drawn at {} by a Qt.font, is under the floor", size.floor());
            assert!(size == scaled_text(given, scale) || caps((size - 0.5).floor()) < floor, "{given} at {scale}: {size} is the smallest that passes");
        }
        // The reference's own sizes are above the floor and do not move, on the big screen and the desktop.
        for form in [Form::BigScreen, Form::Desktop] {
            assert_eq!(legible_type(form, &base_type(form), scale, caps), type_scale(form, scale), "{form:?} at {scale}");
        }
    }
    // A heading a theme sets under the floor is raised; the kit's own does not move.
    assert!(caps(legible_header(Form::BigScreen, 14.0, 1.0, caps).floor()) >= floor, "a heading at 14");
    for form in [Form::BigScreen, Form::Desktop] {
        assert_eq!(legible_header(form, header_base(form), 1.3, caps), header_size(form, 1.3), "{form:?}'s heading");
    }
    // The desktop is not held to the big screen's floor: its sizes are scaled alone.
    assert_eq!(legible_type(Form::Desktop, &eclipse, 1.0, caps), eclipse.scaled(1.0));
}
