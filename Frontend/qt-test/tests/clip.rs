// EmuSen_Kit.md §7.34: a body and a pad glyph in a clipping Flickable are cut at its edge as the words beside them are, on
// the software backend and the GPU's. EMUSEN_QT_GPU runs the scene on the GPU through OpenGL, offscreen; the grab is written
// to EMUSEN_QT_COMPARE (else the temporary folder) as clip-<step>-<backend>.png.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/ClipScene.qml";

/// The grid of window points across one item's columns, in a band of rows.
fn band(columns: (f64, f64), rows: (f64, f64)) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let mut y = rows.0;
    while y < rows.1 {
        let mut x = columns.0;
        while x < columns.1 {
            points.push((x, y));
            x += 4.0;
        }
        y += 4.0;
    }
    points
}

#[test]
fn qt_kit_7_32_a_body_and_a_glyph_are_cut_at_a_clipping_edge() {
    let gpu = std::env::var_os("EMUSEN_QT_GPU").is_some();
    if gpu {
        // The offscreen platform on the GPU through OpenGL, no window on the desk (EmuSen_Kit.md §7.10).
        std::env::set_var("QT_QUICK_BACKEND", "rhi");
        std::env::set_var("QSG_RHI_BACKEND", "opengl");
    }
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 400, 300);
    scene.wait(200);
    let out = std::env::var_os("EMUSEN_QT_COMPARE").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let backend = if gpu { "gpu" } else { "software" };
    let ground = scene.colours_at(&[(380.0, 290.0)])[0].expect("the window's own colour");
    let away = |c: &Option<Colour>| {
        c.is_some_and(|c| [(c.r, ground.r), (c.g, ground.g), (c.b, ground.b)].iter().any(|(a, b)| (i32::from(*a) - i32::from(*b)).abs() > 8))
    };
    let columns = [("the body", (48.0, 104.0)), ("the glyph", (136.0, 184.0)), ("the words", (216.0, 330.0))];
    // The Flickable stands from 40 to 200 in the window; the third row from 168 to 232 at rest, cut by the foot.
    for &(what, span) in &columns {
        let inside = scene.colours_at(&band(span, (172.0, 198.0)));
        assert!(inside.iter().filter(|c| away(c)).count() > 10, "{what} is drawn above the edge");
    }
    // The first row's ring reaches past its body's box on the left, from 48 to about 40: a layer cut to the box would lose it.
    let ring = scene.colours_at(&band((41.0, 47.0), (60.0, 90.0)));
    assert!(ring.iter().any(|c| away(c)), "the ring past the body's box is drawn");
    // At rest, scrolled 32 (the first row cut by the top, the fourth past the foot), and back: nothing outside the Flickable.
    let mut faults = Vec::new();
    for (step, y) in [("rest", "0"), ("scrolled", "32"), ("back", "0")] {
        assert!(scene.set_property("clipper", "contentY", y));
        scene.wait(100);
        assert!(scene.grab(&out.join(format!("clip-{step}-{backend}.png")).display().to_string()));
        for &(what, span) in &columns {
            let mut past = scene.colours_at(&band(span, (2.0, 38.0)));
            past.extend(scene.colours_at(&band(span, (202.0, 298.0))));
            let drawn = past.iter().filter(|c| away(c)).count();
            if drawn > 0 {
                faults.push(format!("{step}: {what} drawn past the clipping edges at {drawn} of {} points", past.len()));
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}
