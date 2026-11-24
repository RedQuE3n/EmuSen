//! Eclipse's Settings on Memory cards, Saves on the cards and One save on the graphics card in the test's own compositor, held to boards #58, #63 and #64 (EmuSen_Qt.md §3.4.11). A GPU job.

mod common;
mod eclipse_board;
mod eclipse_pad;
mod eclipse_settings_cards_pages;
mod eclipse_settings_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_memory_cards_and_saves_as_their_boards_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_settings_cards_pages::cards("card");
}
