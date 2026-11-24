//! Eclipse's game page (Overview, States, Details, Options) held to its boards on the graphics card (EmuSen_Qt.md §3.4.12).

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use emusen_kit_core::input::UiButton;
use emusen_qt_test::compositor::Compositor;
use game::{add_media, add_options, board_states, focused, open_page, open_tab, program_with, set_description, states_tab, world, BOARD_DESCRIPTION};

const TABLE_OVERVIEW: &str = include_str!("eclipse/game_overview.tsv");
const TABLE_STATES: &str = include_str!("eclipse/game.tsv");
const TABLE_DETAILS: &str = include_str!("eclipse/game_details.tsv");
const TABLE_OPTIONS: &str = include_str!("eclipse/game_options.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_page_is_its_board_at_every_size_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-gpu", &states, true, true);
    set_description(&w, BOARD_DESCRIPTION);
    let cores = add_options(&w);
    let media = add_media(&mut harness, &w, &["covers", "screenshots"]);
    let mut scene = program_with(&mut harness, Some(&media), Some(&cores));
    open_page(&mut scene, &w);
    let mut differences = Vec::new();

    // Overview (#77).
    assert_eq!(focused(&mut scene), "pill.0", "Resume focused on Overview");
    game::hold_board(&mut scene, TABLE_OVERVIEW, &[], &["Resume", "Favourite", "Back", "Home"], &["A", "Y", "B", "MENU"], &mut differences);
    game::picture(&mut scene, "game-overview-1920x1200-card.png");

    // States (#78).
    states_tab(&mut scene);
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert_eq!(focused(&mut scene), "card.1", "slot 1 focused, as the board draws it");
    game::hold_board(&mut scene, TABLE_STATES, &[], &["Load this state", "Delete", "Back"], &["A", "X", "B"], &mut differences);
    game::picture(&mut scene, "game-1920x1200-card.png");

    // Details (#79).
    open_tab(&mut scene, 3);
    scene.pad(UiButton::Down);
    scene.wait(200);
    assert_eq!(focused(&mut scene), "pageRow.1", "Edit Details focused, as the board draws it");
    game::hold_board(&mut scene, TABLE_DETAILS, &[], &["Open", "Back", "Home"], &["A", "B", "MENU"], &mut differences);
    game::picture(&mut scene, "game-details-1920x1200-card.png");

    // Options (#80).
    open_tab(&mut scene, 4);
    assert_eq!(focused(&mut scene), "pageRow.0", "Core focused, as the board draws it");
    game::hold_board(
        &mut scene,
        TABLE_OPTIONS,
        &[],
        &["Change", "Value", "Back to the console's", "Back", "Home"],
        &["A", "◀ ▶", "X", "B", "MENU"],
        &mut differences,
    );
    game::picture(&mut scene, "game-options-1920x1200-card.png");

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    scene.assert_quiet();
}
