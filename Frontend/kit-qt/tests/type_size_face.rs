// EmuSen_Kit.md §7.121: a look's size is held to the legibility floor with the capitals of the face it draws in, measured by
// Qt: under Sora the smallest size whose Sora capitals reach the floor, which the application's face does not give.

use emusen_kit_core::audit::smallest_capitals_in_units;
use emusen_kit_core::tokens::scaled_text;
use emusen_kit_qt::type_size::{caps, caps_in, legible_in};

#[test]
fn kit_7_121_a_look_s_size_is_held_with_its_own_face_s_capitals() {
    let _harness = emusen_qt_test::Harness::new();
    let families = emusen_kit_qt::tokens::bundled_families();
    assert!(families.iter().any(|f| f == "Sora"), "Sora is bundled: {families:?}");
    let floor = smallest_capitals_in_units();
    // The two faces' capitals differ, so a floor held with one is not the other's.
    assert!((10..40).any(|s| (caps_in("Sora", 400, f64::from(s)) - caps(f64::from(s))).abs() > 0.01), "Sora's capitals are its own");
    for base in [8.0, 10.0, 13.333, 14.0, 16.0, 17.333, 18.667, 20.0, 74.667] {
        for weight in [300, 400, 600] {
            let size = legible_in(base, 1.0, "Sora", weight);
            assert!(caps_in("Sora", weight, size) >= floor, "{base} at {weight}: Sora's capitals at {size} reach the floor {floor}");
            let smaller = size - 0.5;
            assert!(
                smaller < scaled_text(base, 1.0) || caps_in("Sora", weight, smaller) < floor,
                "{base} at {weight}: {size} is the smallest size whose Sora capitals reach the floor"
            );
        }
    }
    // An empty family is the application's face, as `legible` holds it.
    assert_eq!(legible_in(14.0, 1.0, "", 400), emusen_kit_qt::type_size::legible(14.0, 1.0));
}
