//! The library over a generated tree and a fresh `games.db`, with the e_sqlite3 the program ships: from
//! `EMUSEN_SQLITE`, else the NuGet cache's copy; a machine with neither says so and runs nothing. The parity class in
//! WiseMan holds every answer to the C#'s (EmuSen_Backend.md §28.6).

use emusen_library::Library;
use emusen_sqlite::Sqlite;
use std::path::PathBuf;
use std::sync::Arc;

fn sqlite() -> Option<Arc<Sqlite>> {
    let mut candidates: Vec<PathBuf> = std::env::var_os("EMUSEN_SQLITE").map(PathBuf::from).into_iter().collect();
    if let Some(home) = std::env::var_os("HOME") {
        let root = PathBuf::from(home).join(".nuget/packages/sqlite");
        if let Ok(versions) = std::fs::read_dir(&root) {
            let mut found: Vec<PathBuf> = versions.flatten().map(|v| v.path().join("runtimes/linux-x64/native/libe_sqlite3.so")).filter(|p| p.exists()).collect();
            found.sort();
            candidates.extend(found.into_iter().rev());
        }
    }
    candidates.into_iter().find_map(|p| Sqlite::open(&p.to_string_lossy()).ok().map(Arc::new))
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
        eprintln!("no e_sqlite3 found: nothing was run");
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
    let id = lib.identity_request(&format!("{{\"path\":{},\"record\":true}}", emusen_dotnet::json::quoted(&b))).unwrap();
    assert!(id.contains("\"system\":\"nes\"") && id.contains("\"cached\":false"), "{id}");
    assert!(!id.contains("\"sha1\""), "a 1.0 request sees no sha1: {id}");
    let with = lib.identity_request(&format!("{{\"path\":{},\"sha1\":true}}", emusen_dotnet::json::quoted(&b))).unwrap();
    assert!(with.contains("\"crc32\":\"") && with.contains("\"sha1\":\"2fbb1a94481697640581db35542dabf3dee0a72b\""), "{with}");
    lib.change_request(&format!("{{\"op\":\"toggle_favourite\",\"path\":{}}}", emusen_dotnet::json::quoted(&b))).unwrap();
    let id = lib.change_request("{\"op\":\"create_collection\",\"name\":\"RPGs\",\"at\":\"2026-10-18T12:00:00.0000000+02:00\"}").unwrap();
    lib.change_request(&format!("{{\"op\":\"add_to_collection\",\"id\":{id},\"path\":{}}}", emusen_dotnet::json::quoted(&b))).unwrap();

    std::fs::rename(&b, t.at("roms/nes/renamed.nes")).unwrap();
    lib.scan(&[t.at("roms")], &systems).unwrap();
    assert_eq!(lib.change_request("{\"op\":\"reclaim\"}").unwrap(), "{\"moved\":1}");
    let moved = lib.get_request(&format!("{{\"what\":\"game\",\"path\":{}}}", emusen_dotnet::json::quoted(&t.at("roms/nes/renamed.nes")))).unwrap();
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
        emusen_dotnet::json::quoted(&t.at("roms")),
        emusen_dotnet::json::quoted(&t.at("packages")),
        emusen_dotnet::json::quoted(&t.at("compressed"))
    );
    assert_eq!(lib.scan_request(&request).unwrap(), 4);
    let games = lib.games_request("").unwrap();
    let q = |s: &str| emusen_dotnet::json::quoted(s);
    assert!(games.contains(&format!("{{\"path\":{},\"system\":\"nes\",\"package\":{}}}", q(&t.at("roms/nes/Claimed.nes")), q(&t.at("packages/NES/Claimed.nes")))), "{games}");
    assert!(games.contains(&format!("{{\"path\":{},\"system\":\"psx\",\"package\":{}}}", q(&t.at("compressed/psx/Disc.chd")), q(&t.at("packages/PlayStation/Disc.chd")))), "{games}");
    assert!(games.contains(&format!("{{\"path\":{},\"system\":\"nes\",\"package\":null}}", q(&t.at("roms/nes/Free.nes")))), "{games}");
    assert!(games.contains("Stray.chd") && !games.contains("Disc.cue") && !games.contains("Inside.nes") && !games.contains("Gone.nes"), "{games}");
    assert!(games.matches("Claimed.nes\"").count() == 1, "{games}");
    // Without the two folders the scan is the one before packages.
    assert_eq!(lib.scan(&[t.at("roms")], &systems).unwrap(), 3);
}
