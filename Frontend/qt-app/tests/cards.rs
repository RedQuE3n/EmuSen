//! The Home card and the List card through the program on the software backend (EmuSen_Qt.md §7.1, stage 2): Home's
//! Power place opens the Home card at Power, Quit EmuSen... asks first, B climbs a page at a time; Start goes Home from
//! the grid and takes Home's focus back to where it starts; Select opens the grid's List card, whose Sort, Filter,
//! Jump to and Search change what the grid shows. Each card is held at the gate's five sizes and recorded beside
//! Mistress's recordings (HomeCardPower, HomeCardQuit, GamesListCard, GamesSearch).

mod common;

use common::{draw_covers, hold, hold_walking, sandbox, wait_for};
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::WITH_SHOULDERS;
use emusen_client_fixture::library;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::excuse::Excuses;
use emusen_qt_test::{mockup, Scene};

/// Qt's key codes for the letters typed on a keyboard.
const KEY_G: i32 = 0x47;
const KEY_R: i32 = 0x52;
const KEY_A: i32 = 0x41;

/// The fixture's games, as Mistress's own-look library names them.
const GAMES: [(&str, &str); 8] = [
    ("SNES", "Aurora Drift (Synthetic).sfc"),
    ("SNES", "Brass Lantern (Synthetic).sfc"),
    ("SNES", "Cobalt Harbor (Synthetic).sfc"),
    ("SNES", "Dune Relay (Synthetic).sfc"),
    ("SNES", "Ember Circuit (Synthetic).sfc"),
    ("NES", "Fable of Tiles (Synthetic).nes"),
    ("NES", "Granite Choir (Synthetic).nes"),
    ("GB", "Hollow Comet (Synthetic).gb"),
];

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// The shown card's rows: each row's label and value, in order.
fn rows(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let words = |parent: &str, name: &str| tree.nodes.iter().find(|n| n.parent == parent && n.name == name).and_then(|n| n.text.clone()).unwrap_or_default();
    let mut rows: Vec<(f64, String)> = tree
        .nodes
        .iter()
        .filter(|n| n.visible && n.ty.starts_with("MenuRow"))
        .map(|n| (n.y, [words(&n.id, "label"), words(&n.id, "value")].join(" | ").trim_end_matches(" | ").to_owned()))
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    rows.into_iter().map(|(_, r)| r).collect()
}

fn titled(scene: &mut Scene<'_>, card: &str) -> (String, String) {
    let tree = scene.tree();
    let under = |name: &str| {
        let root = tree.nodes.iter().find(|n| n.name == card).map(|n| n.id.clone()).unwrap_or_default();
        let mut found = String::new();
        for n in &tree.nodes {
            let mut at = Some(n);
            while let Some(p) = at {
                if p.id == root && n.name == name && n.visible {
                    found = n.text.clone().unwrap_or_default();
                }
                at = tree.nodes.iter().find(|q| q.id == p.parent);
            }
        }
        found
    };
    (under("cardBreadcrumb"), under("cardTitle"))
}

/// A grid cell's cover as drawn: the name its picture is heard by (the game's title once drawn from its file, none for a
/// generated card), its box, and the colours of a grid of points inside it, 12 pixels in from each side.
fn cover_of(scene: &mut Scene<'_>, index: usize) -> (String, f64, f64, Vec<Option<emusen_kit_core::tokens::Colour>>) {
    let tree = scene.tree();
    let cell = tree.nodes.iter().find(|n| n.visible && n.name == format!("gridCell.{index}")).map(|n| n.id.clone()).unwrap_or_default();
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
    let picture = tree.nodes.iter().find(|n| n.visible && n.name == "coverPicture" && inside(n));
    let Some(p) = picture else { return (String::new(), 0.0, 0.0, Vec::new()) };
    let (heard, x, y, w, h) = (p.heard.clone(), p.x, p.y, p.w, p.h);
    let points: Vec<(f64, f64)> = (0..5).flat_map(|i| (0..5).map(move |j| (x + 12.0 + (w - 24.0) * f64::from(i) / 4.0, y + 12.0 + (h - 24.0) * f64::from(j) / 4.0))).collect();
    (heard, w, h, scene.colours_at(&points))
}

fn settle(scene: &mut Scene<'_>) {
    scene.wait(450);
}

#[test]
fn the_home_card_and_the_list_card_are_mistresss() {
    let world = sandbox("qt-app-cards", &GAMES);
    let mut harness = emusen_qt_test::Harness::new();
    // Each game's cover in an ES-DE media folder, as Mistress's own-look fixture draws them, so the grid shows pictures.
    let media = world.dir("Media");
    let covers: Vec<_> = GAMES
        .iter()
        .enumerate()
        .map(|(i, (folder, file))| {
            let (system, w, h) = match *folder { "SNES" => ("snes", 560, 400), "NES" => ("nes", 420, 600), _ => ("gb", 540, 600) };
            let title = file.rsplit_once('.').unwrap().0;
            (media.join(system).join("covers").join(format!("{title}.png")), (w, h), i, format!("{system} covers\n{}\n{w}x{h}", title.trim_end_matches(" (Synthetic)")))
        })
        .collect();
    draw_covers(&mut harness, &world.root, &covers, |_| {});
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    // The folder the theme's covers may come from, as the program's start reads it from the settings.
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    let mut report = Vec::new();
    let mut excuses = Excuses::none();

    // A hidden card's words wait for it to show (EmuSen_Qt.md §7.8): its facts changed under Home (a paused game's minute
    // turning over did it in acceptance) swap no frame, and the card shown reads them current. Then the facts as the
    // window gives them, and the card closed.
    // The clock held: its time is the held instant's, not the wall clock's (acceptance holds it so).
    assert!(scene.set_property("clock", "now", common::HELD_NOW));
    assert_eq!(scene.property("clock", "time"), "12:00", "the clock reads the held instant");
    let given = scene.property("homeCardModel", "facts");
    scene.assert_still("Home");
    let before = scene.frames();
    let mut facts: serde_json::Value = serde_json::from_str(&given).unwrap();
    // A suspended game paused a minute ago, which the hidden card's first page draws as its Continue row and caption,
    // and Steam's Game Mode, which Power's rows show once it opens.
    facts["suspended"] = serde_json::json!("Echo Forge (Synthetic)");
    facts["paused"] = serde_json::json!("Super Nintendo · Paused 1 minute ago");
    facts["gameMode"] = serde_json::json!(true);
    assert!(scene.set_property("homeCardModel", "facts", &facts.to_string()));
    scene.wait(500);
    assert_eq!(scene.frames() - before, 0, "the hidden Home card's new facts swapped a frame");
    for _ in 0..6 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    settle(&mut scene);
    assert_eq!(rows(&mut scene), ["Quit EmuSen..."], "shown, the card reads the facts it was given while hidden");
    scene.pad(UiButton::Back);
    settle(&mut scene);
    assert!(scene.set_property("homeCardModel", "facts", &given));
    scene.pad(UiButton::Menu);
    scene.wait(50);

    // Home's Power place opens the Home card at Power, with Home as the way back (HomeCardPower).
    for _ in 0..6 {
        scene.pad(UiButton::Right);
    }
    assert_eq!(focused(&mut scene), "place.6");
    scene.pad(UiButton::Accept);
    settle(&mut scene);
    assert_eq!(scene.property("screens", "shown"), "HomeCard");
    assert_eq!(titled(&mut scene, "homeCardCanvas"), ("Home".into(), "Power".into()));
    assert_eq!(rows(&mut scene), ["Exit Big Picture", "Quit EmuSen..."]);
    assert_eq!(scene.property("homeCardModel", "current"), "0");
    scene.assert_still("the Home card in place");
    hold(&mut scene, "HomeCardPower", &mockup::Mockup::default(), &mut excuses, &mut report);

    // Quit EmuSen... asks first: No, then Yes, the sentence above the question (HomeCardQuit); B goes back to its row.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(50);
    assert_eq!(titled(&mut scene, "homeCardCanvas"), ("EmuSen closes.".into(), "Quit EmuSen?".into()));
    assert_eq!(rows(&mut scene), ["No", "Yes"]);
    hold(&mut scene, "HomeCardQuit", &mockup::Mockup::default(), &mut excuses, &mut report);
    scene.pad(UiButton::Back);
    scene.wait(50);
    assert_eq!(rows(&mut scene), ["Exit Big Picture", "Quit EmuSen..."]);
    assert_eq!(scene.property("homeCardModel", "current"), "1", "back on Quit EmuSen...");
    // No goes back too; B on Power closes the card, and Home has the focus again.
    scene.pad(UiButton::Accept);
    scene.pad(UiButton::Accept);
    scene.wait(50);
    assert_eq!(rows(&mut scene), ["Exit Big Picture", "Quit EmuSen..."]);
    scene.pad(UiButton::Back);
    settle(&mut scene);
    assert_eq!(scene.property("screens", "shown"), "Home");
    wait_for(&mut scene, "home", "activeFocus", "true", 1000);
    // Start on Home takes the focus back to where Home starts.
    scene.pad(UiButton::Menu);
    scene.wait(50);
    assert_eq!(focused(&mut scene), "place.0", "Start puts the focus back on Games");

    // Games, All Games (focused by name: the first focus is the first console, EmuSen_Qt.md §3.4.2), the grid; Start goes Home.
    all_games(&mut scene);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "gridCells", "count", "8", 3000);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "Home", 1000);
    all_games(&mut scene);
    wait_for(&mut scene, "gridCells", "count", "8", 3000);
    // Granite Choir as the full grid draws it, seventh by name: its picture, drawn, at its painted size.
    scene.wait(300);
    let granite = cover_of(&mut scene, 6);
    assert_eq!(granite.0, "Granite Choir (Synthetic)", "the full grid draws Granite Choir's cover from its file");

    // Select opens the List card over the grid (GamesListCard): the section and its count, Mistress's rows.
    scene.pad(UiButton::Options);
    settle(&mut scene);
    assert_eq!(scene.property("screens", "shown"), "List");
    assert_eq!(titled(&mut scene, "listCardCanvas"), ("List".into(), "All · 8 games".into()));
    assert_eq!(rows(&mut scene), ["Search", "Sort | Name", "Filter", "Jump to | A", "Random game", "Collections"]);
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("Type part of a name; the covers narrow as you type.")), "the pane says what Search does");
    hold(&mut scene, "GamesListCard", &mockup::Mockup::default(), &mut excuses, &mut report);

    // Sort steps on Right and on A, the grid re-sorted beneath; Left steps back.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert_eq!(rows(&mut scene)[1], "Sort | Last played");
    scene.pad(UiButton::Left);
    scene.wait(200);
    assert_eq!(rows(&mut scene)[1], "Sort | Name");
    // Jump to steps the letter, moving the grid's focus to its first game.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert_eq!(rows(&mut scene)[3], "Jump to | B");
    assert_eq!(scene.property("gridCells", "currentIndex"), "1");

    // Filter ▸ Favourites narrows the section: none are favourites here, so "0 of 8 games".
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Accept);
    scene.wait(50);
    assert_eq!(titled(&mut scene, "listCardCanvas").1, "Filter");
    assert_eq!(rows(&mut scene), ["Every game", "Favourites"]);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    settle(&mut scene);
    assert_eq!(scene.property("screens", "shown"), "Grid", "a filter chosen closes the card");
    wait_for(&mut scene, "gridGames", "countLine", "0 of 8 games", 2000);
    // B clears the filter first, then goes back to Choose a console.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "gridGames", "countLine", "8 games", 2000);
    assert_eq!(scene.property("screens", "shown"), "Grid");

    // Search ▸ docks the keyboard with the focus on its keys; letters typed on a keyboard and a key clicked narrow the
    // grid, and the count says so (GamesSearch).
    scene.pad(UiButton::Options);
    settle(&mut scene);
    scene.pad(UiButton::Accept);
    settle(&mut scene);
    assert_eq!(scene.property("screens", "shown"), "Search");
    assert_eq!(scene.property("GalleryKeyboard", "activeFocus"), "true", "the keys hold the focus");
    for key in [KEY_G, KEY_R, KEY_A] {
        scene.key(key, 0);
    }
    let tree = scene.tree();
    let n = &tree.nodes[tree.find_named("key.36").expect("the n key") as usize];
    let (x, y) = (n.x + n.w / 2.0, n.y + n.h / 2.0);
    scene.click(x, y);
    assert_eq!(scene.property("GallerySearch", "text"), "gran", "the field shows the words");
    wait_for(&mut scene, "gridGames", "countLine", "1 of 8 games", 2000);
    assert_eq!(scene.property("gridCells", "count"), "1");
    // Mistress's field is never focused, so it draws its Border edge and no ring while the keys hold the focus.
    assert_eq!(scene.property("GallerySearch", "activeFocus"), "false");
    assert_eq!(scene.property("GalleryKeyboard", "showsCurrent"), "true", "the current key ringed");
    // The narrowed grid draws the tile the full grid draws: the same picture, drawn, at the same size.
    scene.wait(300);
    assert_eq!(cover_of(&mut scene, 0), granite, "Granite Choir's tile in the search: its picture, box and colours as in the full grid");
    // The pointer set aside on the sky, so that at no size does it rest on a cover, which would take the focus there.
    scene.hover(2.0, 2.0);
    // The gate's walk presses the shoulders too, which take the focus between the keys and the covers (EmuSen_Kit.md §7.103).
    hold_walking(&mut scene, "GamesSearch", &mockup::Mockup::default(), &mut excuses, &mut report, Variant::Dark, &WITH_SHOULDERS);
    // The shoulders move the focus to the covers, the keyboard staying docked without its ring, and back to the keys.
    scene.resize(1280, 800);
    scene.wait(150);
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "SearchGrid", 1000);
    assert_eq!(focused(&mut scene), "gridCell.0", "the found game focused");
    assert_eq!(scene.property("GalleryKeyboard", "showsCurrent"), "false", "no ring on the keys once the focus has left them");
    assert_eq!(scene.property("GalleryKeyboard", "visible"), "true");
    hold_walking(&mut scene, "GamesSearchGrid", &mockup::Mockup::default(), &mut excuses, &mut report, Variant::Dark, &WITH_SHOULDERS);
    scene.resize(1280, 800);
    scene.pad(UiButton::PageUp);
    wait_for(&mut scene, "screens", "shown", "Search", 1000);
    assert_eq!(scene.property("GalleryKeyboard", "activeFocus"), "true");
    // The pointer resting on a cover takes the focus there, as the shoulders do; a click on the band takes it back.
    let tree = scene.tree();
    let cell = &tree.nodes[tree.find_named("gridCell.0").expect("the cover") as usize];
    let (cx, cy) = (cell.x + cell.w / 2.0, cell.y + cell.h / 2.0);
    scene.hover(cx, cy);
    wait_for(&mut scene, "screens", "shown", "SearchGrid", 1000);
    scene.click(x, y + 80.0);
    wait_for(&mut scene, "screens", "shown", "Search", 1000);
    assert_eq!(scene.property("GalleryKeyboard", "activeFocus"), "true");
    // B erases a letter at a time; on the empty field it closes the keyboard, and the screen does not go back with it.
    for _ in 0..4 {
        scene.pad(UiButton::Back);
    }
    assert_eq!(scene.property("GallerySearch", "text"), "");
    wait_for(&mut scene, "gridGames", "countLine", "8 games", 2000);
    assert_eq!(scene.property("screens", "shown"), "Search");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "gridCells", "activeFocus", "true", 1000);
    // X is Done: the keyboard closes and the narrowed grid stays; B clears the search, then goes to Choose a console.
    scene.pad(UiButton::Options);
    settle(&mut scene);
    scene.pad(UiButton::Accept);
    settle(&mut scene);
    scene.key(KEY_G, 0);
    wait_for(&mut scene, "gridGames", "countLine", "1 of 8 games", 2000);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    assert_eq!(scene.property("gridGames", "countLine"), "1 of 8 games", "Done keeps the search");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "gridGames", "countLine", "8 games", 2000);
    assert_eq!(scene.property("screens", "shown"), "Grid");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    assert!(report.is_empty(), "{} faults:\n{}", report.len(), report.join("\n"));
    scene.assert_quiet();
}

/// From Home's Games, Choose a console's All Games: the first place, `tile.0`, opened with A.
fn all_games(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let place = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, place));
    scene.pad(UiButton::Accept);
}

