//! The disk corpus of the runtime's files: a game's own settings and a package's state records, as the platform wrote
//! them on 2026-11-07 and as a person may edit them, each beside what it reads to. See EmuSen_RustPlatform.md §47.

use emusen_crystal::{settings, states};
use std::path::{Path, PathBuf};

const DISK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/disk");

fn files(folder: &str, extension: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(Path::new(DISK).join(folder)).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == extension)).collect();
    found.sort();
    assert!(!found.is_empty(), "{folder} holds no .{extension}");
    found
}

fn expected(file: &Path, extension: &str) -> String {
    std::fs::read_to_string(file.with_extension(extension)).unwrap_or_else(|_| panic!("{} has no .{extension}", file.display()))
}

/// A settings file's values, one `key<TAB>value` line each, in the file's order.
fn values_outcome(file: &Path) -> String {
    settings::values_at(file.to_str().unwrap()).iter().map(|(k, v)| format!("{k}\t{v}\n")).collect()
}

#[test]
fn every_game_settings_file_reads_to_its_recorded_values() {
    for file in files("settings", "json") {
        assert_eq!(values_outcome(&file), expected(&file, "values"), "{}", file.display());
    }
}

#[test]
fn game_settings_written_and_read_again_are_the_same_values() {
    let at = std::env::temp_dir().join(format!("crystal-disk-settings-{}.json", std::process::id()));
    for file in files("settings", "json") {
        let values = settings::values_at(file.to_str().unwrap());
        settings::write_values(at.to_str().unwrap(), &values).unwrap();
        assert_eq!(settings::values_at(at.to_str().unwrap()), values, "{}", file.display());
    }
    let _ = std::fs::remove_file(&at);
}

const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;

/// A stored time as the instant it names, in UTC, so that the corpus does not depend on the zone it is read in; a time with no zone as its text.
fn instant(text: &str) -> String {
    let n = |a: usize, b: usize| text[a..b].parse::<i64>().unwrap();
    let Some(zone) = text.get(19..).map(|rest| rest.trim_start_matches(|c: char| c == '.' || c.is_ascii_digit())).filter(|z| !z.is_empty()) else { return format!("{text} (local)") };
    let offset = if zone == "Z" { 0 } else { (if zone.starts_with('-') { -1 } else { 1 }) * (n(text.len() - 5, text.len() - 3) * 60 + n(text.len() - 2, text.len())) };
    let (y, m, d) = (n(0, 4), n(5, 7), n(8, 10));
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let days = 365 * y + y / 4 - y / 100 + y / 400 + (153 * m + 2) / 5 + d - 719_469;
    let seconds = days * 86_400 + n(11, 13) * 3600 + n(14, 16) * 60 + n(17, 19) - offset * 60;
    format!("{seconds}.{} UTC", &text[20..27])
}

/// A state record line's outcome against a state of 4,096 bytes written at tick 639,000,000,001,234,567: its members, one `name=value` line each, or `none`.
fn record_outcome(line: &Path) -> String {
    let state = std::env::temp_dir().join(format!("crystal-disk-{}-{}.state", std::process::id(), line.file_stem().unwrap().to_str().unwrap()));
    std::fs::write(&state, vec![0u8; 4096]).unwrap();
    let nanos = (639_000_000_001_234_567 - UNIX_EPOCH_TICKS) as u64 * 100;
    std::fs::File::options().write(true).open(&state).unwrap().set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)).unwrap();
    let record = states::read_packaged(&line, state.to_str().unwrap());
    std::fs::remove_file(&state).unwrap();
    match record {
        None => "none\n".to_string(),
        Some(r) => format!(
            "console={}\ncore={}\nstate_version={}\nbuild={}\nsaved_at={}\nrom_file={}\nrom_md5={}\nrom_bytes={}\n",
            r.console,
            r.core,
            r.state_version,
            r.build,
            instant(&r.saved_at),
            r.rom_file,
            r.rom_md5.as_deref().unwrap_or("(null)"),
            r.rom_bytes
        ),
    }
}

#[test]
fn every_package_state_record_reads_to_its_recorded_members() {
    for line in files("states", "line") {
        assert_eq!(record_outcome(&line), expected(&line, "record"), "{}", line.display());
    }
}

#[test]
fn a_state_record_written_reads_back_to_the_same_members() {
    let line = Path::new(DISK).join("states/written.line");
    let state = std::env::temp_dir().join(format!("crystal-disk-again-{}.state", std::process::id()));
    std::fs::write(&state, vec![0u8; 4096]).unwrap();
    let r = states::read_packaged(&line, state.to_str().unwrap()).or_else(|| {
        let nanos = (639_000_000_001_234_567 - UNIX_EPOCH_TICKS) as u64 * 100;
        std::fs::File::options().write(true).open(&state).unwrap().set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)).unwrap();
        states::read_packaged(&line, state.to_str().unwrap())
    });
    let r = r.expect("the corpus record binds");
    let again = std::env::temp_dir().join(format!("crystal-disk-again-{}.line", std::process::id()));
    std::fs::write(&again, states::package_record_text(&r, 4096, 639_000_000_001_234_567)).unwrap();
    assert_eq!(states::read_packaged(&again, state.to_str().unwrap()), Some(r));
    let _ = (std::fs::remove_file(&state), std::fs::remove_file(&again));
}

#[test]
fn every_stored_save_time_names_its_recorded_instant() {
    let table = std::fs::read_to_string(Path::new(DISK).join("times.tsv")).unwrap();
    for line in table.lines().filter(|l| !l.starts_with('#')) {
        let [stored, want] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("{line}") };
        assert_eq!(time_outcome(stored), want, "{line}");
    }
}

fn time_outcome(stored: &str) -> String {
    emusen_galaxia::clock::round_trip(stored).map_or("none".to_string(), |t| instant(&t))
}
