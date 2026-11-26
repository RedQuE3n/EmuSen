//! Eclipse's game page, its Cheats tab, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use common::wait_for;
use emusen_kit_core::input::UiButton;
use game::{add_cheats, add_media, board_states, focused, open_page, open_tab, page_tab, program_with_media, until, words_under, world, BOARD_CHEATS};

const TABLE: &str = include_str!("eclipse/game_cheats.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_cheats_is_its_board_toggles_scrolls_opens_tool_and_backs() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-cheats", &states, true, true);
    add_cheats(&w, &BOARD_CHEATS);
    let media = add_media(&mut harness, &w, &["covers"]);
    let mut scene = program_with_media(&mut harness, Some(&media));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 5);
    until(&mut scene, "focus starts on pageRow.0", 2000, |s| focused(s) == "pageRow.0");

    scene.pad(UiButton::Down);
    scene.wait(150);
    assert_eq!(focused(&mut scene), "pageRow.1", "Start with all keys focused, as the board draws it");
    let words = [
        ("trailWords.0", "Consoles".to_owned(), false),
        ("trailWords.1", "Super Nintendo".to_owned(), false),
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("battery", "82%".to_owned(), false),
        ("clock", "9:41 PM".to_owned(), false),
        ("gameKicker", "SUPER NINTENDO · 1994 · ACTION".to_owned(), true),
        ("gameTitle", "Aurora Drift".to_owned(), false),
        ("gameLine", "Last played yesterday · 3 h 20 min played · 1 to 2 players".to_owned(), false),
        ("buttonLabel.0", "Resume".to_owned(), false),
        ("pillLabel.1", "Start from the beginning".to_owned(), false),
        ("pillLabel.2", "Favourite".to_owned(), false),
        ("tabWords.5", "Cheats".to_owned(), false),
        ("pageRowName.0", "Infinite lives".to_owned(), false),
        ("pageRowWords.0", "On".to_owned(), false),
        ("pageRowName.1", "Start with all keys".to_owned(), false),
        ("pageRowWords.1", "On".to_owned(), false),
        ("pageRowName.2", "Moon jump".to_owned(), false),
        ("pageRowWords.2", "Off".to_owned(), false),
        ("pageRowName.3", "Every level open".to_owned(), false),
        ("pageRowWords.3", "On".to_owned(), false),
        ("pageRowName.4", "Slow motion".to_owned(), false),
        ("pageRowWords.4", "Off".to_owned(), false),
        ("gameAsideWords", emusen_qt_models::game_page::CHEATS_SENTENCE.to_owned(), false),
        ("gameAsidePlace", "2 of 12 · 3 of 12 on".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(&mut scene, TABLE, &words, &["Turn off", "All cheats", "Back", "Home"], &["A", "X", "B", "MENU"], &mut differences);
    game::picture(&mut scene, "game-cheats-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // A, Right and Left toggle the focused cheat and update the count and help label.
    scene.pad(UiButton::Accept);
    until(&mut scene, "A turns cheat off", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") == "Off");
    assert_eq!(eclipse_board::read(&scene.tree(), "gameAsidePlace", "text"), "2 of 12 · 2 of 12 on");
    assert_eq!(words_under(&scene.tree(), "help"), ["Turn on", "All cheats", "Back", "Home"]);
    scene.pad(UiButton::Right);
    until(&mut scene, "Right turns cheat on", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") == "On");
    assert_eq!(eclipse_board::read(&scene.tree(), "gameAsidePlace", "text"), "2 of 12 · 3 of 12 on");
    scene.pad(UiButton::Left);
    until(&mut scene, "Left turns cheat off", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") == "Off");
    scene.pad(UiButton::Accept);
    until(&mut scene, "A restores cheat on", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") == "On");

    // Scrolling past 5 rows keeps the 900x350 window and hides out-of-window rows.
    for _ in 0..5 {
        scene.pad(UiButton::Down);
        scene.wait(120);
    }
    assert_eq!(focused(&mut scene), "pageRow.6");
    let tree = scene.tree();
    assert_eq!(eclipse_board::read(&tree, "gameAsidePlace", "text"), "7 of 12 · 3 of 12 on");
    assert!(tree.nodes.iter().any(|n| n.name == "pageRow.6" && n.visible), "scrolled row 6 is visible");
    assert!(!tree.nodes.iter().any(|n| n.name == "pageRow.0" && n.visible), "out-of-window row 0 is hidden");
    assert!(!tree.nodes.iter().any(|n| n.name == "pageRow.1" && n.visible), "out-of-window row 1 is hidden");

    // X All cheats opens the Cheats tool from Shown::Page, and B returns to Shown::Page on Cheats.
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Tool", 3000);
    assert_eq!(scene.property("screens", "tool"), "cheats");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Page", 3000);
    until(&mut scene, "returned to Eclipse Cheats tab", 3000, |s| page_tab(s) == 5 && s.property("theme", "pageDrawn") == "true");

    // Moving up to pill.0 hides the aside; B on Cheats leaves the Game page for Home.
    until(&mut scene, "focus on a cheat row", 2000, |s| focused(s).starts_with("pageRow."));
    while focused(&mut scene) != "pill.0" {
        scene.pad(UiButton::Up);
        scene.wait(100);
    }
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "gameAside" && n.visible), "aside hidden on pills");
    scene.pad(UiButton::Down);
    scene.wait(150);
    assert_eq!(focused(&mut scene), "pageRow.0");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.assert_quiet();
}
