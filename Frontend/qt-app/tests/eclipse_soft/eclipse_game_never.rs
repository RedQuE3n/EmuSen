//! Eclipse's game page for a game never played, held to its board in the program (EmuSen_Qt.md §3.4.12): no state, no play,
//! not a favourite; the States tab shows the empty slot at rest beside the board's words, Start takes the focus, and every
//! row of `eclipse/game_never.tsv` is held at each of the gate's sizes with the board's words.

use crate::eclipse_board;
use crate::game;

use game::{focused, open_page, program, states_tab, world};

const TABLE: &str = include_str!("../eclipse/game_never.tsv");

#[test]
fn qt_3_4_12_a_game_never_played_is_its_board_at_every_size() {
    let mut harness = emusen_qt_test::Harness::new();
    let world = world("qt-app-eclipse-game-never", &[], false, true);
    let mut scene = program(&mut harness);
    open_page(&mut scene, &world);
    states_tab(&mut scene);
    assert_eq!(focused(&mut scene), "pill.0", "Start takes the focus: the empty slot is never pressed (R-20261116-36)");
    let words = [
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("gameKicker", "SUPER NINTENDO · 1994 · ACTION".to_owned(), true),
        ("gameTitle", "Aurora Drift".to_owned(), false),
        ("gameLine", "Never played · 1 to 2 players".to_owned(), false),
        ("buttonLabel.0", "Start".to_owned(), false),
        ("pillLabel.1", "Favourite".to_owned(), false),
        ("tabWords.1", "States".to_owned(), false),
        ("tabWords.5", "Cheats".to_owned(), false),
        ("emptyName", "Empty slot".to_owned(), false),
        ("gameNote", "No states yet. A state is saved from the Pause Card while the game runs.".to_owned(), false),
    ];
    let mut differences = Vec::new();
    // Start in the help line is the kit's name for the pad's Start: MENU for the Xbox family whose letters the board draws.
    game::hold_board(&mut scene, TABLE, &words, &["Start", "Favourite", "Back", "Home"], &["A", "Y", "B", "MENU"], &mut differences);
    let tree = scene.tree();
    if eclipse_board::read(&tree, "gameNote", "lineCount") != "2" {
        differences.push(format!("the note takes {} lines where the board's takes 2", eclipse_board::read(&tree, "gameNote", "lineCount")));
    }
    game::picture(&mut scene, "game-never-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    scene.assert_quiet();
}
