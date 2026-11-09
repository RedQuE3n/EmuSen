//! The program at a library's scale (EmuSen_Qt.md §9.3.1): 9,000 synthetic games, hard links to one image. A step on
//! Choose a console builds the row's three covers and nothing per game of the console, and returns at once; the grid of every
//! game follows the focus, the focused cover always inside the view.

mod common;

use common::{large_sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use std::time::{Duration, Instant};

/// The longest a step may hold the GUI thread, far above a step's own cost and far below a walk of thousands of games.
const STEP: Duration = Duration::from_millis(400);

fn focused(scene: &mut emusen_qt_test::Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// The row's covers, made and shown or not, and every item of the scene.
fn counts(scene: &mut emusen_qt_test::Scene<'_>) -> (usize, usize) {
    let tree = scene.tree();
    (tree.nodes.iter().filter(|n| n.name.starts_with("rowCover.")).count(), tree.nodes.len())
}

#[test]
fn a_step_builds_nothing_per_game_and_the_grid_follows_the_focus() {
    let _world = large_sandbox("qt-app-scale", &[("NES", "nes", 6000), ("GB", "gb", 3000)]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 120000);
    assert_eq!(scene.property("library", "summary"), "2 consoles · 9,000 games");
    for _ in 0..40 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("screens", "shown"), "Consoles");
    scene.wait(100);

    // The NES (6,000) first, the Game Boy (3,000), the favourites (none), All games (9,000) and back: each step answered at once.
    let (fan, items) = counts(&mut scene);
    assert_eq!(fan, 3, "the first console shows three covers");
    let mut place = None;
    let walk = [
        (UiButton::Right, "gb", 3),
        (UiButton::Left, "nes", 3),
        (UiButton::Up, "favourites", 0),
        (UiButton::Left, "all", 3),
        (UiButton::Down, "nes", 3),
        (UiButton::Up, "favourites", 0),
        (UiButton::Left, "all", 3),
    ];
    for (button, key, fanned) in walk {
        let started = Instant::now();
        scene.pad(button);
        let took = started.elapsed();
        assert_eq!(scene.property("library", "selectedKey"), key);
        assert!(took < STEP, "the step onto {key} held the GUI thread {took:?}");
        scene.wait(100);
        let (fan, now) = counts(&mut scene);
        assert_eq!(fan, fanned, "{key} shows its first covers");
        if fanned == 0 {
            continue;
        }
        // A place and a lit world differ by their own marks alone; a console's games add nothing.
        let alike = if key == "all" { *place.get_or_insert(now) } else { items };
        assert_eq!(now, alike, "the scene on {key} holds as many items as on a row of another size");
    }

    // The grid of every game: read off the GUI thread, then Down held through its rows, the focus always in view.
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("screens", "shown"), "Grid");
    wait_for(&mut scene, "gridCells", "count", "9000", 10000);
    let view = |scene: &mut emusen_qt_test::Scene<'_>| {
        let tree = scene.tree();
        let cells = tree.nodes.iter().find(|n| n.name == "gridCells").map(|n| (n.y, n.h)).unwrap();
        (tree, cells)
    };
    let rows = {
        let tree = scene.tree();
        let y = |i: usize| tree.nodes.iter().find(|n| n.name == format!("gridCell.{i}")).map(|n| n.y).unwrap();
        (y(0), y(5))
    };
    for step in 1..=30 {
        scene.pad(UiButton::Down);
        let index: usize = scene.property("gridCells", "currentIndex").parse().unwrap();
        assert_eq!(index, 5 * step, "Down moves a row");
        let (tree, (top, height)) = view(&mut scene);
        let cell = tree.nodes.iter().find(|n| n.name == format!("gridCell.{index}") && n.visible);
        let cell = cell.unwrap_or_else(|| panic!("the focused cover {index} is made"));
        assert!(cell.y >= top && cell.y + cell.h <= top + height, "cover {index} at {}..{} outside the view {top}..{}", cell.y, cell.y + cell.h, top + height);
        // Whole rows: the focused cover stands where the first two rows stood, on the second once the view has moved.
        let second = rows.1;
        assert!((cell.y - second).abs() < 0.5, "step {step}: cover {index} at {}, the second row's place {second}", cell.y);
    }
    assert_eq!(scene.property("gridFocusTitle", "text"), "Game 00075");

    // The List card over 9,000 games: each step of Sort and Jump to and each search's words answer within 400 ms, the
    // listing read off the GUI thread (EmuSen_Qt.md §7.1, stage 2).
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 1000);
    scene.pad(UiButton::Down);
    for (by, sort) in [(UiButton::Right, "Last played"), (UiButton::Left, "Name")] {
        let next = (scene.property("gridGames", "revision").parse::<i32>().unwrap() + 1).to_string();
        let at = std::time::Instant::now();
        scene.pad(by);
        wait_for(&mut scene, "gridGames", "revision", &next, 2000);
        let took = at.elapsed().as_millis();
        assert!(took < 400, "Sort to {sort} took {took} ms over 9,000 games");
    }
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    let at = std::time::Instant::now();
    scene.pad(UiButton::Right);
    assert!(at.elapsed().as_millis() < 400, "Jump to took {} ms", at.elapsed().as_millis());
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    let at = std::time::Instant::now();
    // Game 00075 stands on both shelves.
    assert!(scene.set_property("gridGames", "search", "00075"));
    wait_for(&mut scene, "gridGames", "countLine", "2 of 9,000 games", 2000);
    assert!(at.elapsed().as_millis() < 400, "the search took {} ms over 9,000 games", at.elapsed().as_millis());
    assert!(scene.set_property("gridGames", "search", ""));
    wait_for(&mut scene, "gridGames", "countLine", "9,000 games", 2000);

    // Another row's grid opens at its first row.
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Down);
    assert_eq!(scene.property("library", "selectedKey"), "nes");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "gridCells", "count", "6000", 10000);
    scene.wait(50);
    let tree = scene.tree();
    let first = tree.nodes.iter().find(|n| n.name == "gridCell.0" && n.visible).map(|n| n.y);
    assert_eq!(first, Some(rows.0), "the NES's first cover where the first row stands");
    scene.assert_quiet();
}
