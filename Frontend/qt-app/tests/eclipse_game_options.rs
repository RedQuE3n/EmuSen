//! Eclipse's game page, its Options tab, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use emusen_kit_core::input::UiButton;
use game::{add_options, board_states, centre, focused, open_page, open_tab, program_with, until, words_under, world};

const TABLE: &str = include_str!("eclipse/game_options.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_options_is_its_board_and_every_row_steps_resets_and_opens() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-options", &states, true, true);
    let cores = add_options(&w);
    let mut scene = program_with(&mut harness, None, Some(&cores));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 4);
    assert_eq!(focused(&mut scene), "pageRow.0", "focus starts on Core");

    // Chevrons are visible only on the focused stepped row (row 0), hidden on resting stepped rows 1 and 3.
    let tree = scene.tree();
    assert!(tree.nodes.iter().any(|n| n.name == "pageRowLess.0" && n.visible), "focused Core shows left chevron");
    assert!(tree.nodes.iter().any(|n| n.name == "pageRowMore.0" && n.visible), "focused Core shows right chevron");
    assert!(!tree.nodes.iter().any(|n| n.name == "pageRowLess.1" && n.visible), "resting Controller profile hides chevrons");
    assert!(!tree.nodes.iter().any(|n| n.name == "pageRowLess.3" && n.visible), "resting Television standard hides chevrons");
    assert!(!tree.nodes.iter().any(|n| n.name == "pageRow.5" && n.visible), "Cheats row omitted from Options");
    assert_eq!(eclipse_board::read(&tree, "help-Back to the console's", "enabled"), "true", "X live on overridden Core");

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
        ("tabWords.4", "Options".to_owned(), false),
        ("pageRowName.0", "Core".to_owned(), false),
        ("pageRowWords.0", "Venus".to_owned(), false),
        ("pageRowName.1", "Controller profile".to_owned(), false),
        ("pageRowWords.1", "Console's".to_owned(), false),
        ("pageRowName.2", "Display crop".to_owned(), false),
        ("pageRowWords.2", "Console's".to_owned(), false),
        ("pageRowName.3", "Television standard".to_owned(), false),
        ("pageRowWords.3", "The game's own".to_owned(), false),
        ("pageRowName.4", "Package".to_owned(), false),
        ("pageRowWords.4", "Linked · 2 layers".to_owned(), false),
        ("gameAsideWords", "The core this game runs on; the console's own unless one is chosen here. Cores lists them all.".to_owned(), false),
        ("gameAsidePlace", "1 of 5".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(
        &mut scene,
        TABLE,
        &words,
        &["Change", "Value", "Back to the console's", "Back", "Home"],
        &["A", "◀ ▶", "X", "B", "MENU"],
        &mut differences,
    );
    game::picture(&mut scene, "game-options-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // X resets Core back to console's default and dims Back to the console's; Right/A/chevron clicks step values.
    scene.pad(UiButton::Screensaver);
    until(&mut scene, "Core reset to console's default", 2000, |s| {
        eclipse_board::read(&s.tree(), "help-Back to the console's", "enabled") == "false"
    });
    assert_eq!(eclipse_board::read(&scene.tree(), "help-Back to the console's", "opacity"), "0.38");

    // Step Controller profile with A (Change), chevron click, and reset with X.
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.1");
    assert_eq!(eclipse_board::read(&scene.tree(), "gameAsidePlace", "text"), "2 of 5");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "pageRowMore.1" && n.visible), "focused row 1 shows chevrons");
    assert_eq!(eclipse_board::read(&scene.tree(), "help-Back to the console's", "enabled"), "false");
    scene.pad(UiButton::Accept);
    until(&mut scene, "Controller profile stepped by A", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") != "Console's");
    assert_eq!(eclipse_board::read(&scene.tree(), "help-Back to the console's", "enabled"), "true");
    let before_click = eclipse_board::read(&scene.tree(), "pageRowWords.1", "text");
    let (mx, my) = centre(&mut scene, "pageRowMore.1");
    scene.click(mx, my);
    until(&mut scene, "Controller profile stepped by chevron click", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") != before_click);
    scene.pad(UiButton::Screensaver);
    until(&mut scene, "Controller profile reset by X", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.1", "text") == "Console's");

    // Display crop is a door row with A Open help; A opens Eclipse's This game's crop subpage and B returns.
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.2");
    assert_eq!(eclipse_board::read(&scene.tree(), "gameAsidePlace", "text"), "3 of 5");
    assert_eq!(words_under(&scene.tree(), "help"), ["Open", "Back", "Home"]);
    scene.pad(UiButton::Accept);
    until(&mut scene, "Eclipse Crop subpage opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "packageTitle", "text") == "This game's crop"
    });
    scene.pad(UiButton::Back);
    until(&mut scene, "back to Eclipse Options from Display crop", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });

    // Step Television standard with Right/Left and reset with X.
    until(&mut scene, "row focus restored", 2000, |s| focused(s).starts_with("pageRow."));
    while focused(&mut scene) != "pageRow.3" {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    assert_eq!(eclipse_board::read(&scene.tree(), "gameAsidePlace", "text"), "4 of 5");
    scene.pad(UiButton::Right);
    until(&mut scene, "Television standard stepped", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.3", "text") != "The game's own");
    scene.pad(UiButton::Screensaver);
    until(&mut scene, "Television standard reset", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.3", "text") == "The game's own");

    // Package is row 4 (underlying row 5 with Cheats omitted), door row with A Open; A opens Eclipse's Package subpage and B returns.
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.4");
    assert_eq!(eclipse_board::read(&scene.tree(), "gameAsidePlace", "text"), "5 of 5");
    assert_eq!(words_under(&scene.tree(), "help"), ["Open", "Back", "Home"]);
    scene.pad(UiButton::Accept);
    until(&mut scene, "Eclipse Package subpage opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    scene.pad(UiButton::Back);
    until(&mut scene, "back to Eclipse Options from Package", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });

    // Moving focus up to pills hides the aside.
    until(&mut scene, "row focus restored", 2000, |s| focused(s).starts_with("pageRow."));
    while focused(&mut scene) != "pill.0" {
        scene.pad(UiButton::Up);
        scene.wait(100);
    }
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "gameAside" && n.visible), "aside hidden on pills");
    scene.assert_quiet();
}
