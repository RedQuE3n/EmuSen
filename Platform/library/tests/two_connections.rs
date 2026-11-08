//! Two handles on one games.db in one process, as a front end's own and the platform's for the scraper and the shells:
//! writes through both under load, none lost and no busy database reaching a caller, and a write that waits out the
//! other's transaction (EmuSen_RustPlatform.md §47.25).

use emusen_library::Library;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

fn games_db(label: &str) -> PathBuf {
    let dir = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-library-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("games.db")
}

fn records(lib: &Library) -> serde_json::Map<String, serde_json::Value> {
    match serde_json::from_str(&lib.get_request(r#"{"what":"records"}"#).unwrap()).unwrap() {
        serde_json::Value::Object(m) => m,
        other => panic!("records are an object: {other}"),
    }
}

#[test]
fn writes_through_two_handles_under_load_are_all_kept_and_none_is_refused_busy() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let path = games_db("two-handles");
    let path = path.to_string_lossy().into_owned();
    let handles = [Arc::new(Library::open(sqlite.clone(), &path).unwrap()), Arc::new(Library::open(sqlite.clone(), &path).unwrap())];
    const EACH: usize = 150;
    let start = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|w| {
            let (lib, start) = (handles[w % 2].clone(), start.clone());
            std::thread::spawn(move || {
                start.wait();
                for i in 0..EACH {
                    let game = format!("/roms/w{w}/g{i}.sfc");
                    lib.change_request(&serde_json::json!({ "op": "set_favourite", "path": game, "favourite": true }).to_string()).unwrap_or_else(|e| panic!("{game}: {}", e.words()));
                    lib.change_request(&serde_json::json!({ "op": "save_edits", "path": game, "changes": { "name": format!("Game {w}.{i}") } }).to_string()).unwrap_or_else(|e| panic!("{game}: {}", e.words()));
                    if i % 10 == 0 {
                        records(&lib);
                    }
                }
            })
        })
        .collect();
    for w in workers {
        w.join().expect("no write failed");
    }
    for lib in &handles {
        let all = records(lib);
        assert_eq!(all.values().filter(|r| r["favourite"] == true).count(), 4 * EACH, "every favourite is seen through each handle");
        let edits: serde_json::Value = serde_json::from_str(&lib.get_request(r#"{"what":"all_edits"}"#).unwrap()).unwrap();
        assert_eq!(edits.as_object().unwrap().len(), 4 * EACH, "every edit is seen through each handle");
    }
}

#[test]
fn a_write_waits_out_the_other_connection_s_transaction() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let path = games_db("held-write");
    let path = path.to_string_lossy().into_owned();
    let lib = Library::open(sqlite.clone(), &path).unwrap();
    let other = emusen_sqlite::Connection::open(sqlite, &path).unwrap();
    other.execute("BEGIN IMMEDIATE; INSERT INTO game (path, favourite) VALUES ('/roms/held.sfc', 1);").unwrap();
    let (held, began) = (Duration::from_millis(1200), Instant::now());
    let release = std::thread::spawn(move || {
        std::thread::sleep(held);
        other.execute("COMMIT;").unwrap();
    });
    lib.change_request(r#"{"op":"set_favourite","path":"/roms/waited.sfc","favourite":true}"#).expect("the write waits for the other's commit rather than fail");
    assert!(began.elapsed() >= held, "the write went in only after the other transaction ended");
    release.join().unwrap();
    let all = records(&lib);
    assert!(all.contains_key("/roms/held.sfc") && all.contains_key("/roms/waited.sfc"), "both writes are kept");
}
