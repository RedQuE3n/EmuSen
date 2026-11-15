//! Cheats on the tool screen (tools/cheats.rs) over the platform library built from this commit, a scratch library and
//! a loopback server of the test's own serving a database the test made: the List card gets it, the bar's progress
//! stands part way, a stop leaves no database, a second get installs it, and A on the SNES game makes its own list from
//! the game's file, which the game's list then holds. The real address is never fetched.

use emusen_client_fixture::cheats::{served, GAME};
use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_models::cheats::List;
use emusen_qt_models::library::LibrarySource;
use emusen_qt_models::tools::cheats::{self, Cheats};
use emusen_qt_models::tools::{Source, Tool};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A scratch library of one SNES game, named by the settings as the ROM folder.
fn sandbox(name: &str) -> World {
    let world = World::new(name);
    std::fs::write(world.dir("Roms/snes").join(format!("{GAME}.sfc")), vec![0u8; 512 * 1024]).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    world
}

/// The tool polled every 10 ms until `done` holds of it, at most `ms`.
fn until(tool: &mut Tool<Cheats>, what: &str, ms: u64, done: impl Fn(&Tool<Cheats>) -> bool) {
    let end = Instant::now() + Duration::from_millis(ms);
    while !done(tool) && Instant::now() < end {
        tool.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(done(tool), "{what} within {ms} ms: bar {:?}, rows {:?}", tool.bar().line, tool.shown());
}

/// The List card's row of `label` chosen.
fn choose(tool: &mut Tool<Cheats>, label: &str) {
    tool.list();
    let rows = tool.card().expect("the List card").2;
    let at = rows.iter().position(|r| r.label == label).unwrap_or_else(|| panic!("no row {label}: {:?}", rows.iter().map(|r| &r.label).collect::<Vec<_>>()));
    tool.card_choose(at);
}

#[test]
fn the_database_is_got_stopped_got_again_and_a_games_list_made_from_it() {
    let world = sandbox("qt-cheats-get");
    let server = served(&world);
    let source = Arc::new(emusen_qt_models::clients::source_for(library().to_str().unwrap(), "").unwrap());
    let mut tool = Tool::new(Cheats::new(source.clone(), Some(server.first_held.clone())));
    tool.refresh();
    until(&mut tool, "the first read", 10000, |t| !t.source.running());
    assert!(tool.source.facts().iter().any(|f| f.words == "no database"));

    // Get the database: the server holds back half, so the bar's progress stands part way; Stop leaves no database.
    choose(&mut tool, "Get the database");
    until(&mut tool, "the download part way", 10000, |t| t.bar().progress.is_some_and(|p| p > 0.0 && p < 1.0));
    assert!(tool.bar().line.words.starts_with("Downloading: "), "{:?}", tool.bar().line);
    choose(&mut tool, "Stop the download");
    until(&mut tool, "the stop", 10000, |t| !t.source.running());
    assert_eq!(tool.bar().line.words, "Stopped");
    assert!(tool.source.facts().iter().any(|f| f.words == "no database"), "a stop installs nothing: {:?}", tool.source.facts());
    assert!(tool.shown().is_empty());

    // Get it again: whole this time, installed, and the library's SNES game listed with no list of its own.
    choose(&mut tool, "Get the database");
    until(&mut tool, "the install and the read after", 20000, |t| !t.source.running() && !t.shown().is_empty());
    assert_eq!(tool.bar().line.words, "Installed 1 cheat file(s)");
    let row = tool.shown()[0].clone();
    assert_eq!((row.cells[0].words.as_str(), row.cells[1].words.as_str()), (GAME, "No list yet"));
    tool.list();
    assert!(tool.card().unwrap().2.iter().any(|r| r.label == "Update the database"), "with a database the List card updates it");
    tool.card_back();

    // A makes the game's own list from its file, read with the SNES's codecs; the row and the list then count it.
    assert_eq!(tool.action().map(|a| (a.key, a.label)), Some((cheats::MAKE.to_owned(), "Make its list".to_owned())));
    tool.activate(0);
    assert_eq!(tool.shown()[0].cells[1].words, "1 of 2 on");
    assert_eq!(tool.action(), None, "a game with its list has no A");
    let snes = source.systems().unwrap().into_iter().find(|s| s.id == "snes").unwrap();
    let list = List::of(&source.platform, &snes.console, &row.key).unwrap().expect("the game's list");
    let descriptions: Vec<_> = list.switches().into_iter().map(|c| (c.description, c.on)).collect();
    assert_eq!(descriptions, [("Infinite lives".to_owned(), true), ("Moon jump".to_owned(), false)]);
    let write = &list.entries()[0]["Writes"][0];
    assert_eq!((&write["Space"], &write["Address"]), (&json!("WRAM"), &json!("DBE")), "bank 7E is the work RAM's, by the SNES's Pro Action Replay codec");
    drop(server);
    drop(world);
}

#[test]
fn a_list_made_with_cheats_left_out_says_how_many() {
    assert_eq!(cheats::made_words(0), None, "a list that holds every cheat of its file says nothing more");
    let said = cheats::made_words(2).expect("words for a list with two left out");
    assert!(said.words.contains(" 2 "), "{}", said.words);
}

#[test]
fn a_download_ended_with_no_reason_or_no_archive_says_so_plainly() {
    use emusen_platform_client::net::State;
    let failed = cheats::ended(State::Failed, Ok::<_, String>(None)).expect("a failure says why");
    assert_eq!(failed.words, "It could not be downloaded, and no reason was given");
    assert!(cheats::ended(State::Done, Ok::<_, String>(None)).is_none(), "a download done goes on to its archive");
    let empty = cheats::archive(Ok::<_, String>(None)).expect_err("no archive is said");
    assert_eq!(empty.words, "It was downloaded, but there was no archive to install");
}
