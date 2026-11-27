//! Eclipse's game page, Scrape This Game, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

use crate::common;
use crate::eclipse_board;
use crate::game;

use common::wait_for;
use emusen_kit_core::input::UiButton;
use game::{board_states, focused, open_page, open_tab, page_tab, program_full, until, world, MockScraper};

const TABLE: &str = include_str!("../eclipse/game_scrape.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_scrape_is_its_board_and_handles_matches_actions_and_cancel() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-scrape", &states, true, true);
    let mock = MockScraper::start(&w.dir("MockCert"), &w.root.join("Config"));
    let mut scene = program_full(&mut harness, Some(&w.root), None, Some(&mock.test));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 3);
    until(&mut scene, "Details row 0 focused", 2000, |s| focused(s) == "pageRow.0");

    // Opening Scrape This Game launches jeuRecherche and populates the 3 matches + 3 bottom actions.
    scene.pad(UiButton::Accept);
    for _ in 0..40 {
        let tree = scene.tree();
        if eclipse_board::read(&tree, "scrapeRowName.0", "text") == "Aurora Drift (1994, Super Nintendo)" {
            break;
        }
        scene.wait(150);
    }
    let tree = scene.tree();
    assert_eq!(
        eclipse_board::read(&tree, "scrapeRowName.0", "text"),
        "Aurora Drift (1994, Super Nintendo)",
        "intro={:?}, asked={:?}, problem={:?}",
        eclipse_board::read(&tree, "scrapeIntro", "text"),
        mock.asked(),
        scene.property("pageModel", "problem")
    );
    until(&mut scene, "scrapeRow.0 focused", 2000, |s| focused(s) == "scrapeRow.0");

    let words = [
        ("trailWords.0", "Consoles".to_owned(), false),
        ("trailWords.1", "Super Nintendo".to_owned(), false),
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("trailWords.3", "Details".to_owned(), false),
        ("trailWords.4", "Scrape This Game".to_owned(), false),
        ("battery", "82%".to_owned(), false),
        ("clock", "9:41 PM".to_owned(), false),
        ("packageTitle", "Scrape This Game".to_owned(), false),
        ("scrapeIntro", "The match chosen gives the game its name, facts, cover and pictures, fetched by one paced run.".to_owned(), false),
        ("scrapeRowName.0", "Aurora Drift (1994, Super Nintendo)".to_owned(), false),
        ("scrapeRowName.1", "Aurora Drift (1995, Game Boy)".to_owned(), false),
        ("scrapeRowName.2", "Aurora Drifters (1996, Super Nintendo)".to_owned(), false),
        ("scrapeRowName.3", "Search by another name…".to_owned(), false),
        ("scrapeRowName.4", "By the file's checksums…".to_owned(), false),
        ("scrapeRowName.5", "No".to_owned(), false),
        ("packagePlace", "1 of 6".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(
        &mut scene,
        TABLE,
        &words,
        &["Choose", "Favourite", "No", "Home"],
        &["A", "Y", "B", "MENU"],
        &mut differences,
    );
    game::picture(&mut scene, "game-scrape-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // Search by another name… with "empty" shows "No matches found." and keeps the 3 bottom actions operable.
    for _ in 0..3 {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "scrapeRow.3");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "typing", "true", 2000);
    let current = scene.property("pageTyped", "text");
    for _ in 0..current.len() {
        scene.key(0x0100_0003, 0);
    }
    for c in "empty".chars() {
        scene.key(c.to_ascii_uppercase() as i32, 0);
    }
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "typing", "false", 2000);
    until(&mut scene, "empty search shows No matches found.", 10000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "scrapeIntro", "text") == "No matches found."
    });
    assert_eq!(eclipse_board::read(&scene.tree(), "scrapeRowName.0", "text"), "Search by another name…");
    assert_eq!(eclipse_board::read(&scene.tree(), "scrapeRowName.2", "text"), "No");

    // No closes Scrape This Game cleanly back to Details.
    while focused(&mut scene) != "scrapeRow.2" {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, "No returns to Details tab", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && page_tab(s) == 3 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });

    // Re-open Scrape This Game and pick match 0: closes back to Details and runs the scrape against the open store.
    scene.pad(UiButton::Accept);
    until(&mut scene, "Scrape matches reloaded", 10000, |s| {
        eclipse_board::read(&s.tree(), "scrapeRowName.0", "text") == "Aurora Drift (1994, Super Nintendo)"
    });
    until(&mut scene, "scrapeRow.0 focused", 2000, |s| focused(s) == "scrapeRow.0");
    scene.pad(UiButton::Accept);
    until(&mut scene, "picking match closes back to Details", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && page_tab(s) == 3 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    until(&mut scene, "scrape reaches done", 15000, |s| s.property("pageModel", "scrapeState") == "done");
    assert!(mock.asked().iter().any(|p| p.contains("jeuRecherche.php")), "asked jeuRecherche: {:?}", mock.asked());
    assert!(mock.asked().iter().any(|p| p.contains("cover.png")), "downloaded cover from picked match: {:?}", mock.asked());
    scene.assert_quiet();
}
