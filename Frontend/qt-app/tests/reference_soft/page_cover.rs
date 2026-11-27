//! A game's own picture as its cover, in the program (EmuSen_Qt.md §7.8.5; EmuSen_Backend.md §30.6): Details ▸ Cover Art
//! shows the game's cover and its own pictures as cards, the one in use marked; A on the title screen's card makes it the
//! cover, which the page, the card's mark and the grid's tile then show, and A on Its own puts the cover found back; at
//! each of the gate's window sizes nothing is cut off or overlapping and the pad reaches every card. The media viewer's
//! X, Set as cover, makes the picture shown the cover. The choice is games.db's `covertype` edit; no file is written.

use crate::common;

use common::cover_art::{cards, judged_at_every_size, move_to, open_cover_art, open_page, press, reads, section, until};
use common::{draw_covers, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::help::Screen;
use emusen_qt_models::help_model::shown_screen;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::Scene;
use std::path::{Path, PathBuf};

const COBALT: &str = "Cobalt Harbor (Synthetic)";
/// The pictures drawn, each by its folder, in the order `draw_covers` colours them.
const FOLDERS: [&str; 3] = ["covers", "titlescreens", "screenshots"];
/// The size the fixture draws each picture at, a constant of this test.
const DRAWN: (u32, u32) = (512, 448);

/// `draw_covers`' fill for the picture at `index`.
fn fill(index: usize) -> Colour {
    let (r, g, b) = emusen_qt_test::covers::fill(index);
    Colour { r, g, b, a: 255 }
}

fn near(a: Colour, b: Colour) -> bool {
    [(a.r, b.r), (a.g, b.g), (a.b, b.b)].iter().all(|(x, y)| x.abs_diff(*y) <= 6)
}

/// The grid's first cell's picture as drawn: its colour at the middle of the picture, which stands whole at the foot of its
/// box (CoverTile.qml's picture fits and aligns to the bottom), clear of the rim.
fn grid_colour(scene: &mut Scene<'_>) -> Option<Colour> {
    let tree = scene.tree();
    let cell = tree.nodes.iter().find(|n| n.visible && n.name == "gridCell.0")?.id.clone();
    let inside = |n: &emusen_qt_test::tree::Node| {
        let mut at = Some(n);
        while let Some(p) = at {
            if p.id == cell {
                return true;
            }
            at = tree.nodes.iter().find(|q| q.id == p.parent);
        }
        false
    };
    let p = tree.nodes.iter().find(|n| n.visible && n.name == "coverPicture" && inside(n))?;
    let (dw, dh) = (f64::from(DRAWN.0), f64::from(DRAWN.1));
    let painted = dh * (p.w / dw).min(p.h / dh);
    let point = (p.x + p.w / 2.0, p.y + p.h - painted / 2.0);
    scene.colours_at(&[point]).into_iter().next().flatten()
}

fn grid_shows(scene: &mut Scene<'_>, index: usize, what: &str) {
    until(scene, what, 5000, |s| grid_colour(s).is_some_and(|c| near(c, fill(index))));
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}


#[test]
fn qt_7_8_2_cover_art_and_the_viewer_make_a_games_own_picture_its_cover_in_the_page_and_the_grid() {
    let world = sandbox("qt-app-page-cover", &[("SNES", &format!("{COBALT}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{COBALT}.sfc"));
    let media = world.dir("EsdeMedia");
    let at = |folder: &str| -> PathBuf { media.join("snes").join(folder).join(format!("{COBALT}.png")) };
    let mut harness = emusen_qt_test::Harness::new();
    let pictures: Vec<_> = FOLDERS.iter().enumerate().map(|(i, folder)| (at(folder), DRAWN, i, String::new())).collect();
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    let drawn: Vec<Vec<u8>> = FOLDERS.iter().map(|f| std::fs::read(at(f)).unwrap()).collect();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = serde_json::json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let chosen = || {
        let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
        games.get(&serde_json::json!({ "what": "edits", "path": rom.to_str().unwrap() })).unwrap()["covertype"].clone()
    };

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
    grid_shows(&mut scene, 0, "the grid draws the game's cover");

    // Details ▸ Cover art: the cover found in use, then the game's own pictures; A on the title screen's card makes it the cover.
    open_page(&mut scene);
    assert_eq!(scene.property("pageModel", "cover"), file_url(&at("covers")));
    open_cover_art(&mut scene);
    let (shown, in_use) = cards(&mut scene);
    assert_eq!(shown.len(), FOLDERS.len(), "a card for the cover found and each picture: {shown:?}");
    assert!(shown.iter().any(|w| w == "Scraped screenshot") && shown.iter().any(|w| w == "Title screen"));
    assert_eq!(in_use.as_deref(), Some("Its own"), "the cover found marked in use");
    assert_eq!(scene.property("coverArtTitle", "text"), "Cover Art");
    judged_at_every_size(&mut scene);
    move_to(&mut scene, UiButton::Right, "Title screen");
    assert_eq!(scene.property("coverInFocusWords", "text"), "Title screen", "the focused card large beside");
    press(&mut scene, UiButton::Accept);
    until(&mut scene, "the page's cover is the title screen", 5000, |s| s.property("pageModel", "cover") == file_url(&at("titlescreens")));
    assert_eq!(chosen(), "titlescreen", "stored in games.db");
    until(&mut scene, "the title screen's card marked in use", 2000, |s| cards(s).1.as_deref() == Some("Title screen"));
    assert_eq!(scene.property("pageModel", "problem"), "");
    // B back to Details, whose row names the choice; B again to the grid, whose tile draws the title screen.
    press(&mut scene, UiButton::Back);
    until(&mut scene, "Details' rows again", 1000, |s| reads(s, "Open Folder"));
    assert_eq!(scene.property("screens", "shown"), "Page");
    press(&mut scene, UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    grid_shows(&mut scene, 1, "the grid's tile draws the title screen");

    // A on Its own puts the cover found back, in the page and the grid.
    open_page(&mut scene);
    open_cover_art(&mut scene);
    move_to(&mut scene, UiButton::Left, "Its own");
    press(&mut scene, UiButton::Accept);
    until(&mut scene, "the page's cover found again", 5000, |s| s.property("pageModel", "cover") == file_url(&at("covers")));
    assert_eq!(chosen(), serde_json::Value::Null, "the choice gone from games.db");
    until(&mut scene, "the cover found's card marked in use", 2000, |s| cards(s).1.as_deref() == Some("Its own"));

    press(&mut scene, UiButton::Back);
    until(&mut scene, "Details' rows again", 1000, |s| reads(s, "Open Folder"));
    press(&mut scene, UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    grid_shows(&mut scene, 0, "the grid's tile draws the cover found again");

    // Media: the viewer on the scraped screenshot offers Set as cover; X makes it the cover, and the offer goes.
    open_page(&mut scene);
    section(&mut scene, 3);
    press(&mut scene, UiButton::Down);
    until(&mut scene, "the tiles focused", 1000, |s| s.property("pageTiles", "activeFocus") == "true");
    move_to(&mut scene, UiButton::Right, "Scraped screenshot");
    press(&mut scene, UiButton::Accept);
    wait_for(&mut scene, "pageModel", "viewing", "true", 1000);
    until(&mut scene, "the viewer holds the focus", 1000, |s| s.property("pageViewer", "activeFocus") == "true");
    assert_eq!(scene.property("pageModel", "viewSource"), file_url(&at("screenshots")));
    assert_eq!(scene.property("pageModel", "viewHelp"), shown_screen(Screen::GamePageViewerCover).to_string());
    assert!(reads(&mut scene, "Set as cover"), "the viewer's help bar offers it");
    press(&mut scene, UiButton::Screensaver);
    until(&mut scene, "the page's cover is the screenshot", 5000, |s| s.property("pageModel", "cover") == file_url(&at("screenshots")));
    assert_eq!(chosen(), "screenshot");
    until(&mut scene, "the offer gone for the cover shown", 1000, |s| s.property("pageModel", "viewHelp") == shown_screen(Screen::GamePageViewer).to_string());
    assert_eq!(scene.property("pageModel", "viewing"), "true", "X left the viewer open");
    press(&mut scene, UiButton::Back);
    wait_for(&mut scene, "pageModel", "viewing", "false", 1000);
    press(&mut scene, UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    grid_shows(&mut scene, 2, "the grid's tile draws the screenshot");

    let after: Vec<Vec<u8>> = FOLDERS.iter().map(|f| std::fs::read(at(f)).unwrap()).collect();
    assert_eq!(after, drawn, "no picture file was written");
    scene.assert_quiet();
}
