//! The disk corpus of sidecars: two as the build wrote them on 2026-11-07 and three edited, each beside the listing it
//! reads to. See EmuSen_RustPlatform.md §47.

use emusen_core_host::sidecar::Sidecar;
use std::path::{Path, PathBuf};

const DISK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/disk");

fn files() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(DISK).unwrap().map(|e| e.unwrap().path()).filter(|p| p.to_str().unwrap().ends_with(".core.json")).collect();
    found.sort();
    found
}

fn outcome(file: &Path) -> String {
    match Sidecar::read(file) {
        None => "none\n".to_string(),
        Some(s) => {
            let settings = match &s.settings {
                serde_json::Value::Array(items) => items.len(),
                _ => usize::MAX,
            };
            format!(
                "library={}\nsha256={}\nabi={}\ncapabilities={}\ndevelopment={}\nsettings={settings}\n{:#?}\n",
                s.library_path.file_name().unwrap().to_str().unwrap(),
                s.sha256,
                s.abi,
                s.capabilities,
                s.development,
                s.info
            )
        }
    }
}

#[test]
fn every_sidecar_reads_to_its_recorded_listing() {
    for file in files() {
        let listing = file.with_extension("").with_extension("listing");
        assert_eq!(outcome(&file), std::fs::read_to_string(&listing).unwrap(), "{}", file.display());
    }
}
