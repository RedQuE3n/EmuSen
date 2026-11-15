// EmuSen_Kit.md §7.119: the kit's glow moved by a box-shadow's offset on the graphics card, Qt's RectangularShadow with its
// offset, in a compositor of the test's own: the moved glow is the still glow moved, above and below its box. A GPU job.

use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/ShadowScene.qml");

fn alpha(scene: &mut Scene<'_>, x: f64, y: f64) -> f64 {
    let c = scene.colours_at(&[(x, y)])[0].expect("a colour");
    1.0 - f64::from(c.r) / 255.0
}

#[test]
fn kit_7_119_the_cast_glow_moved_by_its_offset_is_the_still_glow_moved() {
    let kwin = Compositor::start(800, 400);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 800, 400);
    scene.wait(300);
    let tree = scene.tree();
    assert_eq!(tree.nodes.iter().filter(|n| n.name == "glowShadow").count(), 2, "two cast glows");
    let offset: f64 = scene.property("scene", "offsetY").parse().unwrap();
    let (plain, moved, top, bottom) = (200.0, 600.0, 150.0, 250.0);
    let mut worst: f64 = 0.0;
    for d in [2.0, 6.0, 12.0, 20.0, 30.0] {
        let above = alpha(&mut scene, moved, top - d - 0.5);
        let still_above = alpha(&mut scene, plain, top - d - offset - 0.5);
        let below = alpha(&mut scene, moved, bottom + offset + d + 0.5);
        let still_below = alpha(&mut scene, plain, bottom + d + 0.5);
        println!("shadow\tcard\t{d}\tabove {above:.4} {still_above:.4}\tbelow {below:.4} {still_below:.4}");
        worst = worst.max((above - still_above).abs()).max((below - still_below).abs());
    }
    assert!(alpha(&mut scene, moved, bottom + 2.5) > alpha(&mut scene, moved, top - 2.5), "the shadow lies below its box");
    assert!(worst <= 2.0 / 255.0 + 1e-9, "the moved glow stands {worst:.4} from the still one moved");
    assert_eq!(alpha(&mut scene, moved, 200.0), 0.0, "nothing inside the box");
    scene.assert_quiet();
}
