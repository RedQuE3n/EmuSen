//! Eclipse's games view in the running program on the graphics card (EmuSen_Qt.md §3.4.10), in a compositor of the test's own: the same oracle as eclipse_games.rs, so the disc's glows, the fields and the tiles' shadows blur at the scene's pixels on the card, and the pictures when EMUSEN_ECLIPSE_PICTURES names a folder. A GPU job.

mod common;
mod eclipse_board;
mod eclipse_games_oracle;
mod eclipse_pad;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_10_eclipses_games_view_is_its_board_on_the_card() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(900);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_games_oracle::oracle("card");
}
