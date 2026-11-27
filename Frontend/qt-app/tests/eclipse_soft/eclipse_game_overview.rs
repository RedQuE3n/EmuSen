//! Eclipse's game page, its Overview tab, held to its board and undrawn states (EmuSen_Qt.md §3.4.12).

use crate::common;
use crate::eclipse_board;
use crate::game;

use emusen_kit_core::input::UiButton;
use game::{add_media, board_states, focused, open_page, program, program_with_media, set_description, world, BOARD_DESCRIPTION};

const TABLE: &str = include_str!("../eclipse/game_overview.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_overview_is_its_board_at_every_size_and_handles_missing_parts() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    {
        let w = world("qt-app-eclipse-game-overview", &states, true, true);
        set_description(&w, BOARD_DESCRIPTION);
        let media = add_media(&mut harness, &w, &["covers", "screenshots"]);
        let mut scene = program_with_media(&mut harness, Some(&media));
        open_page(&mut scene, &w);
        assert_eq!(scene.property("theme", "pageDrawn"), "true", "Overview is Eclipse's");
        assert_eq!(focused(&mut scene), "pill.0", "focus starts on Resume");
        scene.pad(UiButton::Down);
        scene.wait(120);
        assert_eq!(focused(&mut scene), "pill.0", "Down stays on the pills on Overview");
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
            ("gameOverviewWords", BOARD_DESCRIPTION.to_owned(), false),
        ];
        let mut differences = Vec::new();
        game::hold_board(&mut scene, TABLE, &words, &["Resume", "Favourite", "Back", "Home"], &["A", "Y", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-overview-1920x1200-software.png");
        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        scene.assert_quiet();
    }

    // Undrawn states: no description shows muted placeholder; no screenshot hides art box and keeps description 900 wide.
    let empty = world("qt-app-eclipse-game-overview-empty", &states, true, false);
    let mut scene = program(&mut harness);
    open_page(&mut scene, &empty);
    {
        let tree = scene.tree();
        assert_eq!(eclipse_board::read(&tree, "gameOverviewWords", "text"), emusen_qt_models::game_page::NO_DESCRIPTION);
        assert_eq!(eclipse_board::read(&tree, "gameOverviewWords", "color").to_ascii_uppercase(), "#B9BED0");
        assert!(!tree.nodes.iter().any(|n| n.name == "gameOverviewArt" && n.visible), "empty picture hides the 560x315 box");
        let unit: f64 = eclipse_board::read(&tree, "gameCanvas", "unit").parse().unwrap();
        let k = emusen_kit_core::canvas::DESIGN_HEIGHT / 1200.0;
        let desc = tree.nodes.iter().rev().find(|n| n.name == "gameOverviewWords" && n.visible).expect("description drawn");
        assert!(((desc.w / unit) - 900.0 * k).abs() < 1.0, "description stays 900 wide without art: {}", desc.w / unit);
    }

    // Long description elides at 10 lines without touching the footer.
    scene.pad(UiButton::Back);
    common::wait_for(&mut scene, "screens", "shown", "Home", 3000);
    let long_text = vec![BOARD_DESCRIPTION; 14].join(" ");
    set_description(&empty, &long_text);
    open_page(&mut scene, &empty);
    {
        let tree = scene.tree();
        assert_eq!(eclipse_board::read(&tree, "gameOverviewWords", "truncated"), "true", "long description elides");
        let lines: i32 = eclipse_board::read(&tree, "gameOverviewWords", "lineCount").parse().unwrap();
        assert!(lines <= 10, "maximum 10 lines: {lines}");
        let desc = tree.nodes.iter().rev().find(|n| n.name == "gameOverviewWords" && n.visible).expect("description drawn");
        let help = tree.nodes.iter().rev().find(|n| n.name == "help" && n.visible).expect("help drawn");
        assert!(desc.y + desc.h < help.y, "description stays above footer: {} < {}", desc.y + desc.h, help.y);
    }
    scene.assert_quiet();
}
