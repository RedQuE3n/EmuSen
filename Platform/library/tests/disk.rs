//! The disk corpus of games.db's times: `last_played` as the platform and the C# before it stored it, each beside the
//! instant it names. A time with no zone is local and is held by the owner's own tests. See EmuSen_RustPlatform.md §47.

use std::path::Path;

#[test]
fn every_stored_play_time_names_its_recorded_instant() {
    let table = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/disk/last-played.tsv")).unwrap();
    for line in table.lines().filter(|l| !l.starts_with('#')) {
        let [stored, ticks] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("{line}") };
        assert_eq!(emusen_library::clock::instant(stored).map_or("none".to_string(), |t| t.to_string()), ticks, "{line}");
    }
}
