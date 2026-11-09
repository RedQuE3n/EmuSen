// EmuSen_Kit.md §7.34: on the software backend a body and a pad glyph drawn uncut inside a clipping Flickable are the same
// picture, pixel for pixel, as the same two under no clip at the same fractional phase, 200 units to the right; and the
// body is the picture of its shapes drawn plainly by Qt, 20 units to the left and 160 down.

use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/PhaseScene.qml";

#[test]
fn qt_kit_7_34_a_body_and_a_glyph_inside_a_clip_and_uncut_are_drawn_as_under_no_clip() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 400, 340);
    scene.wait(200);
    let out = std::env::var_os("EMUSEN_QT_COMPARE").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    assert!(scene.grab(&out.join("phase-software.png").display().to_string()));
    let clipped: Vec<(f64, f64)> = (40..160).flat_map(|y| (20..180).map(move |x| (f64::from(x), f64::from(y)))).collect();
    let free: Vec<(f64, f64)> = clipped.iter().map(|&(x, y)| (x + 200.0, y)).collect();
    let (a, b) = (scene.colours_at(&clipped), scene.colours_at(&free));
    let differing: Vec<_> = clipped.iter().zip(a.iter().zip(&b)).filter(|(_, (p, q))| p != q).map(|(at, (p, q))| (*at, *p, *q)).collect();
    let ground = scene.colours_at(&[(10.0, 330.0)])[0];
    let drawn = a.iter().filter(|c| **c != ground).count();
    assert!(drawn > 2000, "the pair is drawn: {drawn} pixels off the ground");
    assert!(differing.is_empty(), "{} of {} pixels differ, the first {:?}", differing.len(), clipped.len(), differing.first());
    let body: Vec<(f64, f64)> = (48..120).flat_map(|y| (38..112).map(move |x| (f64::from(x), f64::from(y)))).collect();
    let plain: Vec<(f64, f64)> = body.iter().map(|&(x, y)| (x - 20.0, y + 160.0)).collect();
    let (a, b) = (scene.colours_at(&body), scene.colours_at(&plain));
    let differing: Vec<_> = body.iter().zip(a.iter().zip(&b)).filter(|(_, (p, q))| p != q).map(|(at, (p, q))| (*at, *p, *q)).collect();
    assert!(differing.is_empty(), "the body differs from its shapes drawn plainly at {} of {} pixels, the first {:?}", differing.len(), body.len(), differing.first());
}
