// EmuSen_Kit.md §7.13: the gallery, shown at its own 1280 × 800, is given 1920 × 1200 (the handheld's panel) by the
// platform, as a window manager's resize reaches Qt, and its content fills the window: no point near an edge or a corner
// shows the void behind the canvas.

use emusen_kit_core::tokens::Colour;
use emusen_kit_gallery::GALLERY_URL;
use emusen_qt_test::Harness;

const VOID: Colour = Colour { r: 0, g: 0, b: 0, a: 255 };

#[test]
fn the_gallery_fills_the_size_a_window_manager_gives_it() {
    let mut harness = Harness::new();
    let mut scene = harness.load(GALLERY_URL, 1280, 800);
    scene.resize(1920, 1200);
    let points = [(1915.0, 1195.0), (1915.0, 600.0), (960.0, 1195.0), (4.0, 1195.0), (1915.0, 4.0), (1300.0, 820.0)];
    let colours = scene.colours_at(&points);
    for (point, colour) in points.iter().zip(&colours) {
        assert!(
            colour.is_some_and(|c| (c.r, c.g, c.b) != (VOID.r, VOID.g, VOID.b)),
            "{point:?} shows the void ({colour:?}): the content stopped short of 1920 x 1200"
        );
    }
    scene.assert_quiet();
}
