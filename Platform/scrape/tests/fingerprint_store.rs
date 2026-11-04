//! A big file's record keyed by its fingerprint: the migration from schema 4 keeps every record, a fingerprinted game is
//! found by its path and its art by its key after the store is opened again, and its checksums arrive in the path index
//! later without moving its rows - see EmuSen_LibraryTools.md §9.12.

use emusen_scrape::store::{Key, MIGRATIONS, Record, SCHEMA_VERSION, State, Store, Stored};
use emusen_sqlite::Connection;
use std::path::PathBuf;

fn folder(name: &str) -> PathBuf {
    let dir = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-scrape-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn count(db: &Connection, table: &str) -> i64 {
    let mut n = 0;
    db.query(&format!("SELECT COUNT(*) FROM {table}"), &[], |row| n = row.int64(0)).unwrap();
    n
}

#[test]
fn the_migration_from_schema_4_keeps_every_record() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let dir = folder("migrate4");
    let file = dir.join("media.db").to_string_lossy().into_owned();
    {
        let db = Connection::open(sqlite.clone(), &file).unwrap();
        for step in &MIGRATIONS[..4] {
            db.execute(step).unwrap();
        }
        db.execute("PRAGMA user_version = 4").unwrap();
        db.execute("INSERT INTO scrape_game (md5, bytes, status, game_id, rating, fetched_at, name, matched_by, kinds_known) VALUES ('aa', 1, 'Found', 7, 0.8, '2026-01-01T00:00:00.000Z', 'Café', 'file', 'cover'), ('bb', 2, 'Unknown', NULL, NULL, '2026-01-02T00:00:00.000Z', NULL, NULL, NULL)").unwrap();
        db.execute("INSERT INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ('aa', 1, 'cover', 'snes/covers/A.png', 'us', 'ff', '2026-01-01T00:00:00.000Z'), ('aa', 1, 'marquee', 'snes/marquees/A.png', NULL, NULL, '2026-01-01T00:00:00.000Z')").unwrap();
        db.execute("INSERT INTO scrape_file (path, bytes, modified, md5, crc32, sha1) VALUES ('/r/snes/A.sfc', 1, 5, 'aa', 'c1', 's1'), ('/r/snes/B.sfc', 2, 6, 'bb', NULL, NULL)").unwrap();
        db.execute("INSERT INTO scrape_queue (path, system, priority, seq) VALUES ('/r/snes/B.sfc', 'snes', 2, 1)").unwrap();
        db.execute("INSERT INTO quota_day (day, requests) VALUES ('2026-01-01', 3)").unwrap();
        db.execute("INSERT INTO store_step VALUES ('folders', '2026-01-01T00:00:00.000Z')").unwrap();
    }
    let store = Store::open(sqlite.clone(), &dir.to_string_lossy()).unwrap();
    let db = Connection::open(sqlite, &file).unwrap();
    let mut version = 0;
    db.query("PRAGMA user_version", &[], |row| version = row.int64(0)).unwrap();
    assert_eq!(version, SCHEMA_VERSION);
    assert_eq!((count(&db, "scrape_game"), count(&db, "scrape_media"), count(&db, "scrape_file"), count(&db, "scrape_queue"), count(&db, "quota_day"), count(&db, "store_step")), (2, 2, 2, 1, 1, 1));
    let a = store.game(&Key::md5("aa", 1)).unwrap().unwrap();
    assert_eq!((a.name.as_deref(), a.game_id, a.matched_by.as_deref(), a.kinds_known.as_deref(), a.fingerprint.as_str()), (Some("Café"), Some(7), Some("file"), Some("cover"), ""));
    assert_eq!(store.game(&Key::md5("bb", 2)).unwrap().unwrap().state, State::Unknown);
    assert_eq!(store.media(&Key::md5("aa", 1)).unwrap().iter().map(|m| m.relative_path.as_str()).collect::<Vec<_>>(), ["snes/covers/A.png", "snes/marquees/A.png"]);
    assert_eq!(store.found_for("/r/snes/A.sfc").unwrap().map(|r| r.md5), Some("aa".into()));
    let cached = store.cached_file("/r/snes/A.sfc", 1, 5).unwrap().unwrap();
    assert_eq!((cached.md5.as_str(), cached.crc32.as_deref(), cached.fingerprint.as_str()), ("aa", Some("c1"), ""));
    drop((store, db));
    let _ = std::fs::remove_dir_all(&dir);
}

fn found(key: &Key, name: &str) -> Record {
    let mut r = Record::new(key, State::Found, 0);
    r.name = Some(name.into());
    r.matched_by = Some("name+size".into());
    r
}

#[test]
fn a_fingerprinted_game_and_its_art_are_found_again_after_a_restart_and_its_checksums_arrive_without_moving_it() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let dir = folder("fingerprint");
    let root = dir.to_string_lossy().into_owned();
    let size = 4_700_000_000;
    let (one, two) = (Key::fingerprint("f1", size), Key::fingerprint("f2", size));
    {
        let store = Store::open(sqlite.clone(), &root).unwrap();
        store.remember_fingerprint("/r/ps2/One.iso", size, 9, "f1").unwrap();
        store.remember_fingerprint("/r/ps2/Two.iso", size, 9, "f2").unwrap();
        store.record(&found(&one, "One")).unwrap();
        store.record(&found(&two, "Two")).unwrap();
        store.record_media(&one, &Stored { ty: "cover".into(), relative_path: "ps2/covers/One.png".into(), region: None, sha1: None }, 0).unwrap();
        assert_eq!(store.unhashed().unwrap().len(), 2);
    }
    let store = Store::open(sqlite.clone(), &root).unwrap();
    assert_eq!(store.found_for("/r/ps2/One.iso").unwrap().map(|r| r.name), Some(Some("One".into())), "two discs of one size keep their own records");
    assert_eq!(store.found_for("/r/ps2/Two.iso").unwrap().map(|r| r.name), Some(Some("Two".into())));
    assert_eq!(store.media(&one).unwrap().len(), 1);
    assert!(store.media(&Key::md5("", size)).unwrap().is_empty(), "a fingerprint never answers for an empty MD5");
    let outcomes = store.outcomes_by_path().unwrap();
    assert!(outcomes.iter().any(|(p, s, cover)| p == "/r/ps2/One.iso" && *s == State::Found && *cover));
    let cached = store.cached_file("/r/ps2/One.iso", size, 9).unwrap().unwrap();
    assert_eq!((cached.md5.as_str(), cached.crc32.clone(), cached.key(size)), ("", None, one.clone()));

    assert!(!store.fill_hashes("/r/ps2/One.iso", size, 10, "f1", "m1", "c1", "s1").unwrap(), "a changed file is not filled");
    assert!(store.fill_hashes("/r/ps2/One.iso", size, 9, "f1", "m1", "c1", "s1").unwrap());
    let cached = store.cached_file("/r/ps2/One.iso", size, 9).unwrap().unwrap();
    assert_eq!((cached.md5.as_str(), cached.crc32.as_deref(), cached.sha1.as_deref(), cached.key(size)), ("m1", Some("c1"), Some("s1"), one.clone()));
    assert_eq!(store.unhashed().unwrap().iter().map(|u| u.0.as_str()).collect::<Vec<_>>(), ["/r/ps2/Two.iso"]);
    let kept = store.found_for("/r/ps2/One.iso").unwrap().unwrap();
    assert_eq!((kept.md5.as_str(), kept.fingerprint.as_str(), kept.name.as_deref()), ("", "f1", Some("One")), "the record keeps its fingerprint key");
    assert!(store.game(&Key::md5("m1", size)).unwrap().is_none());
    assert_eq!(store.paths_of(&one).unwrap(), ["/r/ps2/One.iso"]);

    store.remember_file("/r/ps2/One.iso", size, 11, "m2", "c2", "s2").unwrap();
    assert_eq!(store.cached_file("/r/ps2/One.iso", size, 11).unwrap().unwrap().key(size), Key::md5("m2", size), "a file read in full anew is keyed by its MD5");
    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
}
