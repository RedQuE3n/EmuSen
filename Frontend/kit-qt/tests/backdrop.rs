// EmuSen_Kit.md §7.118: the kit's backdrop on the software backend, which makes no blur: the ground, each field unblurred in its
// colour at its opacity over the ground, an ellipse's box corners the ground's, the turned streak turned, and a picture given in
// their place drawing nothing sharp. One test, as the harness is one per process.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::{Harness, Scene};

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/BackdropScene.qml");

fn at(scene: &mut Scene<'_>, x: f64, y: f64) -> Colour {
    scene.colours_at(&[(x, y)])[0].expect("a colour")
}

/// A colour at an opacity over another, each channel rounded as an 8-bit blend.
fn over(top: Colour, opacity: f64, under: Colour) -> Colour {
    let mix = |a: u8, b: u8| (f64::from(a) * opacity + f64::from(b) * (1.0 - opacity)).round() as u8;
    Colour::rgb(mix(top.r, under.r), mix(top.g, under.g), mix(top.b, under.b))
}

fn near(a: Colour, b: Colour) -> bool {
    a.r.abs_diff(b.r) <= 1 && a.g.abs_diff(b.g) <= 1 && a.b.abs_diff(b.b) <= 1
}

#[test]
fn kit_7_118_the_backdrop_on_the_software_backend_is_its_fields_unblurred() {
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 1200, 800);
    scene.wait(200);
    assert!(emusen_qt_test::ffi::ffi::harness_software(), "the software backend");
    let ground = Colour::parse("#0A1024").unwrap();
    let teal = Colour::parse("#1FC9A8").unwrap();
    let lilac = Colour::parse("#D9C8FF").unwrap();
    assert_eq!(at(&mut scene, 20.0, 20.0), ground, "the ground");
    let centre = at(&mut scene, 400.0, 300.0);
    assert!(near(centre, over(teal, 0.85, ground)), "the ellipse's middle is its colour at 0.85 over the ground: {centre:?}");
    // Just inside and just outside the ellipse's edge across its middle: unblurred, the edge is sharp.
    assert!(near(at(&mut scene, 203.0, 300.0), over(teal, 0.85, ground)), "inside the ellipse's left edge");
    assert_eq!(at(&mut scene, 196.0, 300.0), ground, "outside the ellipse's left edge, nothing blurred past it");
    assert_eq!(at(&mut scene, 205.0, 155.0), ground, "the box's corner outside the ellipse");
    // The streak, turned 18 degrees counter-clockwise about its middle (850, 520): its middle is drawn, its unturned end is not.
    assert!(near(at(&mut scene, 850.0, 520.0), over(lilac, 0.5, ground)), "the streak's middle");
    assert_eq!(at(&mut scene, 990.0, 520.0), ground, "where the unturned streak's end would be");
    let turned = (850.0 + 130.0 * 18f64.to_radians().cos(), 520.0 - 130.0 * 18f64.to_radians().sin());
    assert!(near(at(&mut scene, turned.0, turned.1), over(lilac, 0.5, ground)), "the turned streak's end");

    // A picture in their place: on the software backend no sharp picture stands in for the blur, and the fields go.
    let picture = format!("file://{}/../themes/emusen/assets/previews/home.png", env!("CARGO_MANIFEST_DIR"));
    assert!(scene.set_property("scene", "picture", &picture));
    scene.wait(300);
    let tree = scene.tree();
    let shown = |name: &str| tree.nodes.iter().any(|n| n.name == name && n.visible);
    assert!(!shown("backdropPicture") && !shown("backdropFields"), "no picture and no fields");
    assert_eq!(at(&mut scene, 400.0, 300.0), ground, "the ground alone");
    scene.assert_quiet();
}
