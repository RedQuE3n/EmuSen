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

    // Eclipse's top-level dark scalar overrides from `tokens.dark`, skipping nested objects such as `desktop`.
    let start = ECLIPSE.find("\"dark\": {").expect("Eclipse's dark tokens");
    let block = &ECLIPSE[start + ECLIPSE[start..].find('{').expect("dark brace")..];
    let mut given: Vec<(Variant, String, String)> = Vec::new();
    let (mut depth, mut in_str, mut esc, mut entry_start) = (0usize, false, false, None);
    for (i, b) in block.bytes().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => {
                if depth == 1 && entry_start.is_none() {
                    entry_start = Some(i);
                }
                in_str = true;
            }
            b'{' => {
                if depth == 1 {
                    entry_start = None;
                }
                depth += 1;
            }
            b'}' => {
                if depth == 1 {
                    if let Some(s) = entry_start.take() {
                        if let Some((n, v)) = block[s..i].trim().split_once(':') {
                            given.push((Variant::Dark, n.trim().trim_matches('"').to_owned(), v.trim().trim_matches('"').to_owned()));
                        }
                    }
                    break;
                }
                depth -= 1;
            }
            b',' if depth == 1 => {
                if let Some(s) = entry_start.take() {
                    if let Some((n, v)) = block[s..i].trim().split_once(':') {
                        given.push((Variant::Dark, n.trim().trim_matches('"').to_owned(), v.trim().trim_matches('"').to_owned()));
                    }
                }
            }
            _ => {}
        }
    }
    scene.kit_tokens().expect("the kit's tokens").set_overrides(given);
    scene.wait(50);
    let seen = rings(&mut scene);
    assert!(!seen.is_empty() && seen.iter().all(|r| *r == (TIGHT, GAPPED)), "each tile's ring drawn Gapped under Eclipse: {seen:?}");
}
