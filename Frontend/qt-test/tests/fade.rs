// EmuSen_Kit.md §7.35: the kit's EdgeFade draws on the software backend and the GPU's alike over a flat backdrop: the
// scroller's own colour above the fade, the backdrop at the edge, and between them neither. EMUSEN_QT_GPU runs the scene
// on the GPU through OpenGL, offscreen; the grab is written to EMUSEN_QT_COMPARE (else the temporary folder) as fade-<backend>.png.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/FadeScene.qml";

fn near(a: Colour, b: Colour, levels: i32) -> bool {
    [(a.r, b.r), (a.g, b.g), (a.b, b.b)].iter().all(|(x, y)| (i32::from(*x) - i32::from(*y)).abs() <= levels)
}

#[test]
fn qt_kit_7_33_an_edge_fade_draws_on_either_backend() {
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
    assert!(scene.grab(&out.join(format!("fade-{}.png", if gpu { "gpu" } else { "software" })).display().to_string()));
    // The column stands from 40 to 200 in the window and fades over its last 40.
    let seen: Vec<Colour> = scene.colours_at(&[(380.0, 290.0), (200.0, 100.0), (200.0, 180.0), (200.0, 198.0)]).into_iter().map(|c| c.expect("a pixel")).collect();
    let (ground, above, middle, edge) = (seen[0], seen[1], seen[2], seen[3]);
    assert!(!near(above, ground, 40), "the column is drawn above its fade: {above:?} on {ground:?}");
    assert!(near(edge, ground, 16), "the edge is the backdrop's: {edge:?}, the backdrop {ground:?}");
    assert!(!near(middle, ground, 20) && !near(middle, above, 20), "the middle of the fade is between the two: {middle:?}");
}
