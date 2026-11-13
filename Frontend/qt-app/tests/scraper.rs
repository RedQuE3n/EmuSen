//! The Scraper in the program, over the platform library built from this commit, a scratch library and the recorded
//! ScreenScraper the crates' scraping tests answer from: the pad opens it from Home's place; a run over one console is
//! stopped from Select's List card and the games done before keep their covers; a run over the whole library moves, A
//! pauses it and the count holds, A resumes it, it reaches done, and every game, none of which had a picture, has its
//! cover in the grid.

mod common;

use common::wait_for;
use emusen_client_fixture::World;
use emusen_client_fixture::library;
use emusen_client_fixture::scraper::RecordedScraper;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;

/// `count` games of their own bytes in each console's folder, the ROM folder named by the settings, OpenEmu's sources off.
fn sandbox(consoles: &[(&str, &str, usize)]) -> World {
    let world = World::new("qt-app-scraper");
    let mut at = 0u8;
    for (folder, ext, count) in consoles {
        let roms = world.dir(&format!("Roms/{folder}"));
        for n in 0..*count {
            let mut image = vec![0u8; 512 * 1024];
            image[0] = at;
            at += 1;
            std::fs::write(roms.join(format!("Game {n}.{ext}")), image).unwrap();
        }
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    settings["OpenEmuFallback"] = serde_json::json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    RecordedScraper::developer(&world.root.join("Config"));
    world
}

/// The covers in the media store, counted on disk.
fn covers(world: &World) -> usize {
    fn walk(dir: &std::path::Path, n: &mut usize) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, n);
            } else if path.parent().and_then(|p| p.file_name()).is_some_and(|f| f == "covers") {
                *n += 1;
            }
        }
    }
    let mut n = 0;
    walk(&world.root.join("Home/Media"), &mut n);
    n
}

/// The focused item's words, or those of the first item under it that has some.
fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// `done` within `ms`, else the test fails saying `what`.
fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms: summary {:?}", scene.property("jobsModel", "summary"));
}

/// The row whose words start with `row` focused, by Up or Down.
fn row(scene: &mut Scene<'_>, button: UiButton, row: &str) {
    for _ in 0..8 {
        if focused_words(scene).starts_with(row) {
            return;
        }
        scene.pad(button);
        scene.wait(100);
    }
    assert!(focused_words(scene).starts_with(row), "no row reads {row}: {}", focused_words(scene));
}

#[test]
fn the_scraper_runs_over_the_library_from_the_pad_pauses_resumes_and_stops() {
    let world = sandbox(&[("snes", "sfc", 8), ("nes", "nes", 2)]);
    let server = RecordedScraper::start();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    // The media store the scraper fetches into, a folder the covers may come from, as the program's start names it.
    assert!(scene.set_property("program", "coverStores", world.root.join("Home/Media").to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "scraperTest", &server.test));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(covers(&world), 0, "no game has a cover before");

    // The Scraper, Home's second place.
    scene.pad(UiButton::Right);
    scene.wait(100);
    assert_eq!(focused(&mut scene), "place.1");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Tool", 2000);
    wait_for(&mut scene, "jobsModel", "title", "Scraper", 3000);
    scene.wait(200);
    assert!(focused_words(&mut scene).starts_with("All games"), "the whole library first: {}", focused_words(&mut scene));
    assert_eq!(scene.property("jobsModel", "summary"), "No job has run");

    // A run over the SNES alone, stopped from Select's List card after its first game: what it fetched is kept.
    row(&mut scene, UiButton::Down, "Super Nintendo");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "busy", "true", 2000);
    until(&mut scene, "the first SNES game done", 30000, |s| s.property("jobsModel", "summary").starts_with("Scraping: ") && !s.property("jobsModel", "summary").starts_with("Scraping: 0 "));
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 1000);
    scene.wait(300);
    let rows: serde_json::Value = serde_json::from_str(&scene.property("jobsModel", "cardRows")).unwrap();
    let at = rows.as_array().unwrap().iter().position(|r| r["label"] == "Stop the scrape").expect("Stop the scrape on the List card");
    for _ in 0..at {
        scene.pad(UiButton::Down);
    }
    scene.wait(100);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "questionTitle", "Stop the scrape?", 1000);
    scene.wait(300);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "busy", "false", 30000);
    let stopped = scene.property("jobsModel", "summary");
    assert!(stopped.starts_with("Stopped: "), "the bar says it stopped: {stopped}");
    let done: usize = stopped["Stopped: ".len()..].split(' ').next().unwrap().parse().unwrap();
    assert!((1..8).contains(&done), "stopped before its end: {stopped}");
    let kept = covers(&world);
    assert!((1..8).contains(&kept), "the games done keep their covers: {kept}");

    // A run over the whole library: it moves, A pauses it and the count holds, A resumes it, and it reaches done.
    row(&mut scene, UiButton::Up, "All games");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "busy", "true", 2000);
    until(&mut scene, "the run counts the library", 30000, |s| s.property("jobsModel", "summary").ends_with(" of 10"));
    let mut moved = false;
    for _ in 0..1200 {
        let progress: f64 = scene.property("jobsModel", "progress").parse().unwrap();
        moved = progress > 0.0 && progress < 1.0;
        if moved {
            break;
        }
        scene.wait(25);
    }
    assert!(moved, "the bar's share moved: {}", scene.property("jobsModel", "summary"));
    scene.pad(UiButton::Accept);
    until(&mut scene, "the run paused", 5000, |s| s.property("jobsModel", "summary").starts_with("Paused: "));
    let held = scene.property("jobsModel", "summary");
    scene.wait(1500);
    assert_eq!(scene.property("jobsModel", "summary"), held, "nothing ends while paused");
    assert_eq!(scene.property("jobsModel", "busy"), "true");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the run resumed", 5000, |s| s.property("jobsModel", "summary").starts_with("Scraping: ") || s.property("jobsModel", "busy") == "false");
    wait_for(&mut scene, "jobsModel", "busy", "false", 60000);
    assert_eq!(scene.property("jobsModel", "summary"), "Done: 10 of 10");
    assert!(server.asked().iter().any(|p| p.contains("jeuInfos.php")), "the recorded server was asked");
    assert_eq!(covers(&world), 10, "every game has its cover");

    // B to Home, then the grid of All games: every game drawn from the cover fetched, none as a generated card.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    for _ in 0..20 {
        if focused(&mut scene) == "place.1" {
            break;
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Left);
    scene.wait(100);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    until(&mut scene, "every cover in the grid", 10000, |s| s.tree().nodes.iter().filter(|n| n.visible && n.name == "coverPicture").count() == 10);
    scene.assert_quiet();
}
