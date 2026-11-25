//! Eclipse Cores (#10) and its three core subpages (#72-#74) held to their boards on the graphics card. A GPU job.

mod common;
mod eclipse_board;
mod eclipse_cores_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_cores_hold_boards_10_and_72_to_74_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_cores_oracle::cores("card");
}
