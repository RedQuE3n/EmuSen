//! Play run from a theme's game page, apart from theme_pages.rs, which it would leave (EmuSen_Qt.md §3.2, the proof).

mod common;

use common::{first_game_page, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;
use cxx_qt::casting::Upcast;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::QAbstractListModel;

const PAGES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-host/tests/themes/pages");

#[test]
fn qt_3_2_a_theme_s_game_page_plays_the_game() {
    let _world = sandbox("qt-app-theme-pages-play", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", PAGES));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert_eq!(scene.property("theme", "failed"), "false", "{}", scene.property("theme", "warnings"));
    scene.wait(300);
    first_game_page(&mut scene);
    wait_for(&mut scene, "theme", "game", "true", 2000);
    wait_for(&mut scene, "gameReady", "text", "true", 10000);

    let object = scene.object("pageModel");
    assert!(!object.is_null());
    // Safety: the window's model of that name is a GamePageModel, alive while the window is.
    let page = unsafe { &*GamePageModel::from_base_ptr(object as *const QAbstractListModel) };
    let play = page.shown_actions().iter().position(|a| a.label() == "Play").expect("Play among the actions");
    assert!(scene.set_property("pagesGame", "command", &format!("pageAct {play}")));
    scene.wait(60);
    assert_eq!(scene.property("pagesGame", "took"), "true");
    wait_for(&mut scene, "screens", "shown", "Game", 3000);
    assert_eq!(scene.property("session", "startState"), "", "Play starts the game from its start, no state named");
    scene.assert_quiet();
}
