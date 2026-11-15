//! The None card of Cover Art, in the program over the platform library built from this commit (EmuSen_Qt.md §7.8.5;
//! R-20261114-28): a game with no cover found and its title screen chosen as its cover shows None in the found cover's
//! place; the pad reaches it, nothing is cut off or overlapping at the gate's window sizes, and A on it clears the choice
//! in games.db, after which the card goes. With `COVER_NONE_GRAB` set, the page with None focused is saved there.

mod common;

use common::cover_art::{cards, judged_at_every_size, move_to, open_cover_art, open_page, press, until};
use common::{draw_covers, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_test::harness::SIZES;
use std::path::PathBuf;

const COBALT: &str = "Cobalt Harbor (Synthetic)";
/// The game's own pictures, each by its folder; no cover is drawn, so none is found.
const FOLDERS: [&str; 2] = ["titlescreens", "screenshots"];
/// The size the fixture draws each picture at, a constant of this test.
const DRAWN: (u32, u32) = (512, 448);

#[test]
fn qt_7_8_5_none_stands_for_a_missing_cover_while_a_picture_is_chosen_and_clears_the_choice() {
    let world = sandbox("qt-app-page-cover-none", &[("SNES", &format!("{COBALT}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{COBALT}.sfc"));
    let media = world.dir("EsdeMedia");
    let at = |folder: &str| -> PathBuf { media.join("snes").join(folder).join(format!("{COBALT}.png")) };
    let mut harness = emusen_qt_test::Harness::new();
    let pictures: Vec<_> = FOLDERS.iter().enumerate().map(|(i, folder)| (at(folder), DRAWN, i, String::new())).collect();
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = serde_json::json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let games = || Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    let path = rom.to_str().unwrap();
    // The title screen chosen as the cover, as the base's cover change keeps it: games.db's covertype edit.
    games().change(&serde_json::json!({ "op": "save_edits", "path": path, "changes": { "covertype": "titlescreen" } })).unwrap();
    let chosen = || games().get(&serde_json::json!({ "what": "edits", "path": path })).unwrap()["covertype"].clone();
    assert_eq!(chosen(), "titlescreen");

    let (w, h) = SIZES[0];
    let mut scene = harness.load(MAIN_URL, w, h);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    scene.wait(300);
    press(&mut scene, UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    press(&mut scene, UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);

    // Details ▸ Cover Art: None first, where the cover found would stand, then the game's own pictures, the title screen in use.
    open_page(&mut scene);
    open_cover_art(&mut scene);
    let (shown, in_use) = cards(&mut scene);
    assert_eq!(shown, ["None", "Scraped screenshot", "Title screen"], "None in the found cover's place");
    assert_eq!(in_use.as_deref(), Some("Title screen"));
    move_to(&mut scene, UiButton::Right, "Title screen");
    move_to(&mut scene, UiButton::Left, "None");
    assert_eq!(scene.property("coverInFocusWords", "text"), "None", "the focused card large beside");
    judged_at_every_size(&mut scene);
    if let Ok(grab) = std::env::var("COVER_NONE_GRAB") {
        scene.wait(300);
        assert!(scene.grab(&grab), "the page saved to {grab}");
    }

    // A on None clears the choice in games.db; with nothing chosen the card goes, and nothing is marked in use.
    press(&mut scene, UiButton::Accept);
    until(&mut scene, "the choice cleared", 5000, |_| chosen().is_null());
    until(&mut scene, "the None card gone", 2000, |s| cards(s).0 == ["Scraped screenshot", "Title screen"]);
    assert_eq!(cards(&mut scene).1, None, "no cover found, so no card is in use");
    scene.assert_quiet();
}
