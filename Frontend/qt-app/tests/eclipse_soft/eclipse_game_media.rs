//! Eclipse's game page, its Media tab, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

use crate::eclipse_board;
use crate::game;

use emusen_kit_core::input::UiButton;
use game::{add_board_media, board_states, focused, open_page, open_tab, program, program_with_media, until, words_under, world};

const TABLE: &str = include_str!("../eclipse/game_media.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_media_is_its_board_and_handles_cover_viewer_and_empty() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    {
        let w = world("qt-app-eclipse-game-media", &states, true, true);
        let media_store = add_board_media(&mut harness, &w);
        let mut scene = program_with_media(&mut harness, Some(&media_store));
        open_page(&mut scene, &w);
        open_tab(&mut scene, 2);
        until(&mut scene, "focus starts on media.0", 2000, |s| focused(s) == "media.0");
        assert_eq!(eclipse_board::read(&scene.tree(), "gameMediaPlace", "text"), "1 of 5");
        assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "enabled"), "false", "taken shot ineligible");
        assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "opacity"), "0.38");

        scene.pad(UiButton::Right);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "media.1", "Screenshot 2 focused, as the board draws it");
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
            ("tabWords.2", "Media".to_owned(), false),
            ("gameMediaPlace", "2 of 5".to_owned(), false),
            ("mediaName.0", "Screenshot 1".to_owned(), false),
            ("mediaName.1", "Screenshot 2".to_owned(), false),
            ("mediaName.2", "Title screen".to_owned(), false),
            ("mediaName.3", "Clip".to_owned(), false),
            ("mediaName.4", "Manual".to_owned(), false),
        ];
        let mut differences = Vec::new();
        game::hold_board(&mut scene, TABLE, &words, &["View", "Set as cover", "Back", "Home"], &["A", "X", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-media-1920x1200-software.png");
        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

        // Title screen (tile 2) is eligible for X Set as cover; pressing X sets it and dims the chip.
        scene.pad(UiButton::Right);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "media.2");
        assert_eq!(eclipse_board::read(&scene.tree(), "gameMediaPlace", "text"), "3 of 5");
        assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "enabled"), "true");
        assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "opacity"), "1");
        scene.pad(UiButton::Screensaver);
        until(&mut scene, "Title screen set as cover and X dimmed", 2000, |s| {
            eclipse_board::read(&s.tree(), "help-Set as cover", "enabled") == "false"
        });
        assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "opacity"), "0.38");

        // Clip (tile 3) and Manual (tile 4) are ineligible (dimmed X).
        for (idx, place) in [(3, "4 of 5"), (4, "5 of 5")] {
            scene.pad(UiButton::Right);
            scene.wait(150);
            assert_eq!(focused(&mut scene), format!("media.{idx}"));
            assert_eq!(eclipse_board::read(&scene.tree(), "gameMediaPlace", "text"), place);
            assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "enabled"), "false");
            assert_eq!(eclipse_board::read(&scene.tree(), "help-Set as cover", "opacity"), "0.38");
        }

        // Moving focus up to pills or trail hides the count; moving back down restores it.
        scene.pad(UiButton::Up);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "pill.0");
        assert!(!scene.tree().nodes.iter().any(|n| n.name == "gameMediaPlace" && n.visible), "count hidden on pills");
        scene.pad(UiButton::Up);
        scene.wait(150);
        assert!(!scene.tree().nodes.iter().any(|n| n.name == "gameMediaPlace" && n.visible), "count hidden on trail");
        scene.pad(UiButton::Down);
        scene.wait(120);
        scene.pad(UiButton::Down);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "media.0");
        assert!(scene.tree().nodes.iter().any(|n| n.name == "gameMediaPlace" && n.visible), "count shown on media tile");

        // A opens Eclipse's Media Viewer; B closes it and restores Eclipse Media with refreshed help.
        scene.pad(UiButton::Accept);
        until(&mut scene, "Eclipse viewer opened", 3000, |s| {
            s.property("theme", "pageDrawn") == "true"
                && s.property("pageModel", "viewing") == "true"
                && s.tree().nodes.iter().any(|n| n.name == "viewerCaption" && n.visible)
        });
        scene.pad(UiButton::Back);
        until(&mut scene, "back to Eclipse Media from viewer", 3000, |s| {
            s.property("theme", "pageDrawn") == "true" && s.property("pageModel", "viewing") == "false"
        });
        until(&mut scene, "help chips refreshed after viewer close", 2000, |s| words_under(&s.tree(), "help") == ["View", "Set as cover", "Back", "Home"]);
        scene.assert_quiet();
    }

    // Empty media state (0 items): muted note shown, grid and count hidden, focus stays on pill.0.
    let empty = world("qt-app-eclipse-game-media-empty", &states, true, true);
    let mut scene = program(&mut harness);
    open_page(&mut scene, &empty);
    open_tab(&mut scene, 2);
    until(&mut scene, "empty media note visible", 2000, |s| s.tree().nodes.iter().any(|n| n.name == "gameMediaNote" && n.visible));
    let tree = scene.tree();
    assert_eq!(eclipse_board::read(&tree, "gameMediaNote", "text"), emusen_qt_models::game_page::NO_MEDIA);
    assert!(!tree.nodes.iter().any(|n| n.name == "gameMedia" && n.visible), "empty media hides tile row");
    assert!(!tree.nodes.iter().any(|n| n.name == "gameMediaPlace" && n.visible), "empty media hides count");
    assert_eq!(focused(&mut scene), "pill.0", "focus stays on Resume when media is empty");
    assert_eq!(words_under(&tree, "help"), ["Resume", "Favourite", "Back", "Home"]);
    scene.assert_quiet();
}
