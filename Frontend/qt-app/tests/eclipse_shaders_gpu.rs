//! Eclipse Screen filter / Shaders (#20) live Serenity Vulkan split-preview test. A GPU job.

mod common;
mod eclipse_board;
#[path = "eclipse_pad/mod.rs"]
mod eclipse_pad;
mod eclipse_settings_oracle;
mod eclipse_shaders_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_11_eclipse_shaders_vulkan_split_preview_on_the_card() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_shaders_oracle::oracle_gpu();
}
