//! Cover art's cards as the game page's source reads them over the platform library built from this commit, in a scratch
//! tree (EmuSen_Qt.md §7.8.5): no card for a cover that was not found, a cover borrowed from another game read from the
//! base's `from` as its own card in use (VenusRT installed, since the base follows a borrowed cover only to a game it
//! routes to a console), and a kind the Media section does not show named by the Scraping settings' row.

use emusen_client_fixture::cores::installation_with_snes;
use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::{GameFile, Library};
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::game_page::{CoverChoice, GamePageSource};
use serde_json::json;
use std::path::PathBuf;

/// A scratch tree with an ES-DE media folder holding `pictures` (folder, game title) and two SNES games, with VenusRT
/// installed where `snes` asks, so the base can route a game to its console: the source and the games' files.
fn world(name: &str, pictures: &[(&str, &str)], snes: bool) -> (World, ClientSource, GameFile, GameFile, PathBuf) {
    let world = World::new(name);
    let roms = world.dir("Roms/snes");
    let media = world.dir("EsdeMedia");
    for (folder, title) in pictures {
        let at = media.join("snes").join(folder);
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join(format!("{title}.png")), folder).unwrap();
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let game = |title: &str| {
        let path = roms.join(format!("{title}.sfc"));
        std::fs::write(&path, vec![0u8; 512 * 1024]).unwrap();
        GameFile { path: path.to_string_lossy().into_owned(), system: Some("snes".into()), ..GameFile::default() }
    };
    let (mine, other) = (game("Mine"), game("Other"));
    let cores = if snes { installation_with_snes(&world.dir("lib")).to_string_lossy().into_owned() } else { String::new() };
    let source = emusen_qt_models::clients::source_for(library().to_str().unwrap(), &cores).unwrap();
    (world, source, mine, other, media)
}

fn cards(source: &ClientSource, game: &GameFile) -> (Vec<(CoverChoice, String, String)>, CoverChoice) {
    let read = source.read(game).unwrap();
    assert_eq!(read.covers_unread, None, "the scraper opened");
    (read.covers.into_iter().map(|c| (c.choice, c.text, c.picture)).collect(), read.cover_in_use)
}

#[test]
fn qt_7_8_5_a_game_with_no_cover_found_has_no_card_for_one() {
    let (_world, source, mine, other, media) = world("models-cover-none", &[("titlescreens", "Mine")], false);
    let title = media.join("snes/titlescreens/Mine.png").to_string_lossy().into_owned();
    let (shown, in_use) = cards(&source, &mine);
    assert_eq!(shown, [(CoverChoice::Own("titlescreen".into()), "Title screen".into(), title)], "its title screen alone, no card for the cover");
    assert_eq!(in_use, CoverChoice::Found, "nothing chosen");
    let read = source.read(&other).unwrap();
    assert!(read.covers.is_empty(), "a game with nothing to choose has no cards: {:?}", read.covers);
    assert_eq!(read.covers_unread, None, "and nothing failed");
}

#[test]
fn qt_7_8_5_a_cover_borrowed_from_another_game_is_read_as_its_own_card_in_use() {
    let (world, source, mine, other, media) = world("models-cover-borrowed", &[("covers", "Mine"), ("covers", "Other")], true);
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    games.change(&json!({ "op": "save_edits", "path": mine.path, "changes": { "coverfrom": other.path } })).unwrap();
    let cover = |title: &str| media.join(format!("snes/covers/{title}.png")).to_string_lossy().into_owned();
    let (shown, in_use) = cards(&source, &mine);
    let borrowed = CoverChoice::Borrowed(other.path.clone());
    assert_eq!(
        shown,
        [(borrowed.clone(), "Borrowed from Other".into(), cover("Other")), (CoverChoice::Found, "Cover".into(), cover("Mine"))],
        "the other game's cover first, by the base's from, then the cover found"
    );
    assert_eq!(in_use, borrowed, "the borrowed cover in use");
    drop(world);
}

#[test]
fn qt_7_8_5_a_kind_the_media_section_does_not_show_is_named_by_its_scraping_row() {
    let (_world, source, mine, _, _) = world("models-cover-words", &[("covers", "Mine"), ("miximages", "Mine"), ("screenshots", "Mine")], false);
    let platform = Platform::load(library()).unwrap();
    let rows = platform.settings_rows().unwrap();
    let label = rows.as_object().unwrap().values().flat_map(|l| l.as_array().unwrap().iter()).find(|r| r["key"] == "ScrapeMiximages").unwrap()["label"].clone();
    let (shown, _) = cards(&source, &mine);
    let words: Vec<&str> = shown.iter().map(|c| c.1.as_str()).collect();
    assert_eq!(words, ["Cover", "Scraped screenshot", label.as_str().unwrap()], "the Media section's words, then the Scraping row's");
}
