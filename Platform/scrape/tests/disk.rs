//! The disk corpus of the scraper's member file: as the platform wrote it on 2026-11-07, as the old two boxes wrote
//! it, and as a person may edit it, each beside the account it reads to. The names and passwords are made up; the
//! real file is never copied here. See EmuSen_RustPlatform.md §47.

use emusen_scrape::credentials::Member;
use std::path::{Path, PathBuf};

const DISK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/disk");

fn files() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(Path::new(DISK).join("member")).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
    found.sort();
    found
}

fn outcome(m: &Member) -> String {
    format!("user={}\npassword={}\nverified={}\n", m.user, m.password, m.verified.map_or("none".to_string(), |t| t.to_string()))
}

fn read(file: &Path) -> Member {
    Member::from_text(&String::from_utf8_lossy(&std::fs::read(file).unwrap()))
}

#[test]
fn every_member_file_reads_to_its_recorded_account_and_is_written_back_the_same() {
    for file in files() {
        let member = read(file.as_path());
        assert_eq!(outcome(&member), std::fs::read_to_string(file.with_extension("member")).unwrap(), "{}", file.display());
        assert_eq!(outcome(&Member::from_text(&member.to_text())), outcome(&member), "{}", file.display());
    }
}
