//! The List card's Most played, Year and Rating (EmuSen_Qt.md §7.9) on the software backend: they order the grid by
//! games.db's plays and the scraper's release date and rating, stepped with the pad and with the pointer, the card held
//! at the gate's five sizes. A game opened, failing or not, needs the GPU's picture: esde.rs walks the failure said, the
//! Home card's Continue, Cores and Themes.

mod common;

use common::console::row_centre;
use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::FILE_GAMES;
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};

/// No finding is named for the tester: the gate holds every state whole.
fn nothing_named(_: &str, _: &emusen_qt_test::tree::Tree, _: (i32, i32)) -> bool {
    false
}

const GAMES: [&str; 4] = ["Aurora Drift (Synthetic)", "Brass Lantern (Synthetic)", "Cobalt Harbor (Synthetic)", "Dune Relay (Synthetic)"];

/// The shown card's rows: each row's label and value, in order.
fn rows(scene: &mut Scene<'_>) -> Vec<String> {
    common::console::card_words(scene).into_iter().map(|(label, value)| if value.is_empty() { label } else { format!("{label} | {value}") }).collect()
}

/// The grid's games in the order shown: the focused title read as Right steps through them.
fn order(scene: &mut Scene<'_>) -> Vec<String> {
    let mut seen = Vec::new();
    for _ in 0..GAMES.len() {
        seen.push(scene.property("gridFocusTitle", "text"));
        scene.pad(UiButton::Right);
        scene.wait(60);
    }
    seen.iter().map(|t| t.trim_end_matches(" (Synthetic)").to_owned()).collect()
}

/// From the grid, Select's List card, Down to Sort, stepped until it reads `sort`, then closed; the grid's order.
fn sorted_by(scene: &mut Scene<'_>, sort: &str) -> Vec<String> {
    scene.pad(UiButton::Options);
    wait_for(scene, "screens", "shown", "List", 1000);
    scene.wait(400);
    for _ in 0..6 {
        if rows(scene).get(1).is_some_and(|r| r == &format!("Sort | {sort}")) {
            break;
        }
        // The pad steps the value; the pointer's click on the row steps it too.
        if sort == "Most played" {
            let (x, y) = row_centre(scene, "Sort");
            scene.click(x, y);
        } else {
            scene.pad(UiButton::Down);
            scene.pad(UiButton::Right);
            scene.pad(UiButton::Up);
        }
        scene.wait(150);
    }
    assert_eq!(rows(scene)[1], format!("Sort | {sort}"));
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Grid", 1000);
    // Year and Rating have the library read the scraper's facts again; the order is read once that read is in.
    scene.wait(300);
    wait_for(scene, "library", "loading", "false", 10000);
    scene.wait(300);
    order(scene)
}

fn open_all_games(scene: &mut Scene<'_>) {
    // Games, Home's first place, focused by name: the recently played shelf may hold the focus first.
    for _ in 0..40 {
        let tree = scene.tree();
        if let Some(games) = tree.find_named("place.0") {
            if scene.focus(&tree) == Some(games) || scene.focus_item(&tree, games) {
                break;
            }
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let place = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, place));
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Grid", 1000);
    wait_for(scene, "gridCells", "count", "4", 3000);
}

#[test]
fn qt_7_9_the_list_card_sorts_by_plays_year_and_rating() {
    let files: Vec<(&str, String)> = GAMES.iter().map(|g| ("NES", format!("{g}.nes"))).collect();
    let named: Vec<(&str, &str)> = files.iter().map(|(f, n)| (*f, n.as_str())).collect();
    let world = sandbox("qt-app-repairs", &named);
    let platform = Platform::load(library()).unwrap();
    // games.db's plays and the release dates and ratings as edits, which the scraper's metadata reads first.
    let sqlite = Lent::open(emusen_qt_models::clients::SQLITE).unwrap();
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    let facts = [("Aurora Drift", 3, "1994-03-01", "0.6"), ("Brass Lantern", 3, "1987-07-10", ""), ("Cobalt Harbor", 5, "", "0.9"), ("Dune Relay", 0, "1991-01-01", "0.2")];
    for (title, plays, date, rating) in facts {
        let path = world.root.join("Roms/NES").join(format!("{title} (Synthetic).nes"));
        let path = path.to_str().unwrap();
        for n in 0..plays {
            games.change(&serde_json::json!({ "op": "started", "path": path, "at": format!("2026-11-0{}T10:00:00Z", n + 1) })).unwrap();
        }
        let mut changes = serde_json::Map::new();
        if !date.is_empty() {
            changes.insert("releasedate".into(), date.into());
        }
        if !rating.is_empty() {
            changes.insert("rating".into(), rating.into());
        }
        games.change(&serde_json::json!({ "op": "save_edits", "path": path, "changes": changes, "at": "2026-11-09T00:00:00Z" })).unwrap();
    }
    drop(games);

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    open_all_games(&mut scene);
    let mut report = Vec::new();

    // Most played (by the pointer's click on Sort), Year and Rating (by the pad), each with a game without the datum
    // last; Aurora Drift and Brass Lantern were each played three times, a tie kept in the name's order.
    assert_eq!(sorted_by(&mut scene, "Most played"), ["Cobalt Harbor", "Aurora Drift", "Brass Lantern", "Dune Relay"]);
    assert_eq!(sorted_by(&mut scene, "Year"), ["Brass Lantern", "Dune Relay", "Aurora Drift", "Cobalt Harbor"]);
    assert_eq!(sorted_by(&mut scene, "Rating"), ["Cobalt Harbor", "Aurora Drift", "Dune Relay", "Brass Lantern"]);
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 1000);
    scene.wait(400);
    scene.pad(UiButton::Down);
    scene.wait(100);
    hold(&mut scene, "ListCardRating", &mockup::Mockup::default(), nothing_named, &mut report);
    assert!(report.is_empty(), "{} faults:\n{}", report.len(), report.join("\n"));
    scene.assert_quiet();
}
