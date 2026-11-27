//! Eclipse's game page, Cover Art, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

use crate::eclipse_board;
use crate::game;

use emusen_kit_core::input::UiButton;
use game::{add_media, board_states, favourite, focused, open_page, open_tab, page_tab, program_with_media, until, world};

const TABLE: &str = include_str!("../eclipse/game_cover.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_cover_is_its_board_and_selects_and_favourites() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-cover", &states, true, true);
    let media = add_media(&mut harness, &w, &["covers", "screenshots", "titlescreens", "miximages", "3dboxes", "marquees"]);
    let mut scene = program_with_media(&mut harness, Some(&media));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 3);
    until(&mut scene, "Details row 0 focused", 2000, |s| focused(s) == "pageRow.0");
    while focused(&mut scene) != "pageRow.2" {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, "Eclipse Cover Art opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "packageTitle", "text") == "Cover Art"
    });
    until(&mut scene, "coverCard.0 focused", 2000, |s| focused(s) == "coverCard.0");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "coverBadge.0" && n.visible), "In use pill on card 0");

    // Move right to card 1, as the board draws it.
    scene.pad(UiButton::Right);
    scene.wait(150);
    assert_eq!(focused(&mut scene), "coverCard.1");
    let card1_label = eclipse_board::read(&scene.tree(), "coverLabel.1", "text");
    assert!(!card1_label.is_empty());

    let words = [
        ("trailWords.0", "Consoles".to_owned(), false),
        ("trailWords.1", "Super Nintendo".to_owned(), false),
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("trailWords.3", "Details".to_owned(), false),
        ("trailWords.4", "Cover Art".to_owned(), false),
        ("battery", "82%".to_owned(), false),
        ("clock", "9:41 PM".to_owned(), false),
        ("packageTitle", "Cover Art".to_owned(), false),
        ("coverBadgeWords.0", "In use".to_owned(), false),
        ("coverLabel.0", "Its own".to_owned(), false),
        ("coverPreviewTitle", card1_label.clone(), false),
        ("coverPreviewSubtitle", "Super Nintendo".to_owned(), false),
        ("packagePlace", "2 of 6".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(
        &mut scene,
        TABLE,
        &words,
        &["Use this one", "Favourite", "Details", "Home"],
        &["A", "Y", "B", "MENU"],
        &mut differences,
    );
    game::picture(&mut scene, "game-cover-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // Pressing A chooses card 1, moving the In use pill to card 1.
    scene.pad(UiButton::Accept);
    until(&mut scene, "In use badge moved to card 1", 3000, |s| {
        s.tree().nodes.iter().any(|n| n.name == "coverBadge.1" && n.visible)
            && !s.tree().nodes.iter().any(|n| n.name == "coverBadge.0" && n.visible)
    });

    // Pressing Y toggles Favourite on the game.
    let was_fav = favourite(&w);
    scene.pad(UiButton::Search);
    until(&mut scene, "Favourite toggled by Y", 3000, |_| favourite(&w) != was_fav);

    // B returns to Eclipse Details tab showing the new cover label.
    scene.pad(UiButton::Back);
    until(&mut scene, "B returns to Details tab", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && page_tab(s) == 3 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    assert_eq!(eclipse_board::read(&scene.tree(), "pageRowWords.2", "text"), card1_label);
    scene.assert_quiet();
}
