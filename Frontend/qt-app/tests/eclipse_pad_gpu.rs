//! Eclipse's Controller Bindings (#12) and Keyboard Keys (#75) on the graphics card. A GPU job.

mod common;
mod eclipse_board;
#[path = "eclipse_pad/mod.rs"]
mod eclipse_pad;
mod eclipse_pad_oracle;
mod eclipse_settings_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_11_eclipse_draws_controller_bindings_and_keyboard_keys_as_their_boards_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_pad_oracle::oracle("card");
}
