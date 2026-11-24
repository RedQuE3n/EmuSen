//! Eclipse's game page, its Details tab, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use common::wait_for;
use emusen_kit_core::input::UiButton;
use game::{add_media, board_states, centre, focused, open_page, open_tab, program_with_media, until, words_under, world};

const TABLE: &str = include_str!("eclipse/game_details.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_details_is_its_board_and_every_row_works() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-details", &states, true, true);
    let media = add_media(&mut harness, &w, &["covers"]);
    let mut scene = program_with_media(&mut harness, Some(&media));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 3);
    assert_eq!(focused(&mut scene), "pageRow.0", "focus starts on Scrape This Game");
    assert_eq!(words_under(&scene.tree(), "help"), ["Choose", "Back", "Home"], "action row help");
    assert_eq!(eclipse_board::read(&scene.tree(), "pageRowWords.0", "text"), "", "Scrape idle omits ellipsis");
    assert_eq!(eclipse_board::read(&scene.tree(), "pageRowWords.4", "text"), "", "Hide idle omits ellipsis");

    scene.pad(UiButton::Down);
    scene.wait(150);
    assert_eq!(focused(&mut scene), "pageRow.1", "Edit Details focused, as the board draws it");
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
        ("tabWords.3", "Details".to_owned(), false),
        ("pageRowName.0", "Scrape This Game".to_owned(), false),
        ("pageRowName.1", "Edit Details".to_owned(), false),
        ("pageRowName.2", "Cover Art".to_owned(), false),
        ("pageRowWords.2", "Its own".to_owned(), false),
        ("pageRowName.3", "Open Folder".to_owned(), false),
        ("pageRowName.4", "Hide from Library".to_owned(), false),
        ("gameAsideWords", "The game's name, dates, people and counts, as the scraper found them or as typed here.".to_owned(), false),
        ("gameAsidePlace", "2 of 5".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(&mut scene, TABLE, &words, &["Open", "Back", "Home"], &["A", "B", "MENU"], &mut differences);
    game::picture(&mut scene, "game-details-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // Walk remaining rows and check aside sentences, positions, and help bars.
    let expected = [
        (2, "Cover Art", "Which picture stands as this game's cover in the library.", "3 of 5", "Open"),
        (3, "Open Folder", "Opens the folder holding this game's file in the file manager.", "4 of 5", "Choose"),
        (4, "Hide from Library", "Hides this game from the library without touching its file.", "5 of 5", "Choose"),
    ];
    for (idx, _name, sentence, place, action) in expected {
        scene.pad(UiButton::Down);
        scene.wait(120);
        assert_eq!(focused(&mut scene), format!("pageRow.{idx}"));
        let tree = scene.tree();
        assert_eq!(eclipse_board::read(&tree, "gameAsideWords", "text"), sentence);
        assert_eq!(eclipse_board::read(&tree, "gameAsidePlace", "text"), place);
        assert_eq!(words_under(&tree, "help"), [action, "Back", "Home"]);
    }

    // Moving focus up to pills and trail hides the aside; moving down restores it.
    for _ in 0..5 {
        scene.pad(UiButton::Up);
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "pill.0");
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "gameAside" && n.visible), "aside hidden on pills");
    scene.pad(UiButton::Up);
    scene.wait(100);
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "gameAside" && n.visible), "aside hidden on trail");
    scene.pad(UiButton::Down);
    scene.wait(100);
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.0");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "gameAside" && n.visible), "aside shown on row 0");

    // Pointer click on Edit Details opens the program's deeper page; B returns to Eclipse Details.
    let (x, y) = centre(&mut scene, "pageRow.1");
    scene.click(x, y);
    until(&mut scene, "Edit Details deeper page opened", 3000, |s| s.property("theme", "pageDrawn") == "false");
    scene.pad(UiButton::Back);
    until(&mut scene, "back to Eclipse Details", 3000, |s| s.property("theme", "pageDrawn") == "true");

    // Pad A on Cover Art opens the program's Cover Art page; B returns.
    until(&mut scene, "row focus restored", 2000, |s| focused(s).starts_with("pageRow."));
    while focused(&mut scene) != "pageRow.2" {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, "Cover Art deeper page opened", 3000, |s| s.property("theme", "pageDrawn") == "false");
    scene.pad(UiButton::Back);
    until(&mut scene, "back to Eclipse Details from Cover Art", 3000, |s| s.property("theme", "pageDrawn") == "true");

    // Pad A on Hide from Library opens the program's question; B cancels back to Details.
    until(&mut scene, "row focus restored", 2000, |s| focused(s).starts_with("pageRow."));
    while focused(&mut scene) != "pageRow.4" {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "true", 2000);
    until(&mut scene, "question opened and first answer focused", 3000, |s| {
        let tree = s.tree();
        s.property("pageQuestion", "opened") == "true" && s.focus(&tree).is_some_and(|i| tree.nodes[i as usize].name == "answer.0")
    });
    assert_eq!(scene.property("theme", "pageDrawn"), "false", "question is the program's");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pageModel", "asking", "false", 2000);
    until(&mut scene, "back to Eclipse Details after cancel", 3000, |s| s.property("theme", "pageDrawn") == "true");
    scene.assert_quiet();
}
