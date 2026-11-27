//! Eclipse's game page, Edit Details, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

use crate::common;
use crate::eclipse_board;
use crate::game;

use common::wait_for;
use emusen_kit_core::input::UiButton;
use game::{add_media, board_states, focused, open_page, open_tab, page_tab, program_with_media, set_edits, until, words_under, world};
use serde_json::json;

const TABLE: &str = include_str!("../eclipse/game_edit.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_edit_is_its_board_and_edits_steps_resets_and_clears() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-edit", &states, true, true);
    set_edits(
        &w,
        json!({
            "name": "Aurora Drift",
            "sortname": "Aurora Drift",
            "description": "Skate the frozen canals …",
            "releasedate": "1994-03-14",
            "developer": "Northlight Works",
            "publisher": "Halden",
            "genre": "Racing",
            "players": "1-2",
            "rating": "0.8",
            "controller": "gamepad_nintendo_snes",
        }),
    );
    let media = add_media(&mut harness, &w, &["covers"]);
    let mut scene = program_with_media(&mut harness, Some(&media));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 3);
    until(&mut scene, "Details row 0 focused", 2000, |s| focused(s) == "pageRow.0");
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.1", "Edit Details row focused");
    scene.pad(UiButton::Accept);
    until(&mut scene, "Eclipse Edit Details opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "packageTitle", "text") == "Edit Details"
    });
    until(&mut scene, "Edit row 0 focused", 2000, |s| focused(s) == "pageRow.0");

    // Step down to Developer (row 5), as the board draws it.
    for _ in 0..5 {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "pageRow.5");

    let words = [
        ("trailWords.0", "Consoles".to_owned(), false),
        ("trailWords.1", "Super Nintendo".to_owned(), false),
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("trailWords.3", "Details".to_owned(), false),
        ("trailWords.4", "Edit Details".to_owned(), false),
        ("battery", "82%".to_owned(), false),
        ("clock", "9:41 PM".to_owned(), false),
        ("packageTitle", "Edit Details".to_owned(), false),
        ("pageRowName.0", "Name".to_owned(), false),
        ("pageRowWords.0", "Aurora Drift".to_owned(), false),
        ("pageRowName.4", "Release date".to_owned(), false),
        ("pageRowWords.4", "14 March 1994".to_owned(), false),
        ("pageRowName.5", "Developer".to_owned(), false),
        ("pageRowWords.5", "Northlight Works".to_owned(), false),
        ("pageRowName.9", "Rating".to_owned(), false),
        ("pageRowWords.9", "4 of 5".to_owned(), false),
        ("pageRowName.16", "Controller".to_owned(), false),
        ("pageRowWords.16", "SNES".to_owned(), false),
        ("pageRowName.17", "Clear Scraped Data".to_owned(), false),
        ("packagePlace", "6 of 18".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(
        &mut scene,
        TABLE,
        &words,
        &["Edit", "Value", "Reset to scraped", "Details", "Home"],
        &["A", "◀ ▶", "Y", "B", "MENU"],
        &mut differences,
    );
    game::picture(&mut scene, "game-edit-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // Verify Hidden is omitted (18 rows total, no row 18).
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "pageRow.18" && n.visible), "Hidden omitted per R-20261116-32");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "pageRowPen.5" && n.visible), "focused text row shows pen");

    // Release date row opens the docked keyboard with the raw ISO date string, not the formatted date.
    scene.pad(UiButton::Up);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.4");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "typing", "true", 2000);
    assert_eq!(scene.property("pageTyped", "text"), "1994-03-14", "raw edit string when typing");
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "typing", "false", 2000);
    until(&mut scene, "back to Eclipse Edit after keyboard done", 2000, |s| s.property("theme", "pageDrawn") == "true");

    // Step across to Rating (right column row 9): Right steps 4 of 5 -> 5 of 5; Left steps back.
    scene.pad(UiButton::Right);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.13");
    for _ in 0..4 {
        scene.pad(UiButton::Up);
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "pageRow.9");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "pageRowMore.9" && n.visible), "focused Rating shows chevrons");
    scene.pad(UiButton::Right);
    until(&mut scene, "Rating stepped to 5 of 5", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.9", "text") == "5 of 5");
    scene.pad(UiButton::Left);
    until(&mut scene, "Rating stepped back to 4 of 5", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.9", "text") == "4 of 5");

    // Step down to Clear Scraped Data (row 17): A opens confirmation Question; B cancels back to Edit Details.
    for _ in 0..10 {
        if focused(&mut scene) == "pageRow.17" {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "pageRow.17");
    assert_eq!(eclipse_board::read(&scene.tree(), "packagePlace", "text"), "18 of 18");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "true", 2000);
    until(&mut scene, "Clear Scraped Data question opened", 3000, |s| s.property("pageQuestion", "opened") == "true");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pageModel", "asking", "false", 2000);
    until(&mut scene, "back to Eclipse Edit after cancel", 3000, |s| s.property("theme", "pageDrawn") == "true");
    until(&mut scene, "edit row focus restored", 2000, |s| focused(s).starts_with("pageRow."));

    // Y on an edited field (Developer) resets it to scraped (empty here).
    let idx: usize = focused(&mut scene).strip_prefix("pageRow.").and_then(|s| s.parse().ok()).unwrap_or(0);
    if idx >= 9 {
        scene.pad(UiButton::Left);
        scene.wait(120);
    }
    for _ in 0..10 {
        let now: usize = focused(&mut scene).strip_prefix("pageRow.").and_then(|s| s.parse().ok()).unwrap_or(0);
        if now == 5 {
            break;
        }
        scene.pad(if now > 5 { UiButton::Up } else { UiButton::Down });
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "pageRow.5");
    scene.pad(UiButton::Search);
    until(&mut scene, "Developer reset by Y", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.5", "text").is_empty());

    // B returns to Eclipse Details tab.
    scene.pad(UiButton::Back);
    until(&mut scene, "B returns to Details tab", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && page_tab(s) == 3 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    assert_eq!(words_under(&scene.tree(), "help"), ["Choose", "Back", "Home"]);
    scene.assert_quiet();
}
