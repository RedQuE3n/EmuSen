//! The path index keeps a file's three hashes at its size and time, so a run reads a file once until it changes - see EmuSen_LibraryTools.md §9.11.

use emusen_scrape::store::{Cached, Store};
use std::path::PathBuf;

#[test]
fn a_file_s_three_hashes_are_kept_at_its_size_and_time_and_a_changed_file_misses() {
    let Some(sqlite) = emusen_sqlite::for_tests() else { return };
    let dir = std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-scrape-hashes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    {
        let store = Store::open(sqlite, &dir.to_string_lossy()).unwrap();
        assert_eq!(store.cached_file("/r/ps2/A.iso", 10, 5).unwrap(), None);
        store.remember_file("/r/ps2/A.iso", 10, 5, "md5a", "crca", "sha1a").unwrap();
        let kept = Cached { md5: "md5a".into(), crc32: Some("crca".into()), sha1: Some("sha1a".into()), fingerprint: String::new() };
        assert_eq!(store.cached_file("/r/ps2/A.iso", 10, 5).unwrap(), Some(kept));
        assert_eq!(store.cached_file("/r/ps2/A.iso", 11, 5).unwrap(), None, "a new size is a new file");
        assert_eq!(store.cached_file("/r/ps2/A.iso", 10, 6).unwrap(), None, "a new time is a new file");
        store.remember_file("/r/ps2/A.iso", 11, 6, "md5b", "crcb", "sha1b").unwrap();
        assert_eq!(store.cached_file("/r/ps2/A.iso", 10, 5).unwrap(), None, "the old size and time are forgotten");
        assert_eq!(store.cached_file("/r/ps2/A.iso", 11, 6).unwrap().map(|c| (c.crc32, c.sha1)), Some((Some("crcb".into()), Some("sha1b".into()))));
    }
    let _ = std::fs::remove_dir_all(&dir);
}
