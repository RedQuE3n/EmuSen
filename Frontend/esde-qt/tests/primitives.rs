//! The painter's primitives on the GPU tier (EmuSen_Qt.md §8.1, tier 2): Vulkan in a compositor of the test's own, the
//! validation layer on, each primitive drawn over black and read back (EmuSen_Qt.md §4.8).

use std::path::{Path, PathBuf};

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Primitives.qml");

/// A binary PPM, which Qt reads with no plugin: `pixels` row by row.
fn ppm(path: &Path, width: usize, height: usize, pixels: impl Fn(usize, usize) -> [u8; 3]) {
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    for y in 0..height {
        for x in 0..width {
            bytes.extend_from_slice(&pixels(x, y));
        }
    }
    std::fs::write(path, bytes).expect("a test picture");
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("esde-qt-primitives-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

fn url(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn until_ready(scene: &mut Scene<'_>, names: &[&str]) {
    for name in names {
        let mut waited = 0;
        while scene.property(name, "status") != "Ready" {
            assert!(waited < 5000, "{name} did not load its picture in 5 s: status {}, source {}", scene.property(name, "status"), scene.property(name, "source"));
            scene.wait(20);
            waited += 20;
        }
    }
    scene.wait(200);
}

fn level(c: &Option<Colour>) -> [i32; 3] {
    let c = c.expect("a colour read back");
    [c.r as i32, c.g as i32, c.b as i32]
}

fn near(what: &str, got: [i32; 3], want: [f64; 3], within: f64) {
    let off = (0..3).map(|i| (got[i] as f64 - want[i]).abs()).fold(0.0, f64::max);
    assert!(off <= within, "{what}: read {got:?}, wanted {want:?} within {within} levels");
}

#[test]
fn the_primitives_draw_what_the_scene_asks_on_vulkan() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(120);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    std::env::set_var("QSG_RHI_DEBUG_LAYER", "1");
    std::env::set_var("VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC", "1");
    std::env::set_var("QT_VK_PHYSICAL_DEVICE_INDEX", std::env::var("EMUSEN_GPU_INDEX").unwrap_or_else(|_| "0".into()));
    let dir = scratch();
    let white = dir.join("white.ppm");
    let strip = dir.join("strip.ppm");
    let checker = dir.join("checker.ppm");
    ppm(&white, 64, 64, |_, _| [255, 255, 255]);
    ppm(&strip, 4, 1, |x, _| [[255, 0, 0], [0, 255, 0], [0, 0, 255], [200, 100, 50]][x]);
    ppm(&checker, 4, 4, |x, y| if (x + y) % 2 == 0 { [255; 3] } else { [0; 3] });
    let face = Path::new(env!("CARGO_MANIFEST_DIR")).join("../kit-qt/fonts/BarlowCondensed-Medium.ttf");

    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    assert!(scene.set_property("scene", "white", &url(&white)));
    assert!(scene.set_property("scene", "strip", &url(&strip)));
    assert!(scene.set_property("scene", "checker", &url(&checker)));
    assert!(scene.set_property("scene", "face", &url(&face)));
    until_ready(&mut scene, &["tinted", "halfTinted", "grey", "rounded", "ramp", "sharp", "soft"]);
    assert_eq!((scene.property("scene", "width"), scene.property("scene", "height")), ("1280".into(), "800".into()), "the window at its size");
    let _ = scene.grab(&compare_dir().join("primitives-1280x800.png").to_string_lossy());

    let points = [
        (32.0, 32.0),   // 0 tinted
        (112.0, 32.0),  // 1 half tinted
        (168.0, 8.0),   // 2-5 the strip's four columns
        (184.0, 8.0),
        (200.0, 8.0),
        (216.0, 8.0),
        (0.0, 100.0),   // 6 the rounded corner
        (96.0, 100.0),  // 7 the rounded top edge's middle
        (268.0, 100.0), // 8 the ramp's top row
        (268.0, 148.0), // 9 the ramp at half height
        (407.0, 104.0), // 10-11 the nearest checker either side of a cell's edge
        (408.0, 104.0),
        (457.0, 104.0), // 12 the linear checker at the same place
        (632.0, 32.0),  // 13 the star's middle
        (601.0, 1.0),   // 14 the star's corner
        (96.0, 227.0),  // 15 the rounded bottom edge's middle
        (191.0, 227.0), // 16 the rounded bottom-right corner
    ];
    let c = scene.colours_at(&points);
    near("white tinted FF8000", level(&c[0]), [255.0, 128.0, 0.0], 1.0);
    near("white tinted 80FF8000 over black", level(&c[1]), [128.0 * 255.0 / 255.0, 64.25, 0.0], 1.0);
    for (i, rgb) in [[255.0, 0.0, 0.0], [0.0, 255.0, 0.0], [0.0, 0.0, 255.0], [200.0, 100.0, 50.0]].iter().enumerate() {
        let luma = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
        near(&format!("column {i} at saturation 0 is its Rec. 601 luma"), level(&c[2 + i]), [luma; 3], 1.0);
    }
    near("a 32 px radius is clear at the corner", level(&c[6]), [0.0; 3], 0.0);
    near("and opaque at the edge's middle", level(&c[7]), [255.0; 3], 0.0);
    near("the 192 x 128 box's bottom edge too: the mask measures both axes", level(&c[15]), [255.0; 3], 0.0);
    near("and its far corner clear", level(&c[16]), [0.0; 3], 0.0);
    near("the ramp at 0.5 and falloff 2 reads 0.494 at the top", level(&c[8]), [0.4948 * 255.0; 3], 1.0);
    near("and nothing at half height", level(&c[9]), [0.0; 3], 0.0);
    near("nearest at 8x: the last pixel of a white cell", level(&c[10]), [255.0; 3], 0.0);
    near("nearest at 8x: the first of the black cell", level(&c[11]), [0.0; 3], 0.0);
    let soft = level(&c[12])[0];
    assert!(soft > 40 && soft < 215, "linear blends across the same edge ({soft}), so the nearest reading is the filter's");
    near("the star mark filled in its colour", level(&c[13]), [0.0, 255.0, 0.0], 0.0);
    near("and clear outside its points", level(&c[14]), [0.0; 3], 0.0);

    // A text reports its extent once per change of it, the face loads once per path, and no face named sets the kit's body face.
    assert!(scene.set_property("scene", "reports", "0"));
    assert!(scene.set_property("words", "text", "Measured once"));
    scene.wait(50);
    assert_eq!(scene.property("scene", "reports"), "0", "the same words report nothing");
    let width: f64 = scene.property("words", "contentWidth").parse().unwrap();
    assert!(width > 50.0 && width < 400.0, "a laid-out width ({width})");
    assert_eq!(scene.property("words", "fontStatus"), "Ready", "the theme's face loaded");
    assert_eq!(scene.property("same", "fontStatus"), "Ready");
    assert_eq!(scene.property("words", "family"), scene.property("same", "family"), "one face for one path");
    assert_eq!(scene.property("words", "family"), "Barlow Condensed");
    assert_eq!(scene.property("plain", "family"), "Barlow", "the kit's body face where a theme names none");
    assert!(scene.set_property("words", "text", "Measured once more, and wider"));
    scene.wait(50);
    assert_eq!(scene.property("scene", "reports"), "1", "a new extent reported once");

    // The side-by-side: Art Book Next's SNES slice as its dark scheme tints it (ffffffdd), for the C# grab beside it.
    let art = art_book_next().join("_inc/systems/artwork-screenshots/snes.png");
    if art.exists() {
        assert!(scene.set_property("scene", "art", &url(&art)));
        until_ready(&mut scene, &["slice"]);
        assert!(scene.grab(&compare_dir().join("e1-slice-qt-1280x800.png").to_string_lossy()));
    }
    scene.assert_quiet();
    let _ = std::fs::remove_dir_all(&dir);
}

fn art_book_next() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/whole/art-book-next-es-de")
}

fn compare_dir() -> PathBuf {
    let dir = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/tmp/qt-s4/compare");
    let _ = std::fs::create_dir_all(&dir);
    dir
}
