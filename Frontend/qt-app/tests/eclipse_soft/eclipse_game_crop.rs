//! Eclipse's game page, This game's crop, held to its board and pad/pointer interactions (EmuSen_Qt.md §3.4.12).

use crate::common;
use crate::eclipse_board;
use crate::game;

use common::wait_for;
use emusen_kit_core::input::UiButton;
use emusen_qt_models::display::Crop;
use game::{add_media, add_options, board_states, focused, open_page, open_tab, page_tab, program_with, set_crop, until, words_under, world};

const TABLE: &str = include_str!("../eclipse/game_crop.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_crop_is_its_board_and_steps_by_half_percent_and_types() {
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let w = world("qt-app-eclipse-game-crop", &states, true, true);
    let cores = add_options(&w);
    let media = add_media(&mut harness, &w, &["screenshots"]);
    set_crop(&w, Crop { left: 0.0, right: 0.0, top: 0.025, bottom: 0.025 });
    let mut scene = program_with(&mut harness, Some(&media), Some(&cores));
    open_page(&mut scene, &w);
    open_tab(&mut scene, 4);
    until(&mut scene, "Options row 0 focused", 2000, |s| focused(s) == "pageRow.0");
    while focused(&mut scene) != "pageRow.2" {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, "Eclipse This game's crop opened", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "packageTitle", "text") == "This game's crop"
    });
    until(&mut scene, "crop row 0 focused", 2000, |s| focused(s) == "pageRow.0");
    assert_eq!(eclipse_board::read(&scene.tree(), "packageSubtitle", "text"), emusen_qt_models::game_page::CROP_TOGGLE_SUBTITLE);
    assert_eq!(words_under(&scene.tree(), "help"), ["Value", "Change", "Back"]);

    // Step down to Top (row 3), as the board draws it: Top 2.5% active, Bottom 2.5% dim, 4 of 5.
    for _ in 0..3 {
        scene.pad(UiButton::Down);
        scene.wait(100);
    }
    assert_eq!(focused(&mut scene), "pageRow.3");

    let words = [
        ("trailWords.0", "Consoles".to_owned(), false),
        ("trailWords.1", "Super Nintendo".to_owned(), false),
        ("trailWords.2", "Aurora Drift".to_owned(), false),
        ("trailWords.3", "Options".to_owned(), false),
        ("trailWords.4", "This game's crop".to_owned(), false),
        ("battery", "82%".to_owned(), false),
        ("clock", "9:41 PM".to_owned(), false),
        ("packageTitle", "This game's crop".to_owned(), false),
        ("packageSubtitle", "The part of the picture hidden at its top edge, 0 to 25%; A types it.".to_owned(), false),
        ("pageRowName.0", "This game's crop".to_owned(), false),
        ("pageRowWords.0", "On".to_owned(), false),
        ("pageRowName.3", "Top".to_owned(), false),
        ("pageRowWords.3", "2.5%".to_owned(), false),
        ("cropCaption", "Aurora Drift · Top 2.5%".to_owned(), false),
        ("packagePlace", "4 of 5".to_owned(), false),
    ];
    let mut differences = Vec::new();
    game::hold_board(
        &mut scene,
        TABLE,
        &words,
        &["Value", "Type it", "Back"],
        &["◀ ▶", "A", "B"],
        &mut differences,
    );
    game::picture(&mut scene, "game-crop-1920x1200-software.png");
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // Right steps Top by 0.5% to 3%; Left steps back to 2.5%.
    scene.pad(UiButton::Right);
    until(&mut scene, "Top stepped to 3%", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.3", "text") == "3%");
    assert_eq!(eclipse_board::read(&scene.tree(), "cropCaption", "text"), "Aurora Drift · Top 3%");
    scene.pad(UiButton::Left);
    until(&mut scene, "Top stepped back to 2.5%", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.3", "text") == "2.5%");

    // A on Top opens keyboard to type a percentage (e.g. "4"), X saves it.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "typing", "true", 2000);
    let current = scene.property("pageTyped", "text");
    for _ in 0..current.len() {
        scene.key(0x0100_0003, 0);
    }
    scene.key('4' as i32, 0);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "typing", "false", 2000);
    until(&mut scene, "Top typed to 4%", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && eclipse_board::read(&s.tree(), "pageRowWords.3", "text") == "4%"
    });

    // Turning row 0 Off keeps all 4 edge rows visible; stepping Left from Off turns crop back On at 0.5%.
    while focused(&mut scene) != "pageRow.0" {
        scene.pad(UiButton::Up);
        scene.wait(100);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, "crop turned Off", 2000, |s| eclipse_board::read(&s.tree(), "pageRowWords.0", "text") == "Off");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "pageRow.4" && n.visible), "all 4 edge rows stay visible when Off");
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(focused(&mut scene), "pageRow.1");
    scene.pad(UiButton::Right);
    until(&mut scene, "stepping edge from Off turns crop On at 0.5%", 2000, |s| {
        eclipse_board::read(&s.tree(), "pageRowWords.0", "text") == "On" && eclipse_board::read(&s.tree(), "pageRowWords.1", "text") == "0.5%"
    });

    // B returns to Eclipse Options tab.
    scene.pad(UiButton::Back);
    until(&mut scene, "B returns to Options tab", 3000, |s| {
        s.property("theme", "pageDrawn") == "true" && page_tab(s) == 4 && !s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible)
    });
    scene.assert_quiet();
}
