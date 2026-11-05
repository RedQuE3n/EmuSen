//! One scraper a store across processes (emusen_scrape.h 1.1): a second handle, in another process or this one, is refused while the first holds it - see EmuSen_LibraryTools.md §8.8.

use emusen_scrape::{Error, Scraper, abi};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SETTINGS: &str = r#"{"developer_files":false,"openemu":false,"test":{"offline":true}}"#;

fn open(dir: &Path) -> Result<Scraper, Error> {
    Scraper::open(emusen_sqlite::for_tests().expect("SQLite for the scraper"), &dir.to_string_lossy(), None, &[], SETTINGS)
}

fn refused(r: Result<Scraper, Error>) -> Error {
    match r {
        Ok(_) => panic!("a second scraper opened a held store"),
        Err(e) => e,
    }
}

fn wait_for(file: &Path) {
    let until = Instant::now() + Duration::from_secs(60);
    while !file.exists() {
        assert!(Instant::now() < until, "{} never appeared", file.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_second_process_is_refused_while_the_first_holds_the_store() {
    if let Some(dir) = std::env::var_os("SCRAPE_LOCK_HOLDER").map(PathBuf::from) {
        let held = open(&dir).expect("the holder opens the store");
        std::fs::write(dir.join("held"), b"").unwrap();
        wait_for(&dir.join("release"));
        drop(held);
        return;
    }
    if emusen_sqlite::for_tests().is_none() {
        return;
    }
    let dir = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-scrape-lock-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut holder = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["a_second_process_is_refused_while_the_first_holds_the_store", "--exact", "--nocapture", "--test-threads=1"])
        .env("SCRAPE_LOCK_HOLDER", &dir)
        .spawn()
        .unwrap();
    wait_for(&dir.join("held"));

    let other = refused(open(&dir));
    assert_eq!(other.status(), abi::STORE_IN_USE);
    assert!(other.words().contains("in use by another program") && other.words().contains("Close the other"), "{}", other.words());

    std::fs::write(dir.join("release"), b"").unwrap();
    assert!(holder.wait().unwrap().success(), "the holding process failed");
    let mine = open(&dir).expect("the store opens once the other process let it go");
    let again = refused(open(&dir));
    assert_eq!(again.status(), abi::STORE_IN_USE);
    assert!(again.words().contains("EmuSen's scraper in this program"), "{}", again.words());
    drop(mine);
    drop(open(&dir).expect("closing a handle lets the store go"));
    let _ = std::fs::remove_dir_all(&dir);
}

// A handle for finding pictures alone (1.2) makes no folder and no media.db, holds no lock, and finds a picture already there.
#[test]
fn a_handle_for_sources_alone_makes_nothing_on_disk_and_holds_no_lock() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let dir = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-scrape-alone-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = dir.join("Media");
    let esde = dir.join("esde");
    std::fs::create_dir_all(esde.join("snes").join("covers")).unwrap();
    std::fs::write(esde.join("snes").join("covers").join("Game.png"), b"png").unwrap();
    let settings = format!(r#"{{"developer_files":false,"openemu":false,"sources_alone":true,"esde_media":{:?},"test":{{"offline":true}}}}"#, esde.to_string_lossy());
    let alone = Scraper::open(sqlite.clone(), &store.to_string_lossy(), None, &[], &settings).expect("a handle for sources alone opens");
    let game = emusen_scrape::media::Game { file: "/roms/snes/Game.sfc", folder_path: String::new(), folder: false };
    let found = alone.locate("snes", &game, "cover").map(|(_, p)| p);
    assert_eq!(found.as_deref(), Some(esde.join("snes").join("covers").join("Game.png").to_string_lossy().as_ref()));
    assert!(!store.exists(), "a handle for sources alone made {}", store.display());
    let full = open(&store).expect("a full handle opens beside one for sources alone");
    assert!(store.join("media.db").exists());
    drop(full);
    drop(alone);
    let _ = std::fs::remove_dir_all(&dir);
}
