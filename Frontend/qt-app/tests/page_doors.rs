//! A game's page's doors in the program (EmuSen_Qt.md §7.8.2): A on a picture of Media opens the media viewer, Left and
//! Right page through the game's pictures, Up opens its PDF manual and Right turns its page, B closes it; Scrape This
//! Game runs the scraper against the recorded ScreenScraper the platform's scraping test answers from, reaches done and
//! the page's cover is the one it fetched; Edit Details' Name typed on the docked keyboard is stored in games.db and named
//! in the grid.

mod common;

use common::{draw_covers, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_client_fixture::scraper::RecordedScraper;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;

const COBALT: &str = "Cobalt Harbor (Synthetic)";

/// A PDF of `pages` empty letter-sized pages, its cross-reference table counted as written.
fn pdf(pages: usize) -> Vec<u8> {
    let kids: Vec<String> = (0..pages).map(|i| format!("{} 0 R", 3 + i)).collect();
    let mut objects = vec!["<< /Type /Catalog /Pages 2 0 R >>".to_owned(), format!("<< /Type /Pages /Kids [{}] /Count {pages} >>", kids.join(" "))];
    objects.extend((0..pages).map(|_| "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>".to_owned()));
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{object}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    for at in offsets {
        out.extend(format!("{at:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
    out
}

/// `done` within `ms`, else the test fails saying `what`.
fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms");
}

/// What the focused item is heard as.
fn heard(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default()
}

/// The focus moved by `button` until it is on the item heard as `words`.
fn move_to(scene: &mut Scene<'_>, button: UiButton, words: &str) {
    for _ in 0..12 {
        if heard(scene) == words {
            return;
        }
        scene.pad(button);
        scene.wait(80);
    }
    assert_eq!(heard(scene), words, "{button:?} reaches {words}");
}

/// The section by its index, through the right shoulder, each press waited for.
fn section(scene: &mut Scene<'_>, index: i32) {
    for _ in 0..5 {
        let now = scene.property("pageModel", "section");
        if now == index.to_string() {
            break;
        }
        scene.pad(UiButton::PageDown);
        until(scene, "the section steps", 1000, |s| s.property("pageModel", "section") != now);
    }
    assert_eq!(scene.property("pageModel", "section"), index.to_string());
    scene.wait(200);
}

/// Each character typed as the keyboard's key; QTest's key click types the key's small letter.
fn type_words(scene: &mut Scene<'_>, words: &str) {
    for c in words.chars() {
        scene.key(c.to_ascii_uppercase() as i32, 0);
    }
}

#[test]
fn qt_7_8_2_the_page_views_its_media_scrapes_the_game_and_edits_its_name() {
    let world = sandbox("qt-app-page-doors", &[("SNES", &format!("{COBALT}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{COBALT}.sfc"));
    // Two pictures and a manual of three pages in the ES-DE media folder, and no cover: the scrape fetches one.
    let media = world.dir("EsdeMedia");
    let mut harness = emusen_qt_test::Harness::new();
    let pictures: Vec<_> = ["screenshots", "titlescreens"]
        .iter()
        .enumerate()
        .map(|(i, folder)| (media.join("snes").join(folder).join(format!("{COBALT}.png")), (512, 448), i, format!("snes {folder}")))
        .collect();
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    let manual = media.join("snes/manuals").join(format!("{COBALT}.pdf"));
    std::fs::create_dir_all(manual.parent().unwrap()).unwrap();
    std::fs::write(&manual, pdf(3)).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = serde_json::json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let server = RecordedScraper::start();
    RecordedScraper::developer(&world.root.join("Config"));

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    // The media store the scraper fetches into, a folder the theme's covers may come from, as the program's start names it.
    assert!(scene.set_property("program", "coverStores", world.root.join("Home/Media").to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "scraperTest", &server.test));
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
    scene.wait(300);
    assert_eq!(scene.property("pageModel", "cover"), "", "no cover before the scrape");

    // Media: the screenshot, the title screen and the manual; A on the first opens the viewer on it.
    section(&mut scene, 3);
    scene.pad(UiButton::Down);
    until(&mut scene, "the tiles focused", 1000, |s| s.property("pageTiles", "activeFocus") == "true");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "viewing", "true", 1000);
    until(&mut scene, "the viewer holds the focus", 1000, |s| s.property("pageViewer", "activeFocus") == "true");
    let shot = media.join("snes/screenshots").join(format!("{COBALT}.png"));
    assert_eq!(scene.property("pageModel", "viewSource"), format!("file://{}", shot.display()));
    assert_eq!(scene.property("pageModel", "viewCaption"), "Scraped screenshot · 1 of 2");
    until(&mut scene, "the picture drawn", 2000, |s| s.property("viewerPicture", "status") == "Ready");
    assert_eq!(scene.property("pageModel", "viewHelp"), emusen_qt_models::help_model::shown_screen(emusen_qt_models::help::Screen::GamePageViewerManual).to_string());
    // Right to the title screen, Right at the end stays, Left back.
    scene.pad(UiButton::Right);
    until(&mut scene, "Right steps to the title screen", 1000, |s| s.property("pageModel", "viewCaption") == "Title screen · 2 of 2");
    scene.pad(UiButton::Right);
    scene.wait(100);
    assert_eq!(scene.property("pageModel", "viewCaption"), "Title screen · 2 of 2", "the last stays");
    scene.pad(UiButton::Left);
    until(&mut scene, "Left steps back", 1000, |s| s.property("pageModel", "viewCaption") == "Scraped screenshot · 1 of 2");
    // Up opens the manual through the kit's page of Qt PDF, which reads its three pages; Right turns one, Down leaves it.
    scene.pad(UiButton::Up);
    let read = |s: &mut Scene<'_>| s.property("pageModel", "viewCaption") == "Manual · Page 1 of 3";
    until(&mut scene, "the manual opened", 3000, |s| s.property("pageModel", "viewKind") == "2");
    for _ in 0..120 {
        if read(&mut scene) {
            break;
        }
        scene.wait(25);
    }
    assert!(read(&mut scene), "the manual's pages read: caption {:?}, pages {}, source {}", scene.property("pageModel", "viewCaption"), scene.property("viewerManual", "pages"), scene.property("viewerManual", "source"));
    assert_eq!(scene.property("viewerManual", "pages"), "3");
    scene.pad(UiButton::Right);
    until(&mut scene, "Right turns the page", 1000, |s| s.property("pageModel", "viewCaption") == "Manual · Page 2 of 3");
    assert_eq!(scene.property("viewerManual", "page"), "1");
    scene.pad(UiButton::Down);
    until(&mut scene, "Down goes back to the media", 1000, |s| s.property("pageModel", "viewKind") == "0");
    // B closes it and the page has the focus back on the tile it opened from.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pageModel", "viewing", "false", 1000);
    for _ in 0..40 {
        if scene.property("pageTiles", "activeFocus") == "true" {
            break;
        }
        scene.wait(25);
    }
    let at = heard(&mut scene);
    assert_eq!(scene.property("pageTiles", "activeFocus"), "true", "the tiles focused again; the focus on {at:?}, the page frame's {}, the viewer's {}", scene.property("pageFrame", "activeFocus"), scene.property("pageViewer", "activeFocus"));
    assert_eq!(scene.property("screens", "shown"), "Page", "B closed the viewer, not the page");
    // The manual's tile opens the viewer on the manual.
    move_to(&mut scene, UiButton::Right, "Manual");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the manual opened from its tile", 3000, |s| s.property("pageModel", "viewCaption") == "Manual · Page 1 of 3");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pageModel", "viewing", "false", 1000);

    // Details ▸ Scrape This Game: the recorded server is asked, the run reaches done and the page shows the cover fetched.
    scene.pad(UiButton::Up);
    section(&mut scene, 4);
    scene.pad(UiButton::Down);
    move_to(&mut scene, UiButton::Down, "Scrape This Game");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the scrape done", 20000, |s| s.property("pageModel", "scrapeState") == "done");
    assert!(server.asked().iter().any(|p| p.contains("jeuInfos.php")), "the run asked the recorded server: {:?}", server.asked());
    let state = common::state(&mut scene);
    until(&mut scene, "the fetched cover shown", 5000, |s| s.property("pageModel", "cover").starts_with("file://"));
    let cover = scene.property("pageModel", "cover");
    assert!(std::path::Path::new(cover.trim_start_matches("file://")).is_file(), "the cover is the scraper's file: {cover}; {state}");
    assert_eq!(scene.property("pageModel", "problem"), "");

    // Details ▸ Edit Details ▸ Name: the keyboard opens on the name, the words typed are saved at once and named in the grid.
    move_to(&mut scene, UiButton::Down, "Edit Details");
    scene.pad(UiButton::Accept);
    until(&mut scene, "Edit Details' rows", 2000, |s| heard(s) == "Name" || s.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("Release date")));
    move_to(&mut scene, UiButton::Down, "Name");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "typing", "true", 1000);
    until(&mut scene, "the keyboard focused", 1000, |s| s.property("pageKeyboard", "activeFocus") == "true");
    assert_eq!(scene.property("pageTyped", "text"), COBALT, "the keyboard starts from the name");
    for _ in 0..COBALT.len() {
        scene.key(0x0100_0003, 0);
    }
    type_words(&mut scene, "granite choir");
    assert_eq!(scene.property("pageTyped", "text"), "granite choir");
    // X is Done.
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "typing", "false", 1000);
    until(&mut scene, "the page titled by the new name", 3000, |s| s.property("pageModel", "title") == "granite choir");
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    let edits = games.get(&serde_json::json!({ "what": "edits", "path": rom.to_str().unwrap() })).unwrap();
    assert_eq!(edits["name"], "granite choir", "stored in games.db: {edits}");
    // B leaves Edit Details for Details, B again the page for the grid, which names the game by its new name.
    scene.pad(UiButton::Back);
    until(&mut scene, "Details' rows again", 1000, |s| s.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("Scrape This Game")));
    assert_eq!(scene.property("screens", "shown"), "Page");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    until(&mut scene, "the grid names the game by its new name", 10000, |s| s.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("granite choir")));
    scene.assert_quiet();
}
