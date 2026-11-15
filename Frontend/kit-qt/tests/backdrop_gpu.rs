// EmuSen_Kit.md §7.118: the kit's backdrop on the graphics card, in a compositor of the test's own: each field blurred past its
// edge by MultiEffect, the ground where no field reaches, and a picture blurred in the fields' place; the fall-off across the
// ellipse's left edge printed beside the CSS rule for the manual's table. A GPU job.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/BackdropScene.qml");

fn at(scene: &mut Scene<'_>, x: f64, y: f64) -> Colour {
    scene.colours_at(&[(x, y)])[0].expect("a colour")
}

/// The CSS filter's share of a straight edge's colour d past it, for a Gaussian of deviation s: 1 - Φ(d / s).
fn css(d: f64, s: f64) -> f64 {
    0.5 * erfc(d / (s * std::f64::consts::SQRT_2))
}

/// The complementary error function, Abramowitz and Stegun 7.1.26 (error under 1.5e-7).
fn erfc(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let y = t * (0.254_829_592 + t * (-0.284_496_736 + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429)))) * (-x * x).exp();
    if x >= 0.0 { y } else { 2.0 - y }
}

#[test]
fn kit_7_118_the_backdrop_on_the_card_blurs_its_fields_and_its_picture() {
    let kwin = Compositor::start(1200, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 1200, 800);
    scene.wait(300);
    let ground = Colour::parse("#0A1024").unwrap();
    let teal = Colour::parse("#1FC9A8").unwrap();
    assert_eq!(at(&mut scene, 20.0, 20.0), ground, "the ground where no field reaches");
    // The teal's share across the ellipse's left edge (x 200) along its middle, against the ground's green channel.
    let share = |c: Colour| (f64::from(c.g) - f64::from(ground.g)) / ((f64::from(teal.g) - f64::from(ground.g)) * 0.85);
    let mut past = 0.0;
    for d in [-40.0, -20.0, 0.0, 20.0, 40.0, 80.0] {
        let s = share(at(&mut scene, 200.0 - d, 300.0));
        println!("backdrop\tcard\t{d}\t{s:.4}\t{:.4}", css(d, 40.0));
        if d > 0.0 {
            past += s;
        }
    }
    assert!(past > 0.0, "the field is blurred past its edge");
    let middle = share(at(&mut scene, 400.0, 300.0));
    assert!(middle > 0.5, "the ellipse's middle keeps its colour: {middle:.3}");

    let picture = format!("file://{}/../themes/emusen/assets/previews/home.png", env!("CARGO_MANIFEST_DIR"));
    assert!(scene.set_property("scene", "picture", &picture));
    scene.wait(500);
    let tree = scene.tree();
    let shown = |name: &str| tree.nodes.iter().any(|n| n.name == name && n.visible);
    assert!(shown("backdropPicture") && !shown("backdropFields"), "the picture in the fields' place");
    scene.assert_quiet();
}
