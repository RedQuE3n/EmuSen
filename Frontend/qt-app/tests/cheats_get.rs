//! The Cheats tool getting the database in the program, over the platform library built from this commit, a scratch
//! library and a loopback server of the test's own (never the real address): the List card gets it with the bar's
//! progress moving, a stop leaves no database, a second get installs it, A on the game makes its own list from its file
//! with its console's codecs, and the game's page then counts its cheats on.

mod common;

use common::cheats::{back_home, database_folder, focused, list_choose, open_cheats, until, GAME};
use common::{sandbox, wait_for};
use emusen_client_fixture::cheats::served;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;

#[test]
fn the_cheats_tool_gets_the_database_and_makes_a_games_list_that_its_page_counts() {
    let world = sandbox("qt-app-cheats-get", &[("SNES", &format!("{GAME}.sfc"))]);
    let server = served(&world);
    let folder = database_folder();
    assert!(!folder.exists(), "no database before the get: {}", folder.display());

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "cheatsUrl", &server.first_held));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    until(&mut scene, "Home holds the pad", 2000, |s| focused(s) == "place.0");
    open_cheats(&mut scene);
    assert!(scene.property("jobsModel", "facts").contains("no database"));

    // Get the database: the server holds back half, so the bar stands part way; Stop leaves no database.
    list_choose(&mut scene, "Get the database");
    until(&mut scene, "the download part way", 10000, |s| {
        let share: f64 = s.property("jobsModel", "progress").parse().unwrap_or(-1.0);
        s.property("jobsModel", "summary").starts_with("Downloading: ") && share > 0.0 && share < 1.0
    });
    list_choose(&mut scene, "Stop the download");
    wait_for(&mut scene, "jobsModel", "busy", "false", 10000);
    assert_eq!(scene.property("jobsModel", "summary"), "Stopped");
    assert!(scene.property("jobsModel", "facts").contains("no database"), "a stop installs nothing: {}", scene.property("jobsModel", "facts"));
    assert!(!folder.exists(), "the database's folder is not made by a stopped get");

    // Get it again, whole: installed, and the SNES game listed with no list of its own.
    list_choose(&mut scene, "Get the database");
    until(&mut scene, "the install and the read after", 20000, |s| s.property("jobsModel", "busy") == "false" && s.property("jobsModel", "rows").contains(GAME));
    assert_eq!(scene.property("jobsModel", "summary"), "Installed 1 cheat file(s)");
    assert!(scene.property("jobsModel", "rows").contains("No list yet"));

    // A on the game makes its own list from its file; the row counts its cheats on.
    scene.wait(300);
    scene.pad(UiButton::Accept);
    until(&mut scene, "the game's list made", 5000, |s| s.property("jobsModel", "rows").contains("1 of 2 on"));
    back_home(&mut scene);

    // The game's page counts the same cheats on Options' Cheats row.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    scene.wait(300);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    for _ in 0..2 {
        let now = scene.property("pageModel", "section");
        scene.pad(UiButton::PageDown);
        until(&mut scene, "the next section", 5000, |s| s.property("pageModel", "section") != now);
    }
    until(&mut scene, "the page's Cheats row", 5000, |s| s.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("1 of 2 on")));
    drop(server);
    drop(world);
}
