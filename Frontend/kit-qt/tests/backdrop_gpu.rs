// EmuSen_Kit.md §7.118: the kit's backdrop on the graphics card, in a compositor of the test's own: each field blurred past its
// edge by the kit's Gaussian, the ground where no field reaches, and a picture blurred in the fields' place; the fall-off across the
// ellipse's left edge printed beside the CSS rule for the manual's table. A GPU job.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/BackdropScene.qml");

fn at(scene: &mut Scene<'_>, x: f64, y: f64) -> Colour {
    scene.colours_at(&[(x, y)])[0].expect("a colour")
}

/// How far a drawn share may stand from the computed Gaussian's: measured 2026-11-15 on the RX 6800, the worst of eight
/// distances 0.0163 (at 20 px inside the edge, drawn 0.6423 for 0.6260), two and a half 8-bit levels of the teal's green.
const BOUND: f64 = 0.02;

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
    // The teal's share across the ellipse's left edge (x 200) along its middle, against the ground's green channel, beside the
    // CSS filter's: the ellipse (200, 150, 400 by 300) convolved with a Gaussian of deviation 40 (Filter Effects 1, blur()),
    // summed over the ellipse at half-pixel steps.
    let share = |c: Colour| (f64::from(c.g) - f64::from(ground.g)) / ((f64::from(teal.g) - f64::from(ground.g)) * 0.85);
    let gaussian = |x: f64, y: f64| {
        let (s, step) = (40.0_f64, 0.5);
        let mut sum = 0.0;
        let mut qy = 150.0 + step / 2.0;
        while qy < 450.0 {
            let mut qx = 200.0 + step / 2.0;
            while qx < 600.0 {
                let (ex, ey) = ((qx - 400.0) / 200.0, (qy - 300.0) / 150.0);
                if ex * ex + ey * ey <= 1.0 {
                    let d2 = (qx - x).powi(2) + (qy - y).powi(2);
                    sum += (-d2 / (2.0 * s * s)).exp();
                }
                qx += step;
            }
            qy += step;
        }
        sum * step * step / (2.0 * std::f64::consts::PI * s * s)
    };
    let mut worst: f64 = 0.0;
    for d in [-80.0, -40.0, -20.0, 0.0, 20.0, 40.0, 80.0, 120.0] {
        let (have, want) = (share(at(&mut scene, 200.0 - d + 0.5, 300.5)), gaussian(200.0 - d + 0.5, 300.5));
        println!("backdrop\tcard\t{d}\t{have:.4}\t{want:.4}");
        worst = worst.max((have - want).abs());
    }
    assert!(worst <= BOUND, "the field's fall-off is the CSS Gaussian's within {BOUND}: worst {worst:.4}");
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
