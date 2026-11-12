// EmuSen_Kit.md §7.105: the kit's page frame stands without a host, so it keeps a cover shape of its own until one is given;
// that default is kit-core's COVER_SHAPE, read here from the QML so the two cannot drift.

use emusen_kit_core::page::COVER_SHAPE;

#[test]
fn kit_7_105_the_page_frame_s_cover_shape_is_kit_core_s() {
    let qml = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/qml/PageFrame.qml")).unwrap();
    let given: Vec<f64> = qml
        .lines()
        .filter_map(|l| l.trim().strip_prefix("property real coverAspect:"))
        .map(|v| v.trim().parse().expect("a number"))
        .collect();
    assert_eq!(given, [COVER_SHAPE], "PageFrame's one coverAspect default");
    let orbit = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/qml/OrbitSlot.qml")).unwrap();
    assert!(!orbit.contains("0.75"), "the orbit's slot takes its shape from the host, with no fallback of its own");
}
