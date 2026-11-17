//! Eclipse's games view with no game in the library (EmuSen_Qt.md §3.4.10): the empty board's rows of `eclipse/empty.tsv` held to the screen at each of the gate's sizes, and the program's words; the theme's fit and contrast are theme-check's loaded step. With `ECLIPSE_GAMES_PICTURES` set, the state is saved there at 1280x800 and 1920x1200.

mod common;
#[path = "eclipse/table.rs"]
mod table;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_models::{themes, words};
use emusen_qt_test::harness::SIZES;

#[test]
fn qt_3_4_10_eclipses_games_view_with_no_game_is_the_empty_board_by_the_table() {
    let _world = sandbox("qt-app-eclipse-games-empty", &[]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    for _ in 0..40 {
        let tree = scene.tree();
        if tree.find_named("place.0").is_some_and(|games| scene.focus(&tree) == Some(games)) {
            break;
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All games, a shelf shown with no console");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    // Eclipse chosen on the games view, reached through the reference's Choose a console: Eclipse's Home has no places until the top navigation's piece (R-20261115-7).
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    let warnings = scene.property("host.theme", "warnings");
    assert!(warnings.is_empty() && scene.property("host.theme", "failed") == "false", "Eclipse's screens load clean: {warnings}");
    wait_for(&mut scene, "emptyHeading", "visible", "true", 3000);

    assert_eq!(scene.property("emptyHeading", "text"), words::NO_GAMES);
    assert_eq!(scene.property("emptyLine", "text"), words::ADD_FOLDER);
    for hidden in ["gamesGrid", "gamesAside", "gamesFilters", "gamesSort", "gamesRail"] {
        assert_eq!(scene.property(hidden, "visible"), "false", "{hidden} with no game");
    }

    let pictures = std::env::var_os("ECLIPSE_GAMES_PICTURES").map(std::path::PathBuf::from);
    let rows = table::built();
    let held: Vec<&table::Row> = rows.iter().filter(|r| r.empty_board() && r.state().is_none_or(|s| s == "no-library")).collect();
    assert_eq!(held.len(), 6, "the empty board's rows");
    let mut report = Vec::new();
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(200);
        report.extend(table::differences(&mut scene, &held, f64::from(w), f64::from(h)).into_iter().map(|d| format!("{w}x{h}: {d}")));
        if let Some(dir) = pictures.as_ref().filter(|_| (w, h) == (1280, 800) || (w, h) == (1920, 1200)) {
            std::fs::create_dir_all(dir).unwrap();
            assert!(scene.grab(&dir.join(format!("empty-{w}x{h}.png")).display().to_string()));
        }
    }
    scene.resize(1280, 800);
    scene.wait(200);
    assert!(report.is_empty(), "{}", report.join("\n"));

}
