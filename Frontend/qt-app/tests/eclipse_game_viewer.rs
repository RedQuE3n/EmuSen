//! Eclipse's game page, Media Viewer, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use common::wait_for;
use emusen_kit_core::input::UiButton;
use game::{add_viewer_media, board_states, focused, open_page, open_tab, page_tab, program_with_media, until, words_under, world};

const TABLE: &str = include_str!("eclipse/game_viewer.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_viewer_is_its_board_and_steps_sets_cover_reads_manual_and_deletes() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-viewer", &states, true, true);
    let (store, shot1) = add_viewer_media(&mut harness, &w);
    let mut scene = program_with_media(&mut harness, Some(&store));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 2);
    until(&mut scene, "media.0 focused", 2000, |s| focused(s) == "media.0");
    scene.pad(UiButton::Right);
    scene.wait(150);
    assert_eq!(focused(&mut scene), "media.1");

    // Open Media Viewer on Screenshot 2 (2 of 5), pure black + 132px bar, date + 512 x 448 dimensions.
    scene.pad(UiButton::Accept);
    until(&mut scene, "Eclipse Media Viewer opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true"
            && s.property("pageModel", "viewing") == "true"
            && s.tree().nodes.iter().any(|n| n.name == "viewerCaption" && n.visible)
    });
    until(&mut scene, "viewer picture loaded", 3000, |s| eclipse_board::read(&s.tree(), "viewerPicture", "status") == "Ready");

    let words = [
        ("viewerCaption", "Screenshot 2 (2 of 5) · 1 November 2026 · 512 × 448".to_owned(), false),
        ("packagePlace", "2 of 5".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(
        &mut scene,
        TABLE,
        &words,
        &["Step", "Delete…", "Back"],
        &["◀ ▶", "Y", "B"],
        &mut differences,
    );
    game::picture(&mut scene, "game-viewer-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // Right steps to Scraped screenshot (3 of 5), then Title screen (4 of 5): shows X Set as cover and omits Y Delete….
    scene.pad(UiButton::Right);
    until(&mut scene, "stepped to Scraped screenshot", 2000, |s| {
        eclipse_board::read(&s.tree(), "viewerCaption", "text").starts_with("Scraped screenshot (3 of 5)")
    });
    scene.pad(UiButton::Right);
    until(&mut scene, "stepped to Title screen", 2000, |s| {
        eclipse_board::read(&s.tree(), "viewerCaption", "text").starts_with("Title screen (4 of 5)")
    });
    assert_eq!(words_under(&scene.tree(), "help"), ["Step", "Set as cover", "Back"]);
    scene.pad(UiButton::Screensaver);
    until(&mut scene, "Title screen set as cover; X hidden", 2000, |s| words_under(&s.tree(), "help") == ["Step", "Back"]);

    // Right steps to Clip (5 of 5): host-side VideoTile stage active, viewerBackdrop transparent, no X/Y.
    scene.pad(UiButton::Right);
    until(&mut scene, "stepped to Clip", 2000, |s| {
        eclipse_board::read(&s.tree(), "viewerCaption", "text").starts_with("Clip (5 of 5)")
    });
    assert_eq!(words_under(&scene.tree(), "help"), ["Step", "Back"]);
    assert!(scene.tree().nodes.iter().any(|n| n.name == "themeViewerClip" && n.visible), "host-side VideoTile active on Clip");
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "viewerBackdrop" && n.visible), "viewerBackdrop hidden over host Clip");

    // Up opens the 3-page PDF Manual via Kit.PdfPage; Right turns page; Down returns to media.
    scene.pad(UiButton::Up);
    until(&mut scene, "manual opened", 3000, |s| eclipse_board::read(&s.tree(), "viewerCaption", "text") == "Manual · Page 1 of 3");
    assert_eq!(eclipse_board::read(&scene.tree(), "packagePlace", "text"), "1 of 3");
    scene.pad(UiButton::Right);
    until(&mut scene, "manual turned to page 2", 2000, |s| eclipse_board::read(&s.tree(), "viewerCaption", "text") == "Manual · Page 2 of 3");
    scene.pad(UiButton::Down);
    until(&mut scene, "returned from manual", 2000, |s| eclipse_board::read(&s.tree(), "viewerCaption", "text").starts_with("Clip (5 of 5)"));

    // Step left back to Screenshot 1 (1 of 5), press Y Delete…, confirm Delete, file is removed from disk.
    for _ in 0..4 {
        scene.pad(UiButton::Left);
        scene.wait(120);
    }
    until(&mut scene, "back on Screenshot 1", 2000, |s| {
        eclipse_board::read(&s.tree(), "viewerCaption", "text").starts_with("Screenshot 1 (1 of 5)")
    });
    assert!(shot1.is_file(), "Screenshot 1 exists before delete");
    scene.pad(UiButton::Search);
    wait_for(&mut scene, "pageModel", "asking", "true", 2000);
    until(&mut scene, "Delete Screenshot 1 question focused on No", 3000, |s| {
        s.property("pageQuestion", "opened") == "true" && focused(s) == "answer.0"
    });
    scene.pad(UiButton::Down);
    scene.wait(100);
    assert_eq!(focused(&mut scene), "answer.1", "Delete answer focused");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "false", 2000);
    until(&mut scene, "Screenshot 1 deleted and viewer updated to 1 of 4", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "packagePlace", "text") == "1 of 4"
    });
    assert!(!shot1.exists(), "Screenshot 1 deleted from disk");

    // B closes viewer back to Eclipse Media tab.
    scene.pad(UiButton::Back);
    until(&mut scene, "B returns to Media tab", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && s.property("pageModel", "viewing") == "false" && page_tab(s) == 2
    });
    scene.assert_quiet();
}
