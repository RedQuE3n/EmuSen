//! Eclipse's game page (Overview, States, Media, Details, Options, Cheats, Package) held to its boards on the graphics card (EmuSen_Qt.md §3.4.12).

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use emusen_kit_core::input::UiButton;
use emusen_qt_test::compositor::Compositor;
use game::{add_board_media, add_cheats, add_media, add_options, board_states, focused, open_page, open_tab, program_with, set_description, states_tab, until, world, BOARD_CHEATS, BOARD_DESCRIPTION};

const TABLE_OVERVIEW: &str = include_str!("eclipse/game_overview.tsv");
const TABLE_STATES: &str = include_str!("eclipse/game.tsv");
const TABLE_MEDIA: &str = include_str!("eclipse/game_media.tsv");
const TABLE_DETAILS: &str = include_str!("eclipse/game_details.tsv");
const TABLE_OPTIONS: &str = include_str!("eclipse/game_options.tsv");
const TABLE_CHEATS: &str = include_str!("eclipse/game_cheats.tsv");
const TABLE_PACKAGE: &str = include_str!("eclipse/game_package.tsv");

#[test]
fn qt_3_4_12_eclipse_s_game_page_is_its_board_at_every_size_on_the_card() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = emusen_qt_test::Harness::new();
    let states = board_states();
    let mut differences = Vec::new();

    {
        let w = world("qt-app-eclipse-game-gpu", &states, true, true);
        set_description(&w, BOARD_DESCRIPTION);
        let cores = add_options(&w);
        let media = add_media(&mut harness, &w, &["covers", "screenshots"]);
        let mut scene = program_with(&mut harness, Some(&media), Some(&cores));
        open_page(&mut scene, &w);

        // Overview (#77).
        assert_eq!(focused(&mut scene), "pill.0", "Resume focused on Overview");
        game::hold_board(&mut scene, TABLE_OVERVIEW, &[], &["Resume", "Favourite", "Back", "Home"], &["A", "Y", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-overview-1920x1200-card.png");

        // States (#78).
        states_tab(&mut scene);
        scene.pad(UiButton::Right);
        scene.wait(200);
        assert_eq!(focused(&mut scene), "card.1", "slot 1 focused, as the board draws it");
        game::hold_board(&mut scene, TABLE_STATES, &[], &["Load this state", "Delete", "Back"], &["A", "X", "B"], &mut differences);
        game::picture(&mut scene, "game-1920x1200-card.png");

        // Details (#79).
        open_tab(&mut scene, 3);
        scene.pad(UiButton::Down);
        scene.wait(200);
        assert_eq!(focused(&mut scene), "pageRow.1", "Edit Details focused, as the board draws it");
        game::hold_board(&mut scene, TABLE_DETAILS, &[], &["Open", "Back", "Home"], &["A", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-details-1920x1200-card.png");

        // Options (#80).
        open_tab(&mut scene, 4);
        assert_eq!(focused(&mut scene), "pageRow.0", "Core focused, as the board draws it");
        game::hold_board(
            &mut scene,
            TABLE_OPTIONS,
            &[],
            &["Change", "Value", "Back to the console's", "Back", "Home"],
            &["A", "◀ ▶", "X", "B", "MENU"],
            &mut differences,
        );
        game::picture(&mut scene, "game-options-1920x1200-card.png");
        scene.assert_quiet();
    }

    {
        let w = world("qt-app-eclipse-game-gpu-g3", &states, true, true);
        let cores = add_options(&w);
        let media_store = add_board_media(&mut harness, &w);
        add_cheats(&w, &BOARD_CHEATS);
        let mut scene = program_with(&mut harness, Some(&media_store), Some(&cores));
        open_page(&mut scene, &w);

        // Media (#78).
        open_tab(&mut scene, 2);
        until(&mut scene, "Media tile 0 focused", 2000, |s| focused(s) == "media.0");
        scene.pad(UiButton::Right);
        scene.wait(200);
        assert_eq!(focused(&mut scene), "media.1", "Screenshot 2 focused, as the board draws it");
        game::hold_board(&mut scene, TABLE_MEDIA, &[], &["View", "Set as cover", "Back", "Home"], &["A", "X", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-media-1920x1200-card.png");

        // Cheats (#81).
        open_tab(&mut scene, 5);
        until(&mut scene, "Cheats row 0 focused", 2000, |s| focused(s) == "pageRow.0");
        scene.pad(UiButton::Down);
        scene.wait(200);
        assert_eq!(focused(&mut scene), "pageRow.1", "Start with all keys focused, as the board draws it");
        game::hold_board(&mut scene, TABLE_CHEATS, &[], &["Turn off", "All cheats", "Back", "Home"], &["A", "X", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-cheats-1920x1200-card.png");

        // Package (#83).
        open_tab(&mut scene, 4);
        until(&mut scene, "Options row 0 focused", 2000, |s| focused(s) == "pageRow.0");
        while focused(&mut scene) != "pageRow.4" {
            scene.pad(UiButton::Down);
            scene.wait(100);
        }
        scene.pad(UiButton::Accept);
        until(&mut scene, "Package subpage opened", 3000, |s| focused(s) == "pageRow.1" && s.tree().nodes.iter().any(|n| n.name == "packageTitle" && n.visible));
        game::hold_board(&mut scene, TABLE_PACKAGE, &[], &["Check", "Options", "Home"], &["A", "B", "MENU"], &mut differences);
        game::picture(&mut scene, "game-package-1920x1200-card.png");
        scene.assert_quiet();
    }

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
