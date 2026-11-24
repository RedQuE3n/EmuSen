//! Eclipse's game page held to its board on the graphics card (EmuSen_Qt.md §3.4.12), in a compositor of the test's own: the
//! same oracle as eclipse_game.rs, every row of `eclipse/game.tsv` at each of the gate's sizes, and the picture when a folder is
//! named. A GPU job.

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use emusen_kit_core::input::UiButton;
use emusen_qt_test::compositor::Compositor;
use game::{board_states, focused, open_page, program, states_tab, world};

const TABLE: &str = include_str!("eclipse/game.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_page_is_its_board_at_every_size_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let world = world("qt-app-eclipse-game-gpu", &states, true, true);
    let mut scene = program(&mut harness);
    open_page(&mut scene, &world);
    states_tab(&mut scene);
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert_eq!(focused(&mut scene), "card.1", "slot 1 focused, as the board draws it");
    let mut differences = Vec::new();
    game::hold_board(&mut scene, TABLE, &[], &["Load this state", "Delete", "Back"], &["A", "X", "B"], &mut differences);
    game::picture(&mut scene, "game-1920x1200-card.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    scene.assert_quiet();
}
