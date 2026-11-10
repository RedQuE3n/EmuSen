//! The library over a generated tree and a fresh `games.db`, with the e_sqlite3 the program ships: from
//! `EMUSEN_SQLITE`, else the NuGet cache's copy; a machine with neither says so and runs nothing. The parity class in
//! WiseMan holds every answer to the C#'s (EmuSen_Backend.md §28.6).

use emusen_library::Library;
use emusen_sqlite::Sqlite;
use std::path::PathBuf;
use std::sync::Arc;

fn sqlite() -> Option<Arc<Sqlite>> {
    emusen_sqlite::for_tests()
}

struct Temp(PathBuf);

impl Temp {
    fn new(label: &str) -> Temp {
        let dir = std::env::temp_dir().join(format!("emusen-library-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Temp(dir)
    }

    fn put(&self, relative: &str, bytes: &[u8]) -> String {
        let at = self.0.join(relative);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(&at, bytes).unwrap();
        at.to_string_lossy().into_owned()
    }

    fn at(&self, relative: &str) -> String {
        self.0.join(relative).to_string_lossy().into_owned()
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_tree_is_scanned_shelved_and_a_renamed_game_keeps_its_row() {
    let Some(sqlite) = sqlite() else {
        return;
    };
    let t = Temp::new("scan");
    let mut colour = vec![0u8; 0x150];
    colour[0x143] = 0xC0;
    t.put("roms/gb/Colour.gb", &colour);
    t.put("roms/gb/Plain.gb", &[0u8; 0x150]);
    t.put("roms/nes/b.nes", b"NES\x1a");
    t.put("roms/.hidden/c.nes", b"");
    t.put("roms/nes/notes.txt", b"");
    let lib = Library::open(sqlite.clone(), &t.at("lib/games.db")).unwrap();
    let systems: Vec<String> = ["nes", "gb", "gbc"].iter().map(|s| s.to_string()).collect();
    assert_eq!(lib.scan(&[t.at("roms")], &systems).unwrap(), 3);
    let games = lib.games_request("").unwrap();
    assert!(games.starts_with("{\"status\":\"ok\""), "{games}");
    assert!(games.find("Colour.gb").unwrap() < games.find("Plain.gb").unwrap(), "{games}");
    assert!(lib.games_request("{\"system\":\"gbc\"}").unwrap().contains("Colour.gb"));
    assert!(!lib.games_request("{\"system\":\"gbc\"}").unwrap().contains("Plain.gb"));

    let b = t.at("roms/nes/b.nes");
    let id = lib.identity_request(&format!("{{\"path\":{},\"record\":true}}", serde_json::json!(&b).to_string())).unwrap();
    assert!(id.contains("\"system\":\"nes\"") && id.contains("\"cached\":false"), "{id}");
    assert!(!id.contains("\"sha1\""), "a 1.0 request sees no sha1: {id}");
    let with = lib.identity_request(&format!("{{\"path\":{},\"sha1\":true}}", serde_json::json!(&b).to_string())).unwrap();
    assert!(with.contains("\"crc32\":\"") && with.contains("\"sha1\":\"2fbb1a94481697640581db35542dabf3dee0a72b\""), "{with}");
    lib.change_request(&format!("{{\"op\":\"toggle_favourite\",\"path\":{}}}", serde_json::json!(&b).to_string())).unwrap();
    let id = lib.change_request("{\"op\":\"create_collection\",\"name\":\"RPGs\",\"at\":\"2026-10-18T12:00:00.0000000+02:00\"}").unwrap();
    lib.change_request(&format!("{{\"op\":\"add_to_collection\",\"id\":{id},\"path\":{}}}", serde_json::json!(&b).to_string())).unwrap();

    std::fs::rename(&b, t.at("roms/nes/renamed.nes")).unwrap();
    lib.scan(&[t.at("roms")], &systems).unwrap();
    assert_eq!(lib.change_request("{\"op\":\"reclaim\"}").unwrap(), "{\"moved\":1}");
    let moved = lib.get_request(&format!("{{\"what\":\"game\",\"path\":{}}}", serde_json::json!(&t.at("roms/nes/renamed.nes")).to_string())).unwrap();
    assert!(moved.contains("\"favourite\":true") && moved.contains(&format!("\"collections\":[{id}]")), "{moved}");
    drop(lib);

    let reopened = Library::open(sqlite, &t.at("lib/games.db")).unwrap();
    assert!(reopened.get_request("{\"what\":\"collections\"}").unwrap().contains("\"count\":1"));
}

#[test]
fn a_newer_file_is_refused_unwritten() {
    let Some(sqlite) = sqlite() else { return };
    let t = Temp::new("newer");
    drop(Library::open(sqlite.clone(), &t.at("games.db")).unwrap());
    {
        let db = emusen_sqlite::Connection::open(sqlite.clone(), &t.at("games.db")).unwrap();
        db.execute("PRAGMA user_version = 7").unwrap();
    }
    let refused = Library::open(sqlite, &t.at("games.db")).err().unwrap();
    assert_eq!(refused.status(), emusen_library::abi::NEWER_FILE);
    assert_eq!(refused.words(), "games.db is schema 7; this build knows 6.");
}

#[test]
fn a_package_is_one_entry_and_what_it_claims_is_not_listed_again() {
    let Some(sqlite) = sqlite() else { return };
    let t = Temp::new("packages");
    t.put("roms/nes/Claimed.nes", b"NES\x1aclaimed");
    t.put("roms/nes/Free.nes", b"NES\x1afree");
    t.put("roms/psx/Disc.cue", b"FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    t.put("roms/psx/Disc.bin", &[0u8; 2352]);
    t.put("compressed/psx/Disc.chd", b"not read");
    t.put("compressed/psx/Stray.chd", b"not read either");
    let manifest = |game: &str, extra: &str| format!("{{\"format\":\"emusen-package\",\"version\":\"1.0\",\"system\":\"nes\",\"mode\":\"linked\",\"game\":{game}{extra}}}");
    t.put("packages/NES/Claimed.nes/package.json", manifest("{\"role\":\"image\",\"where\":\"library\",\"path\":\"nes/Claimed.nes\"}", "").as_bytes());
    t.put("packages/NES/Claimed.nes/saves/Claimed.nes", b"NES\x1anot a game of its own");
    t.put(
        "packages/PlayStation/Disc.chd/package.json",
        manifest("{\"role\":\"image\",\"where\":\"compressed\",\"path\":\"psx/Disc.chd\"}", ",\"container\":{\"sources\":[{\"where\":\"library\",\"path\":\"psx/Disc.cue\"},{\"where\":\"library\",\"path\":\"psx/Disc.bin\"}]}").replace("\"nes\"", "\"psx\"").as_bytes(),
    );
    t.put("packages/NES/Gone.nes/package.json", manifest("{\"role\":\"image\",\"where\":\"library\",\"path\":\"nes/Gone.nes\"}", "").as_bytes());
    t.put("packages/NES/Broken.nes/package.json", b"{ not json");
    t.put("packages/NES/Broken.nes/Inside.nes", b"NES\x1a");
    let lib = Library::open(sqlite, &t.at("lib/games.db")).unwrap();
    let systems: Vec<String> = ["nes", "psx"].iter().map(|s| s.to_string()).collect();
    let request = format!(
        "{{\"roots\":[{}],\"systems\":[\"nes\",\"psx\"],\"packages\":{},\"compressed\":{}}}",
        serde_json::json!(&t.at("roms")).to_string(),
        serde_json::json!(&t.at("packages")).to_string(),
        serde_json::json!(&t.at("compressed")).to_string()
    );
    assert_eq!(lib.scan_request(&request).unwrap(), 4);
    let games = lib.games_request("").unwrap();
    let q = |s: &str| serde_json::json!(s).to_string();
    assert!(games.contains(&format!("{{\"path\":{},\"system\":\"nes\",\"package\":{},\"folder\":\"\"}}", q(&t.at("roms/nes/Claimed.nes")), q(&t.at("packages/NES/Claimed.nes")))), "{games}");
    assert!(games.contains(&format!("{{\"path\":{},\"system\":\"psx\",\"package\":{},\"folder\":\"\"}}", q(&t.at("compressed/psx/Disc.chd")), q(&t.at("packages/PlayStation/Disc.chd")))), "{games}");
    assert!(games.contains(&format!("{{\"path\":{},\"system\":\"nes\",\"package\":null,\"folder\":\"\"}}", q(&t.at("roms/nes/Free.nes")))), "{games}");
    assert!(games.contains("Stray.chd") && !games.contains("Disc.cue") && !games.contains("Inside.nes") && !games.contains("Gone.nes"), "{games}");
    let parsed = serde_json::from_str::<serde_json::Value>(&games).unwrap();
    let Some(serde_json::Value::Array(entries)) = parsed.get("games") else { panic!("{games}") };
    let claimed: Vec<_> = entries.iter().filter(|e| e.get("path").and_then(|p| p.as_str()) == Some(t.at("roms/nes/Claimed.nes").as_str())).collect();
    assert_eq!(claimed.len(), 1, "{games}");
    assert_eq!(claimed[0].get("package").and_then(|p| p.as_str()), Some(t.at("packages/NES/Claimed.nes").as_str()), "{games}");
    // Without the two folders the scan is the one before packages.
    assert_eq!(lib.scan(&[t.at("roms")], &systems).unwrap(), 3);
}

#[test]
fn each_game_names_its_folder_below_the_console_folder_and_a_1_2_reader_reads_the_answer_as_before() {
    let Some(sqlite) = sqlite() else { return };
    let t = Temp::new("folders");
    t.put("roms/nes/Top.nes", b"NES\x1atop");
    t.put("roms/nes/RPG/Square/Deep.nes", b"NES\x1adeep");
    t.put("roms/nes/Tower Set/Spire.nes", b"NES\x1aspire");
    let lib = Library::open(sqlite, &t.at("lib/games.db")).unwrap();
    assert_eq!(lib.scan(&[t.at("roms")], &["nes".to_string()]).unwrap(), 3);
    let answer = lib.games_request("").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&answer).unwrap();
    let folders: Vec<(String, String)> = parsed["games"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| (g["path"].as_str().unwrap().rsplit('/').next().unwrap().to_owned(), g["folder"].as_str().unwrap().to_owned()))
        .collect();
    let expected = [("Deep.nes", "RPG/Square"), ("Spire.nes", "Tower Set"), ("Top.nes", "")];
    assert_eq!(folders, expected.map(|(f, d)| (f.to_owned(), d.to_owned())), "{answer}");

    // A client of 1.2 reads the answer into its own shapes, skipping the key it does not know, as the header's rule has it.
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Games12 {
        status: String,
        roots: Vec<String>,
        games: Vec<Game12>,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Game12 {
        path: String,
        system: Option<String>,
        package: Option<String>,
    }
    let old: Games12 = serde_json::from_str(&answer).expect("a 1.2 reader reads 1.3's answer");
    assert_eq!((old.status.as_str(), old.roots.clone(), old.games.len()), ("ok", vec![t.at("roms")], 3));
    assert_eq!(old.games[0], Game12 { path: t.at("roms/nes/RPG/Square/Deep.nes"), system: Some("nes".into()), package: None });
}

#[test]
fn a_games_television_standard_is_read_set_refused_outside_the_two_and_kept_by_clear() {
    let Some(sqlite) = sqlite() else {
        return;
    };
    let t = Temp::new("standard");
    let lib = Library::open(sqlite, &t.at("lib/games.db")).unwrap();
    let game = t.at("roms/snes/Game.sfc");
    let read = |lib: &Library| -> serde_json::Value { serde_json::from_str(&lib.get_request(&serde_json::json!({ "what": "standard", "path": game }).to_string()).unwrap()).unwrap() };
    let set = |lib: &Library, standard: serde_json::Value| lib.change_request(&serde_json::json!({ "op": "set_standard", "path": game, "standard": standard }).to_string());
    assert_eq!(read(&lib), serde_json::json!({ "value": null, "values": ["ntsc", "pal"] }), "the game's own until one is chosen");
    set(&lib, "pal".into()).unwrap();
    assert_eq!(read(&lib)["value"], "pal");
    assert!(set(&lib, "secam".into()).is_err(), "a standard outside the two is refused");
    assert_eq!(read(&lib)["value"], "pal", "and nothing is written");
    lib.change_request(&serde_json::json!({ "op": "clear_edits", "path": game }).to_string()).unwrap();
    assert_eq!(read(&lib)["value"], "pal", "the editor's Clear keeps the player's choice");
    set(&lib, serde_json::Value::Null).unwrap();
    assert_eq!(read(&lib)["value"], serde_json::Value::Null, "null follows the game again");
}
