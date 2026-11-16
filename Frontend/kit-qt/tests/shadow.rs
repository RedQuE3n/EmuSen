// EmuSen_Kit.md §7.119: the kit's glow moved by a box-shadow's offset, as a tile's resting shadow on the boards: on the software
// backend, the glow moved down draws above its box what the glow standing round it draws further out, and below it what that
// draws nearer in, nothing inside its box. One test, as the harness is one per process.

use emusen_qt_test::{Harness, Scene};

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/ShadowScene.qml");

/// The shadow's alpha at a window point: black over white, so one minus the red channel's share.
fn alpha(scene: &mut Scene<'_>, x: f64, y: f64) -> f64 {
    let c = scene.colours_at(&[(x, y)])[0].expect("a colour");
    1.0 - f64::from(c.r) / 255.0
}

#[test]
fn kit_7_119_a_glow_moved_by_its_offset_is_the_still_glow_moved() {
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 800, 400);
    scene.wait(200);
    let offset: f64 = scene.property("scene", "offsetY").parse().unwrap();
    // The two boxes' middles across, and their top and bottom edges.
    let (plain, moved, top, bottom) = (200.0, 600.0, 150.0, 250.0);
    let mut worst: f64 = 0.0;
    for d in [2.0, 6.0, 12.0, 20.0, 30.0] {
        // Above the box: the moved glow's edge is `offset` further away.
        let above = alpha(&mut scene, moved, top - d - 0.5);
        let still_above = alpha(&mut scene, plain, top - d - offset - 0.5);
        // Below the box, past the moved shape's own edge: `offset` nearer.
        let below = alpha(&mut scene, moved, bottom + offset + d + 0.5);
        let still_below = alpha(&mut scene, plain, bottom + d + 0.5);
        println!("shadow\tsoftware\t{d}\tabove {above:.4} {still_above:.4}\tbelow {below:.4} {still_below:.4}");
        worst = worst.max((above - still_above).abs()).max((below - still_below).abs());
    }
    assert!(alpha(&mut scene, moved, bottom + 2.5) > alpha(&mut scene, moved, top - 2.5), "the shadow lies below its box");
    // One level of 255: the same canvas paints both, the one moved by whole pixels.
    assert!(worst <= 1.0 / 255.0 + 1e-9, "the moved glow stands {worst:.4} from the still one moved");
    assert_eq!(alpha(&mut scene, moved, 200.0), 0.0, "nothing inside the box");
    scene.assert_quiet();
}
