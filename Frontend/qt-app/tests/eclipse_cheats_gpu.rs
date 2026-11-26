//! Eclipse Cheats (#21) on Vulkan. A GPU job.

mod common;
mod eclipse_board;
mod eclipse_cheats_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_eclipse_cheats_board_on_vulkan() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_cheats_oracle::cheats_oracle("gpu");
}
