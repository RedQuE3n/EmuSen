//! Eclipse's Home held to its board on the graphics card (EmuSen_Qt.md §3.4.8), in a compositor of the test's own: the
//! same oracle as eclipse_home.rs, every row of `eclipse_home.tsv` at each of the gate's sizes, and the pictures when a folder
//! is named. A GPU job.

// The modules the scene imports register through the program's crate, which a test must link.
extern crate emusen_qt;

mod eclipse_home_oracle;

use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_3_4_8_eclipse_s_home_is_its_board_at_every_size_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    eclipse_home_oracle::oracle("card");
}
