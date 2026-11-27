//! Eclipse's game page, the Package deeper page, held to its board and interactions (EmuSen_Qt.md §3.4.12).

use crate::eclipse_board;
use crate::game;

use emusen_kit_core::input::UiButton;
use game::{add_options, board_states, focused, open_page, open_tab, page_tab, program, program_with, until, words_under, world};

const TABLE: &str = include_str!("../eclipse/game_package.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_package_is_its_board_and_runs_check_and_make() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    // Linked 2-layer package: Check Package row 1 focused, row 0 read-only, job runs, B returns to Options.
    {
        let w = world("qt-app-eclipse-game-package", &states, true, true);
        let cores = add_options(&w);
        let mut scene = program_with(&mut harness, None, Some(&cores));
        open_page(&mut scene, &w);
        open_tab(&mut scene, 4);
        until(&mut scene, "focus starts on Options row 0", 2000, |s| focused(s) == "pageRow.0");
        while focused(&mut scene) != "pageRow.4" {
            scene.pad(UiButton::Down);
            scene.wait(100);
        }
        scene.pad(UiButton::Accept);
        until(&mut scene, "Eclipse Package subpage opened with row 1 focused", 3000, |s| {
            s.property("theme", "pageDrawn") == "true" && focused(s) == "pageRow.1" && s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
        });
        assert_eq!(eclipse_board::read(&scene.tree(), "pageRowWords.1", "text"), "", "idle ellipsis stripped");

        let words = [
            ("trailWords.0", "Consoles".to_owned(), false),
            ("trailWords.1", "Super Nintendo".to_owned(), false),
            ("trailWords.2", "Aurora Drift".to_owned(), false),
            ("trailWords.3", "Package".to_owned(), false),
            ("battery", "82%".to_owned(), false),
            ("clock", "9:41 PM".to_owned(), false),
            ("packageTitle", "Package".to_owned(), false),
            ("packageSubtitle", "The game's package and its layers, from Packages.".to_owned(), false),
            ("pageRowName.0", "Package".to_owned(), false),
            ("pageRowWords.0", "Linked · 2 layers".to_owned(), false),
            ("pageRowName.1", "Check Package".to_owned(), false),
            ("packagePlace", "2 of 2".to_owned(), false),
        ];
        let mut differences = Vec::new();
        game::hold_board(&mut scene, TABLE, &words, &["Check", "Options", "Home"], &["A", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-package-1920x1200-software.png");
        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

        // Row 0 is read-only: 1 of 2 and B Options | START Home (no A).
        scene.pad(UiButton::Up);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "pageRow.0");
        assert_eq!(eclipse_board::read(&scene.tree(), "packagePlace", "text"), "1 of 2");
        assert_eq!(words_under(&scene.tree(), "help"), ["Options", "Home"]);
        assert_eq!(game::glyphs(&scene.tree(), &["ownGlyph-Options", "ownGlyph-Home"]), ["B", "MENU"]);

        // Moving focus up to the trail link hides the footer count; Down restores it.
        scene.pad(UiButton::Up);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "trailLink.2");
        assert!(!scene.tree().nodes.iter().any(|n| n.name == "packagePlace" && n.visible), "count hidden on trail");
        scene.pad(UiButton::Down);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "pageRow.0");

        // Row 1 runs Check Package and shows live/done status words (never idle ellipsis).
        scene.pad(UiButton::Down);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "pageRow.1");
        scene.pad(UiButton::Accept);
        until(&mut scene, "Check Package reports progress or completion", 5000, |s| {
            let v = eclipse_board::read(&s.tree(), "pageRowWords.1", "text");
            !v.is_empty() && v != "…"
        });

        // B returns to Eclipse Options tab with refreshed help chips.
        scene.pad(UiButton::Back);
        until(&mut scene, "B returns to Options tab", 3000, |s| {
            s.property("theme", "pageDrawn") == "true" && page_tab(s) == 4 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
        });
        scene.assert_quiet();
    }

    // Plain (unpackaged) game: Package shows Not a package + Make Package with A Make.
    let plain = world("qt-app-eclipse-game-package-make", &states, true, false);
    let mut scene = program(&mut harness);
    open_page(&mut scene, &plain);
    open_tab(&mut scene, 4);
    until(&mut scene, "Options row focused", 2000, |s| focused(s).starts_with("pageRow."));
    for _ in 0..6 {
        let name = eclipse_board::read(&scene.tree(), &focused(&mut scene).replace("pageRow.", "pageRowName."), "text");
        if name == "Package" {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, "Make Package subpage opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && focused(s) == "pageRow.1" && s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    let tree = scene.tree();
    assert_eq!(eclipse_board::read(&tree, "pageRowWords.0", "text"), "Not a package");
    assert_eq!(eclipse_board::read(&tree, "pageRowName.1", "text"), "Make Package");
    assert_eq!(eclipse_board::read(&tree, "pageRowWords.1", "text"), "", "idle Make omits ellipsis");
    assert_eq!(eclipse_board::read(&tree, "packagePlace", "text"), "2 of 2");
    assert_eq!(words_under(&tree, "help"), ["Make", "Options", "Home"]);
    scene.pad(UiButton::Back);
    until(&mut scene, "B returns from Make Package to Options", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && page_tab(s) == 4 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    scene.assert_quiet();
}
