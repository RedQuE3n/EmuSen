//! Eclipse Search (#8), Collections (#11), and Tools hub (#9) held on the graphics card. A GPU job.

mod common;
mod eclipse_board;
mod eclipse_sys2_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_sys2_holds_boards_8_9_and_11_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_sys2_oracle::sys2("card");
}
