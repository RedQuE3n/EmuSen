//! Program-drawn Eclipse Pause Card held to boards #6 and #35-#43 on the graphics card (R-20261113-5). A GPU job.

mod common;
mod eclipse_board;
mod eclipse_pause_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_pause_holds_boards_6_and_35_to_43_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_pause_oracle::pause("card");
}
