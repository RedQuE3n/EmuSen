//! The program's screens through the harness (EmuSen_Qt.md §7.1): Home's Games place opens Choose a console, the walk
//! goes by the planet rows, A opens a console's grid, Y the favourites', and B goes back a screen at a time.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;

/// The focused item's name, from the tree, since a Repeater's items are not the window's QObject children.
fn focused(scene: &mut emusen_qt_test::Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

const HOME: &str = "Home";
const CONSOLES: &str = "Consoles";
const GRID: &str = "Grid";

#[test]
fn the_screens_switch_by_the_pad_and_the_walk_follows_the_rows() {
    let _world = sandbox("qt-app-screens", &[("NES", "One.nes"), ("NES", "Two.nes"), ("GB", "Three.gb")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert_eq!(scene.property("screens", "shown"), HOME);
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), "place.0", "no game suspended and none played: the places have the focus");
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("screens", "shown"), CONSOLES, "the Games place opens Choose a console");
    let places: usize = scene.property("library", "places").parse().unwrap();
    assert_eq!(focused(&mut scene), "tile.0", "the selected row has the focus");

    scene.pad(UiButton::Down);
    assert_eq!(scene.property("library", "selectedIndex"), places.to_string(), "Down goes to the first planet");
    let key = scene.property("library", "selectedKey");
    scene.pad(UiButton::Right);
    assert_eq!(scene.property("library", "selectedIndex"), (places + 1).to_string(), "Right walks along the row");
    scene.pad(UiButton::Left);
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("screens", "shown"), GRID);
    assert_eq!(scene.property("screens", "key"), key);
    assert!(!scene.property("grid", "name").is_empty(), "the grid's heading names the console it opened");
    wait_for(&mut scene, "gridCells", "count", "2", 2000);

    scene.pad(UiButton::Back);
    assert_eq!(scene.property("screens", "shown"), CONSOLES, "B goes back to Choose a console");
    assert_eq!(scene.property("library", "selectedKey"), key, "on the console it left");
    scene.pad(UiButton::Search);
    assert_eq!((scene.property("screens", "shown").as_str(), scene.property("screens", "key").as_str()), (GRID, "favourites"), "Y opens the favourites");
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    assert_eq!(scene.property("screens", "shown"), HOME, "B again goes Home");
    wait_for(&mut scene, "home", "activeFocus", "true", 1000);
    scene.assert_quiet();
}
