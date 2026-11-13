//! A game's Display crop and Package from its page in the program, the game not running: Options' Display crop opens
//! the game's own crop, stepped through Settings' overscan choices and stored in games.db for that game alone; Package
//! opens the game's package, and A on its job makes one through the library tool, its progress said on the row until it
//! is done and the package's row says what it now is.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::clients::game_crop;
use emusen_qt_models::display::Crop;
use emusen_qt_test::Scene;

const GAME: &str = "Amber Isle (Synthetic)";
const OTHER: &str = "Cobalt Harbor (Synthetic)";

fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..200 {
        if done(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("{what} within 10 s");
}

fn reads(scene: &mut Scene<'_>, words: &str) -> bool {
    scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(words))
}

/// What a reader hears of the item holding the focus.
fn heard(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default()
}

/// The focus moved down until what is heard starts with `row`.
fn down_to(scene: &mut Scene<'_>, row: &str) {
    for _ in 0..10 {
        if heard(scene).starts_with(row) {
            return;
        }
        scene.pad(UiButton::Down);
        scene.wait(80);
    }
    panic!("the focus never reached {row}: {}", heard(scene));
}

#[test]
fn the_game_page_sets_the_games_own_crop_and_makes_its_package() {
    let world = sandbox("qt-app-game-page-rows", &[("SNES", &format!("{GAME}.sfc")), ("SNES", &format!("{OTHER}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{GAME}.sfc"));
    let other = world.root.join("Roms/SNES").join(format!("{OTHER}.sfc"));
    let crop = |path: &std::path::Path| game_crop(&platform, path.to_str().unwrap()).unwrap();

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    scene.wait(300);
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
    assert_eq!(scene.property("pageModel", "title"), GAME);
    for _ in 0..2 {
        let now = scene.property("pageModel", "section");
        scene.pad(UiButton::PageDown);
        until(&mut scene, "the next section", |s| s.property("pageModel", "section") != now);
    }
    assert_eq!(scene.property("pageModel", "section"), "2", "Options");
    until(&mut scene, "the Display crop row", |s| reads(s, "Display crop") && reads(s, "Console's"));

    // A opens Display crop: Settings' row, the console's overscan until one is chosen for the game.
    down_to(&mut scene, "Display crop");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the crop's page", |s| reads(s, "Crop overscan") && !reads(s, "Television standard"));
    down_to(&mut scene, "Crop overscan");
    assert_eq!(crop(&rom), None);
    // Right: the whole picture, stored for this game alone.
    scene.pad(UiButton::Right);
    until(&mut scene, "the whole picture stored", |s| reads(s, "Whole picture") && crop(&rom) == Some(Crop::default()));
    assert_eq!(crop(&other), None, "the other game keeps its console's");
    // Right again: the part a television showed, as Serenity's catalogue sizes it, in percentages in games.db.
    scene.pad(UiButton::Right);
    until(&mut scene, "TV overscan stored", |s| reads(s, "TV overscan") && crop(&rom).is_some_and(|c| c.left > 0.03 && c.left < 0.034));
    let tv = crop(&rom).unwrap();
    assert!(tv.near(&Crop { left: tv.left, top: tv.left, right: tv.left, bottom: tv.left }), "an even crop: {tv:?}");
    assert_eq!(crop(&other), None);
    // B goes back to Options, the row now the game's own.
    scene.pad(UiButton::Back);
    until(&mut scene, "Options again", |s| reads(s, "This game's") && !reads(s, "Crop overscan"));
    assert_eq!(scene.property("screens", "shown"), "Page", "B left Display crop, not the page");

    // A opens Package: not a package yet; A on Make Package runs the library tool's job until it is done.
    down_to(&mut scene, "Package");
    assert!(reads(&mut scene, "Not a package"));
    scene.pad(UiButton::Accept);
    until(&mut scene, "the package's page", |s| reads(s, "Make Package"));
    down_to(&mut scene, "Make Package");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the package made", |s| reads(s, "Linked") && reads(s, "Check Package"));
    // The made package is checked against its manifest the same way.
    down_to(&mut scene, "Check Package");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the check done", |s| reads(s, "Done"));
    assert_eq!(scene.property("pageModel", "problem"), "", "nothing failed");
    scene.pad(UiButton::Back);
    until(&mut scene, "Options again", |s| reads(s, "Display crop") && reads(s, "Linked"));
    scene.assert_quiet();
}
