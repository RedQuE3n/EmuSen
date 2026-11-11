//! The shelves' text across the two crates that keep it (EmuSen_Qt.md §4.9): written by qt-models' `esde_shelves` as the
//! library reads it, read back by esde-qt's `shelves` as the view does, every field the view shows as it was written. No
//! window: the program links both crates, so the round trip is tested where they meet.

use emusen_crystal_client::shapes::SystemRow;
use emusen_esde_qt::shelves;
use emusen_platform_client::library::GameFile;
use emusen_qt_models::esde_shelves::{metadata_of, MediaIndex, Metadata, Plays};
use emusen_qt_models::library::{refresh, Engine, LibrarySource};
use serde_json::{json, Value};

const FULL: &str = "/roms/snes/Aurora Drift (Synthetic).sfc";
const NEVER: &str = "/roms/snes/Brass Lantern (Synthetic).sfc";

/// A library of one console: a game with every field the scraper and games.db give, and a game whose plays are negative.
struct Library;

impl LibrarySource for Library {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        let full = "Super Nintendo Entertainment System";
        Ok(vec![SystemRow { id: "snes".into(), name: full.into(), esde_system: "snes".into(), esde_full_name: full.into(), engines: vec!["x".into()], ..SystemRow::default() }])
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok([FULL, NEVER].map(|p| GameFile { path: p.into(), system: Some("snes".into()), package: None, folder: String::new() }).to_vec())
    }
    fn favourites(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(vec![FULL.into()])
    }
    fn recent(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(vec![FULL.into()])
    }
    fn played(&self, _: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        Ok(vec![(FULL.into(), 1_700_000_000)])
    }
    fn engines(&self, _: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(Vec::new())
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        Ok(vec!["snes".into()])
    }
    fn settings(&self) -> Result<Value, String> {
        Ok(json!({ "LibraryStyle": "Theme", "BigPictureTheme": "/themes/t/", "BigPicture": {}, "ShowHiddenGames": false }))
    }
    fn metadata(&self, _: &[GameFile]) -> Result<Vec<(String, Metadata)>, String> {
        let fields = [
            ("name", "Aurora Drift"),
            ("description", "A game invented for the tests."),
            ("developer", "Studio A"),
            ("publisher", "Publisher P"),
            ("genre", "Puzzle"),
            ("players", "1-2"),
            ("releasedate", "19930704T000000"),
            ("rating", "0.8"),
            ("completed", "1"),
            ("kidgame", "1"),
            ("broken", "1"),
            ("nogamecount", "1"),
            ("hidemetadata", "1"),
            ("controller", "snes_mouse"),
            ("altemulator", "Other Core"),
            ("sortname", "Drift, Aurora"),
        ];
        let answer: serde_json::Map<String, Value> = fields.iter().map(|(k, v)| ((*k).to_owned(), json!({ "value": v, "source": "edited" }))).collect();
        Ok(vec![(FULL.into(), metadata_of(&Value::Object(answer)))])
    }
    fn plays(&self, _: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        Ok(vec![(FULL.into(), Plays { count: 7, seconds: 3_600 }), (NEVER.into(), Plays { count: -1, seconds: -5 })])
    }
    fn media_index(&self, _: &[String]) -> Result<MediaIndex, String> {
        let mut index = MediaIndex::default();
        index.insert("snes", "manual", "Aurora Drift (Synthetic)", "/media/snes/manuals/Aurora Drift (Synthetic).pdf");
        index.insert("snes", "cover", "Aurora Drift (Synthetic)", "/media/snes/covers/Aurora Drift (Synthetic).png");
        Ok(index)
    }
}

#[test]
fn every_field_qt_models_writes_is_read_by_the_view_as_written() {
    let read = refresh(&Library).expect("the library read");
    let found = shelves::shelves(&read.esde.shelves, &read.esde.media).expect("the view reads the library's text");
    let games: Vec<_> = found.iter().flat_map(|s| &s.games).collect();
    let g = games.iter().find(|g| g.file == FULL).expect("the game read back");
    assert_eq!(g.name, "Aurora Drift");
    assert_eq!(g.description, None, "a console's row leaves the description to `esdeDescription`, asked on demand");
    let text = [&g.developer, &g.publisher, &g.genre, &g.players, &g.controller, &g.emulator].map(|t| t.as_deref());
    let written = [Some("Studio A"), Some("Publisher P"), Some("Puzzle"), Some("1-2"), Some("snes_mouse"), Some("Other Core")];
    assert_eq!(text, written, "developer, publisher, genre, players, controller, emulator");
    assert_eq!((g.play_count, g.play_time_seconds, g.rating), (7, Some(3_600), Some(0.8)));
    assert!(g.release_date.is_some() && g.last_played.is_some(), "the dates");
    let flags = [g.favorite, g.completed, g.kid_game, g.broken, g.not_counted, g.hide_metadata, g.manual, g.alt_emulator];
    assert_eq!(flags, [true; 8], "favourite, completed, kid game, broken, not counted, hide metadata, manual, alternative emulator");
    assert_eq!(g.media.get("cover").map(String::as_str), Some("/media/snes/covers/Aurora Drift (Synthetic).png"));
    let never = games.iter().find(|g| g.file == NEVER).expect("the game with negative plays read back, the shelves not emptied");
    assert_eq!((never.play_count, never.play_time_seconds), (0, Some(0)), "plays below zero read as none");
}

#[test]
fn a_shelves_text_with_a_key_the_view_does_not_name_is_an_error_not_an_empty_library() {
    let read = refresh(&Library).expect("the library read");
    let renamed = read.esde.shelves.replacen("\"developer\"", "\"maker\"", 1);
    let words = shelves::shelves(&renamed, &read.esde.media).err().expect("an error");
    assert!(words.contains("shelves") && words.contains("maker"), "{words}");
    let typed = read.esde.shelves.replacen("\"play_count\":7", "\"play_count\":\"7\"", 1);
    assert!(typed != read.esde.shelves && shelves::shelves(&typed, &read.esde.media).is_err(), "a field of the wrong type");
    assert!(shelves::shelves(&read.esde.shelves, "{").is_err(), "a media index that does not read");
}
