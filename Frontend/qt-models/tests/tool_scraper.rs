//! The Scraper on the tool screen (tools/scraper.rs) over the platform library built from this commit, a scratch library
//! and the recorded ScreenScraper: a run over the whole library, paused, resumed and ended, leaves each game a cover; a
//! cancelled run keeps what it fetched; the start request asks for the kinds the Scraping settings switch on; and Home's
//! Scraper place opens it.

use emusen_client_fixture::scraper::RecordedScraper;
use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::library::GameFile;
use emusen_platform_client::Platform;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::LibrarySource;
use emusen_qt_models::screens::{step, Ask, Shown};
use emusen_qt_models::scrape_run::{self, Run};
use emusen_qt_models::tools::scraper::{self, Scraper};
use emusen_qt_models::tools::{Source, Tool};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A scratch library of `count` SNES games, each a file of its own bytes, named by the settings as the ROM folder, with
/// the developer file a run needs.
fn sandbox(name: &str, count: usize) -> World {
    let world = World::new(name);
    let snes = world.dir("Roms/snes");
    for at in 0..count {
        let mut image = vec![0u8; 512 * 1024];
        image[0] = at as u8;
        std::fs::write(snes.join(format!("Game {at}.sfc")), image).unwrap();
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    RecordedScraper::developer(&world.root.join("Config"));
    world
}

fn tool(source: &Arc<ClientSource>, server: &RecordedScraper) -> Tool<Scraper> {
    let mut tool = Tool::new(Scraper::new(source.clone(), serde_json::from_str(&server.test).ok()));
    tool.refresh();
    tool
}

/// The tool polled every 10 ms until `done` holds of it, at most `ms`.
fn until(tool: &mut Tool<Scraper>, what: &str, ms: u64, done: impl Fn(&Tool<Scraper>) -> bool) {
    let end = Instant::now() + Duration::from_millis(ms);
    while !done(tool) && Instant::now() < end {
        tool.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(done(tool), "{what} within {ms} ms: {:?}", tool.source.run());
}

fn run(tool: &Tool<Scraper>) -> Run {
    tool.source.run().cloned().unwrap_or_default()
}

/// The games that have a cover, by the scraper's media.
fn covered(source: &ClientSource) -> Vec<String> {
    let games = source.games().unwrap();
    let systems = source.systems().unwrap();
    source.media(&systems, &games, &["cover"]).unwrap().into_iter().map(|(game, _)| game).collect()
}

#[test]
fn a_run_over_the_library_pauses_resumes_and_leaves_every_game_a_cover() {
    let _world = sandbox("qt-scraper-run", 6);
    let server = RecordedScraper::start();
    let source = Arc::new(emusen_qt_models::clients::source_for(library().to_str().unwrap(), "").unwrap());
    assert!(covered(&source).is_empty(), "no game has a cover before the run");
    let mut tool = tool(&source, &server);
    let rows: Vec<String> = tool.shown().iter().map(|i| i.cells[0].words.clone()).collect();
    assert_eq!(rows[0], "All games", "the whole library first, then the consoles: {rows:?}");
    assert!(rows.len() >= 2, "a row for the console: {rows:?}");
    assert_eq!(tool.action().map(|a| a.label), Some("Start".to_owned()));

    // A on All games starts the run; the bar follows the service's count.
    tool.activate(0);
    assert!(tool.source.running());
    until(&mut tool, "the run counts its games", 10000, |t| run(t).total == 6);
    assert_eq!(tool.action().map(|a| a.key), Some(scraper::PAUSE.to_owned()), "A on the run's row pauses it");
    // A pauses it: the service says paused and the count holds.
    tool.activate(0);
    until(&mut tool, "the run paused", 5000, |t| run(t).paused);
    assert!(tool.bar().line.words.starts_with("Paused: "), "{:?}", tool.bar().line);
    assert_eq!(tool.action().map(|a| a.key), Some(scraper::RESUME.to_owned()), "A on a paused run resumes it");
    let held = run(&tool).done;
    let end = Instant::now() + Duration::from_millis(1500);
    while Instant::now() < end {
        tool.poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(run(&tool).done, held, "nothing ends while paused");
    assert!(held < 6, "paused before its end: {held}");
    // A again resumes it, and it reaches done.
    tool.activate(0);
    until(&mut tool, "the run ended", 60000, |t| !t.source.running());
    let ended = run(&tool);
    assert_eq!((ended.state.as_str(), ended.done, ended.total), ("done", 6, 6), "{ended:?}");
    assert!(server.asked().iter().any(|p| p.contains("jeuInfos.php")), "the recorded server was asked");
    assert_eq!(tool.shown()[0].cells[1].words, "Done");
    assert_eq!(tool.source.page(scraper::ALL, None).len(), 6, "X lists each game's result");
    assert_eq!(covered(&source).len(), 6, "every game has a cover");
    assert!(tool.source.changes_library(), "the run's end has the library read again");
}

#[test]
fn a_cancelled_run_stops_and_keeps_what_it_fetched() {
    let _world = sandbox("qt-scraper-cancel", 12);
    let server = RecordedScraper::start();
    let source = Arc::new(emusen_qt_models::clients::source_for(library().to_str().unwrap(), "").unwrap());
    let mut tool = tool(&source, &server);
    tool.activate(0);
    until(&mut tool, "a game done", 30000, |t| run(t).done >= 1);
    // Select's List card offers Stop the scrape while it runs; it asks first, and Stop cancels the run.
    tool.list();
    let (_, _, rows, _) = tool.card().unwrap();
    let at = rows.iter().position(|r| r.key == scraper::STOP).expect("Stop the scrape on the List card");
    tool.card_choose(at);
    let q = tool.question().cloned().expect("asked first");
    assert_eq!(q.title, "Stop the scrape?");
    tool.answer(1);
    until(&mut tool, "the run stopped", 30000, |t| !t.source.running());
    let ended = run(&tool);
    assert_eq!(ended.state, "cancelled", "{ended:?}");
    assert!(ended.done < 12, "it stopped before its end: {ended:?}");
    assert_eq!(tool.shown()[0].cells[1].words, "Stopped");
    let found = ended.results.iter().filter(|r| r.1.starts_with("Found")).count();
    assert!(found >= 1, "a game was found before it stopped: {ended:?}");
    assert_eq!(covered(&source).len(), found, "each game found keeps its cover, and no other has one");
}

#[test]
fn the_start_request_asks_for_what_the_scraping_settings_switch_on() {
    let world = sandbox("qt-scraper-request", 1);
    let source = emusen_qt_models::clients::source_for(library().to_str().unwrap(), "").unwrap();
    let mut settings = source.settings().unwrap();
    let kinds = source.scraper().unwrap().get(&json!({ "what": "kinds" })).unwrap();
    let wanted = scrape_run::wanted_kinds(&settings, &kinds);
    assert_eq!(wanted, ["cover", "screenshot", "marquee", "titlescreen", "miximage", "backcover", "3dbox", "physicalmedia", "fanart"], "the settings' own switches");
    settings["ScrapeCovers"] = json!(false);
    settings["ScrapeVideos"] = json!(true);
    settings["ScrapeThreads"] = json!(2);
    settings["Scraping"] = json!(false);
    let game = GameFile { path: world.root.join("Roms/snes/Game 0.sfc").to_string_lossy().into_owned(), system: Some("snes".into()), ..GameFile::default() };
    let r = scrape_run::request(&[game.clone()], false, &settings, &kinds);
    let asked: Vec<&str> = r["choices"]["kinds"].as_array().unwrap().iter().filter_map(|k| k.as_str()).collect();
    assert!(!asked.contains(&"cover") && asked.contains(&"video"), "{asked:?}");
    assert_eq!((r["one"].clone(), r["enabled"].clone(), r["choices"]["threads"].clone()), (json!(false), json!(false), json!(2)));
    assert_eq!(r["games"], json!([{ "path": game.path, "system": "snes" }]));
}

#[test]
fn home_s_scraper_place_and_the_tools_row_open_the_scraper() {
    assert_eq!(step(Shown::Home, &Ask::Place(1)), Some((Shown::Tool, Some("scraper".into()))));
    assert_eq!(emusen_qt_models::screens::tool_screen("scraper"), Some(Shown::Tool));
}
