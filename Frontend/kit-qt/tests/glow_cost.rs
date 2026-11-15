// EmuSen_Kit.md §7.113: what a software glow costs at a corona disc's halo, the largest the Eclipse screens draw, at the
// side a 1280 × 800 and a 3840 × 2160 window give it: its first paint makes the blurred shape once, a recolour makes none; the
// time of each is printed for the manual. One test, as the harness is one per process.

use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};
use std::time::Instant;

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/GlowCostScene.qml");

/// A number of the halo's painted glow, none while it has none.
fn canvas(scene: &mut Scene<'_>, property: &str) -> Option<f64> {
    let tree: Tree = scene.tree();
    let node = tree.nodes.iter().find(|n| n.name == "glowCanvas")?;
    let object = node.id.parse::<u64>().unwrap() as *mut cxx_qt::QObject;
    // Safety: the id was read from the live tree.
    let object = unsafe { core::pin::Pin::new_unchecked(&mut *object) };
    emusen_qt_test::ffi::ffi::harness_property(object, &cxx_qt_lib::QString::from("glowCanvas"), &cxx_qt_lib::QString::from(property)).to_string().parse().ok()
}

/// Waits for the glow's next paint after `change`, and the milliseconds it took.
fn timed(scene: &mut Scene<'_>, change: impl FnOnce(&mut Scene<'_>)) -> u128 {
    let before = canvas(scene, "paints").unwrap_or(0.0);
    let started = Instant::now();
    change(scene);
    while canvas(scene, "paints").unwrap_or(0.0) <= before {
        assert!(started.elapsed().as_secs() < 120, "the glow did not paint");
        scene.wait(1);
    }
    started.elapsed().as_millis()
}

#[test]
fn kit_7_113_a_disc_s_halo_blurs_once_and_recolours_without_a_blur() {
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 1280, 800);
    scene.wait(100);
    assert!(emusen_qt_test::ffi::ffi::harness_software(), "the software backend");
    for (window, side) in [("1280x800", "1400"), ("3840x2160", "3780")] {
        let first = timed(&mut scene, |s| assert!(s.set_property("scene", "side", side)));
        let blurs = canvas(&mut scene, "blurs").unwrap();
        let recolour = timed(&mut scene, |s| assert!(s.set_property("scene", "ink", "#476A4CF0")));
        assert_eq!(canvas(&mut scene, "blurs").unwrap(), blurs, "{window}: a recolour makes no blurred shape");
        println!("glow cost\tsoftware\t{window}\tside {side}\tfirst paint {first} ms\trecolour {recolour} ms");
        assert!(scene.set_property("scene", "side", "0"));
        assert!(scene.set_property("scene", "ink", "#47ECA0BA"));
        scene.wait(50);
    }
    scene.assert_quiet();
}
