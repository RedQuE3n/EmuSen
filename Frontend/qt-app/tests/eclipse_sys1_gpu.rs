//! Program-drawn Eclipse Dialogs/Notices (#14) and Power (#15) held on the graphics card (R-20261113-5). A GPU job.

mod common;
mod eclipse_board;
mod eclipse_sys1_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_sys1_holds_boards_14_and_15_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_sys1_oracle::sys1("card");
}
