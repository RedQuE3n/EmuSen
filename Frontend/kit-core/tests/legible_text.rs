// EmuSen_Kit.md §7.120: a look's type size is the kit's scaled size, raised by half units to the smallest whose capitals pass
// the fit audit's floor, and a size already above it is kept (R-20261115-14).

use emusen_kit_core::audit::{smallest_capitals, CAPS_SLACK, DESKTOP_CAPS};
use emusen_kit_core::tokens::{legible_text, scaled_text};

#[test]
fn kit_7_120_a_size_under_the_floor_is_raised_to_the_smallest_that_passes() {
    // A face whose capitals are Inter's share of its size, the floor's own face.
    let caps = |size: f64| size * DESKTOP_CAPS;
    let floor = smallest_capitals() - CAPS_SLACK;
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
