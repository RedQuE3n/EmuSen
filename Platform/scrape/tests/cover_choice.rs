//! A game's own picture as its cover: the service's change `cover` sets it by its media type, the media read then gives it
//! as the cover, a type the game lacks is refused in words, and no type puts the cover found back (EmuSen_Backend.md §30.6).

use emusen_scrape::Scraper;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct World {
    dir: PathBuf,
    scraper: Scraper,
    library: Arc<emusen_library::Library>,
}

const GAME: &str = "/roms/snes/Game.sfc";

/// The scratch world for the scraper and the library tests.
fn world(label: &str) -> World {
    let sqlite = emusen_sqlite::for_tests();
    let dir = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-scrape-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (folder, bytes) in [("covers", "cover"), ("titlescreens", "title"), ("videos", "clip")] {
        let at = dir.join("esde").join("snes").join(folder);
        std::fs::create_dir_all(&at).unwrap();
        let ext = if folder == "videos" { "mp4" } else { "png" };
        std::fs::write(at.join(format!("Game.{ext}")), bytes).unwrap();
    }
    std::fs::write(dir.join("esde").join("snes").join("covers").join("Other.png"), "other").unwrap();
    let library = Arc::new(emusen_library::Library::open(sqlite.clone(), &dir.join("games.db").to_string_lossy()).unwrap());
    let settings = json!({ "developer_files": false, "openemu": false, "sources_alone": true, "esde_media": dir.join("esde").to_string_lossy(), "cores": [["snes"]], "test": { "offline": true } });
    let scraper = Scraper::open(sqlite, &dir.join("Media").to_string_lossy(), Some(library.clone()), &[], &settings.to_string()).unwrap();
    World { dir, scraper, library }
}

impl World {
    fn file(&self, folder: &str, ext: &str) -> String {
        self.dir.join("esde").join("snes").join(folder).join(format!("Game.{ext}")).to_string_lossy().into_owned()
    }

    fn cover(&self) -> Value {
        serde_json::from_str(&self.scraper.get_request(&json!({ "what": "media", "path": GAME, "system": "snes", "type": "cover" }).to_string()).unwrap()).unwrap()
    }

    fn choose(&self, kind: Value) -> Result<String, emusen_scrape::Error> {
        self.scraper.change_request(&json!({ "op": "cover", "path": GAME, "system": "snes", "type": kind }).to_string())
    }

    fn edits(&self) -> Value {
        serde_json::from_str(&self.library.get_request(&json!({ "what": "edits", "path": GAME }).to_string()).unwrap()).unwrap()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn a_games_own_picture_is_set_as_its_cover_read_back_and_reset() {
    let w = world("cover-choice");
    assert_eq!(w.cover(), json!({ "source": "esde", "path": w.file("covers", "png"), "type": "cover" }), "the cover found, before any choice");
    assert_eq!(w.choose("titlescreen".into()).unwrap(), "true");
    assert_eq!(w.cover(), json!({ "source": "esde", "path": w.file("titlescreens", "png"), "type": "titlescreen" }), "the title screen is the cover");
    assert_eq!(w.edits()[emusen_library::records::COVER_TYPE], "titlescreen", "stored as the game's edit in games.db");
    w.choose(Value::Null).unwrap();
    assert_eq!(w.cover()["path"], w.file("covers", "png"), "no type puts the cover found back");
    assert_eq!(w.edits(), json!({}), "and nothing of the choice is left");
    assert_eq!(std::fs::read(w.file("covers", "png")).unwrap(), b"cover", "no picture was written over");
    assert_eq!(std::fs::read(w.file("titlescreens", "png")).unwrap(), b"title");
}

#[test]
fn a_type_the_game_lacks_or_that_is_no_picture_is_refused_in_words_and_nothing_is_written() {
    let w = world("cover-refused");
    w.choose("titlescreen".into()).unwrap();
    let lacking = w.choose("marquee".into()).unwrap_err();
    assert_eq!(lacking.status(), emusen_scrape::abi::BAD_REQUEST);
    assert_eq!(lacking.words(), "Game has no picture of the type marquee to be its cover.");
    let clip = w.choose("video".into()).unwrap_err();
    assert_eq!(clip.words(), "video is not a kind of picture, so it cannot be a game's cover.");
    assert_eq!(w.cover()["type"], "titlescreen", "the choice made before stands");
}

#[test]
fn a_chosen_picture_gone_from_disk_leaves_the_cover_found_and_the_choice_kept() {
    let w = world("cover-gone");
    w.choose("titlescreen".into()).unwrap();
    std::fs::remove_file(w.file("titlescreens", "png")).unwrap();
    assert_eq!(w.cover()["path"], w.file("covers", "png"));
    assert_eq!(w.edits()[emusen_library::records::COVER_TYPE], "titlescreen");
}

#[test]
fn choosing_a_picture_of_its_own_ends_a_cover_borrowed_from_another_game() {
    let w = world("cover-borrowed");
    let changes = json!({ emusen_library::records::COVER_FROM: "/roms/snes/Other.sfc" });
    w.library.change_request(&json!({ "op": "save_edits", "path": GAME, "changes": changes }).to_string()).unwrap();
    w.choose("titlescreen".into()).unwrap();
    assert_eq!(w.edits(), json!({ "covertype": "titlescreen" }));
    assert!(Path::new(&w.file("titlescreens", "png")).exists());
}

#[test]
fn choosing_the_cover_found_ends_a_cover_borrowed_from_another_game() {
    let w = world("cover-found-borrowed");
    let changes = json!({ emusen_library::records::COVER_FROM: "/roms/snes/Other.sfc" });
    w.library.change_request(&json!({ "op": "save_edits", "path": GAME, "changes": changes }).to_string()).unwrap();
    w.choose(Value::Null).unwrap();
    assert_eq!(w.edits(), json!({}), "the borrowing ends with the choice of the cover found");
    assert_eq!(w.cover()["path"], w.file("covers", "png"));
}

#[test]
fn a_cover_borrowed_from_another_game_is_read_with_that_game_and_ended_by_a_choice() {
    let w = world("cover-borrowed-read");
    let other = "/roms/snes/Other.sfc";
    let changes = json!({ emusen_library::records::COVER_FROM: other });
    w.library.change_request(&json!({ "op": "save_edits", "path": GAME, "changes": changes }).to_string()).unwrap();
    let other_cover = w.dir.join("esde").join("snes").join("covers").join("Other.png").to_string_lossy().into_owned();
    assert_eq!(w.cover(), json!({ "source": "other_game", "path": other_cover, "type": "cover", "from": other }), "the other game's cover, and which game");
    w.choose(Value::Null).unwrap();
    assert_eq!(w.cover(), json!({ "source": "esde", "path": w.file("covers", "png"), "type": "cover" }), "the cover found once the borrowing ends");
}

#[test]
fn the_cover_found_is_read_without_the_choices_while_a_picture_is_chosen() {
    let w = world("cover-found-read");
    w.choose("titlescreen".into()).unwrap();
    let found = |choices: bool| -> Value {
        let request = json!({ "what": "media", "path": GAME, "system": "snes", "type": "cover", "choices": choices });
        serde_json::from_str(&w.scraper.get_request(&request.to_string()).unwrap()).unwrap()
    };
    assert_eq!(found(false), json!({ "source": "esde", "path": w.file("covers", "png"), "type": "cover" }), "the cover found");
    assert_eq!(found(true)["path"], w.file("titlescreens", "png"), "the choice, as media gives by default");
}
