//! Eclipse Scraper (#66), Scrape Status (#67), and Library Manager (#68) held to their boards on the GPU.

mod common;
mod eclipse_board;
#[path = "eclipse_board/tools_t4.rs"]
mod tools_t4;

use emusen_qt_test::compositor::Compositor;

#[test]
fn eclipse_tools_t4_hold_boards_66_to_68_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    tools_t4::tools_t4("card");
}
