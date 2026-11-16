// EmuSen_Kit.md §7.116: the disc's rim and its two glows on the graphics card at Eclipse's board's measures (Main.dc.html:25),
// in a compositor of the test's own: the colour drawn at set distances past the disc's edge against CSS's box-shadows, each
// the disc grown by its spread and blurred by a Gaussian of half its blur radius (CSS Backgrounds 3 §6.1.2), computed here. A GPU job.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::Harness;

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/CoronaGlowScene.qml");

/// The scene's disc: its centre, radius and the row the test reads, test inputs written in CoronaGlowScene.qml.
const CENTRE: (f64, f64) = (-250.0, 400.0);
const RADIUS: f64 = 700.0;

/// How far a drawn channel may stand from the computed one, in levels of 255: measured 2026-11-15 on the RX 6800, the worst of
/// thirteen distances 2 (4 to 32 px past the edge); drawn by Qt's RectangularShadow the disc's glows stood 9 levels under.
const BOUND: u8 = 2;

/// The error function, Abramowitz and Stegun 7.1.26 (error under 1.5e-7).
fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let y = 1.0 - t * (0.254_829_592 + t * (-0.284_496_736 + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429)))) * (-x * x).exp();
    if x >= 0.0 { y } else { -y }
}

/// A disc of radius `r` convolved with a Gaussian of deviation `s`, at `rho` from its centre: across the disc along the line
/// to the point, each chord's share by the error function, summed at a tenth of a pixel.
fn disc_share(r: f64, s: f64, rho: f64) -> f64 {
    let step = 0.1;
    let mut sum = 0.0;
    let mut u = -r + step / 2.0;
    while u < r {
        let chord = (r * r - u * u).sqrt();
        let along = (-(u - rho).powi(2) / (2.0 * s * s)).exp() / (s * (2.0 * std::f64::consts::PI).sqrt());
        sum += along * erf(chord / (s * std::f64::consts::SQRT_2)) * step;
        u += step;
    }
    sum
}

fn over(fg: (f64, f64, f64), a: f64, bg: (f64, f64, f64)) -> (f64, f64, f64) {
    (fg.0 * a + bg.0 * (1.0 - a), fg.1 * a + bg.1 * (1.0 - a), fg.2 * a + bg.2 * (1.0 - a))
}

#[test]
fn kit_7_116_the_disc_s_glows_on_the_card_are_the_board_s_box_shadows() {
    let kwin = Compositor::start(1200, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 1200, 800);
    scene.wait(500);
    let ground = (10.0, 16.0, 36.0);
    let glow = ((246.0, 214.0, 200.0), 0.6, 8.0, 44.0);
    let halo = ((236.0, 160.0, 186.0), 0.28, 46.0, 170.0);
    let mut worst = 0u8;
    for d in [4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0, 128.0, 192.0, 256.0, 384.0] {
        let x = CENTRE.0 + RADIUS + d;
        let rho = RADIUS + d;
        // Under the first shadow the second, each its colour at its alpha times its share (CSS Backgrounds 3 §6.1: the first is on top).
        let under = over(halo.0, halo.1 * disc_share(RADIUS + halo.2, halo.3 / 2.0, rho), ground);
        let want = over(glow.0, glow.1 * disc_share(RADIUS + glow.2, glow.3 / 2.0, rho), under);
        let want = Colour::rgb(want.0.round() as u8, want.1.round() as u8, want.2.round() as u8);
        let have = scene.colours_at(&[(x, CENTRE.1)])[0].expect("a colour");
        let diff = have.r.abs_diff(want.r).max(have.g.abs_diff(want.g)).max(have.b.abs_diff(want.b));
        println!("corona\tcard\t{d}\t{}\t{}\t{diff}", have.to_hex(), want.to_hex());
        worst = worst.max(diff);
    }
    assert!(worst <= BOUND, "the disc's glows are the board's box-shadows within {BOUND} levels: worst {worst}");
    scene.assert_quiet();
}
