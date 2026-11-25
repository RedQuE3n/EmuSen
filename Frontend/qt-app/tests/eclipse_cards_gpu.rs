//! Program-drawn Eclipse Cards frame held to boards #5, #44, #45 and #46 on the graphics card (R-20261113-5). A GPU job.

mod common;
mod eclipse_board;
mod eclipse_cards_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_cards_hold_boards_5_44_45_and_46_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_cards_oracle::cards("card");
}
