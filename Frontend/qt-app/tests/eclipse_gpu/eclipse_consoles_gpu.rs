//! Eclipse's Choose a console on the graphics card, eclipse_consoles_oracle in a compositor of its own; a GPU job (EmuSen_Qt.md §3.4.9).

use crate::eclipse_consoles_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_9_eclipse_s_consoles_is_its_board_on_the_card() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_consoles_oracle::oracle("card");
}
