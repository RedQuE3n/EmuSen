//! A page frame's tiles keep their own ring (EmuSen_Kit.md §7.113, R-20261114-21): each tile's FocusRing is Tight under the
//! kit's tokens and drawn Gapped under a look that sets `ringGapped`, Eclipse's. One test, as the harness is one per process.

use emusen_kit_core::tokens::Variant;
use emusen_kit_gallery::SCREENS_URL;
use emusen_qt_test::{Harness, Scene};

/// FocusRing's placements, as its `Placement` enumeration numbers them.
const GAPPED: f64 = 0.0;
const TIGHT: f64 = 1.0;

const ECLIPSE: &str = include_str!("../../themes/eclipse/theme.json");

/// Each tile ring's placement and the placement it draws.
fn rings(scene: &mut Scene<'_>) -> Vec<(f64, f64)> {
    let tree = scene.tree();
    let frame = tree.find_named("frame").expect("the frame");
    let ids: Vec<u64> = tree
        .under(frame)
        .into_iter()
        .filter(|&i| tree.nodes[i as usize].ty == "FocusRing")
        .map(|i| tree.nodes[i as usize].id.parse().expect("an id"))
        .collect();
    ids.into_iter().map(|id| (scene.number(id, "placement"), scene.number(id, "form"))).collect()
}

#[test]
fn kit_7_113_a_page_s_tiles_keep_their_tight_ring_and_draw_eclipse_s_gapped_one() {
    assert!(!SCREENS_URL.is_empty(), "the kit is linked into this test");
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/qml/PageFrameSwap.qml");
    let mut scene = harness.load(&format!("file://{file}"), 1280, 800);
    // The scene hands the frame its two tiles of `second` once the keyboard's model resets (PageFrameSwap.qml).
    assert!(scene.set_property("scene", "swapping", "true"));
    assert!(scene.set_property("first", "layouts", "letters\na b c"));
    for _ in 0..100 {
        if scene.property("frame", "tilesShown") == "true" {
            break;
        }
        scene.wait(20);
    }
    let seen = rings(&mut scene);
    assert!(!seen.is_empty() && seen.iter().all(|r| *r == (TIGHT, TIGHT)), "each tile's ring Tight under the kit's tokens: {seen:?}");

    // Eclipse's dark overrides, each `"name": value` line of its tokens, as the host hands them.
    let dark = &ECLIPSE[ECLIPSE.find("\"dark\": {").expect("Eclipse's dark tokens")..];
    let dark = &dark[..dark.find('}').unwrap_or(dark.len())];
    let given: Vec<(Variant, String, String)> = dark
        .lines()
        .skip(1)
        .filter_map(|l| l.trim().trim_end_matches(',').split_once(": "))
        .map(|(n, v)| (Variant::Dark, n.trim_matches('"').to_owned(), v.trim_matches('"').to_owned()))
        .collect();
    scene.kit_tokens().expect("the kit's tokens").set_overrides(given);
    scene.wait(50);
    let seen = rings(&mut scene);
    assert!(!seen.is_empty() && seen.iter().all(|r| *r == (TIGHT, GAPPED)), "each tile's ring drawn Gapped under Eclipse: {seen:?}");
}
