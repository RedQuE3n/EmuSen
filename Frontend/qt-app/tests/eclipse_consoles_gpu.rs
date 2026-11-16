//! Eclipse's Choose a console in the running program on the graphics card (EmuSen_Qt.md §3.4.9), in a compositor of the
//! test's own: the same oracle as eclipse_consoles.rs, so the disc's glows and the fields blur at the scene's pixels on the
//! card, and the pictures when a folder is named. A GPU job.

mod common;
mod eclipse_board;
mod eclipse_consoles_oracle;
mod eclipse_pad;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_9_eclipse_s_consoles_is_its_board_on_the_card() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_consoles_oracle::oracle("card");
}
