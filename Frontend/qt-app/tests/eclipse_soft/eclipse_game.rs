//! Eclipse's game page, its States tab, held to its board in the program (EmuSen_Qt.md §3.4.12): the board's game with its
//! resume point and three slots, the page opened through Eclipse's Home and the States tab by the right shoulder, slot 1
//! focused, and every row of `eclipse/game.tsv` held at each of the gate's sizes with the board's words.

use crate::game;

use emusen_kit_core::input::UiButton;
use game::{add_media, board_states, focused, open_page, program_with_media, states_tab, when_words, world};

const TABLE: &str = include_str!("../eclipse/game.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_page_is_its_board_at_every_size() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let world = world("qt-app-eclipse-game", &states, true, true);
    let media = add_media(&mut harness, &world, &["covers"]);
    let mut scene = program_with_media(&mut harness, Some(&media));
    open_page(&mut scene, &world);
    assert_eq!(scene.property("theme", "pageDrawn"), "true", "Overview is Eclipse's");
    states_tab(&mut scene);
    assert_eq!(focused(&mut scene), "card.0", "the resume point's card takes the focus");
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert_eq!(focused(&mut scene), "card.1", "slot 1 focused, as the board draws it");
    let when = |i: usize| when_words(states[i].1);
    let words = [
        ("trailWords.0", "Consoles".to_owned(), false),
        ("trailWords.1", "Super Nintendo".to_owned(), false),
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("trailSlash.0", "/".to_owned(), false),
        ("battery", "82%".to_owned(), false),
        ("clock", "9:41 PM".to_owned(), false),
        ("gameKicker", "SUPER NINTENDO · 1994 · ACTION".to_owned(), true),
        ("gameTitle", "Aurora Drift".to_owned(), false),
        ("gameLine", "Last played yesterday · 3 h 20 min played · 1 to 2 players".to_owned(), false),
        ("buttonLabel.0", "Resume".to_owned(), false),
        ("pillLabel.1", "Start from the beginning".to_owned(), false),
        ("pillLabel.2", "Favourite".to_owned(), false),
        ("tabWords.0", "Overview".to_owned(), false),
        ("tabWords.1", "States".to_owned(), false),
        ("tabWords.2", "Media".to_owned(), false),
        ("tabWords.3", "Details".to_owned(), false),
        ("tabWords.4", "Options".to_owned(), false),
        ("tabWords.5", "Cheats".to_owned(), false),
        ("cardBadgeWords.0", "RESUMES HERE".to_owned(), true),
        ("cardName.0", "Resume point".to_owned(), false),
        ("cardWhen.0", when(0), false),
        ("cardName.1", "Slot 1".to_owned(), false),
        ("cardWhen.1", when(1), false),
        ("cardName.2", "Slot 2".to_owned(), false),
        ("cardWhen.2", when(2), false),
        ("cardName.3", "Slot 3".to_owned(), false),
        ("cardWhen.3", when(3), false),
        ("emptyName", "Empty slot".to_owned(), false),
    ];
    // The board's own when-lines where the calendar gives the board's days (EclipseGame:108-111).
    assert_eq!((when(0).as_str(), when(1).as_str()), ("Yesterday, 9:12 PM", "Yesterday, 8:40 PM"));
    let tree = scene.tree();
    assert!(!tree.nodes.iter().any(|n| n.name == "gameNote" && n.visible), "the never-played board's words are not drawn beside a game's states");
    let mut differences = Vec::new();
    game::hold_board(&mut scene, TABLE, &words, &["Load this state", "Delete", "Back"], &["A", "X", "B"], &mut differences);
    game::picture(&mut scene, "game-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    scene.assert_quiet();
}
