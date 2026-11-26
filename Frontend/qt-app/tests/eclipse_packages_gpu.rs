//! Eclipse Packages (#22 Library, #23 Layers, #69 Package Page) held to their boards on the GPU. A GPU job.

mod common;
mod eclipse_board;
mod eclipse_packages_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_packages_hold_boards_22_23_and_69_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_packages_oracle::packages("card");
}
