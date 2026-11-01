//! A read-only connection, as OpenVGDB is opened (EmuSen_Backend.md §30.6), over the e_sqlite3 the program ships: from
//! `EMUSEN_SQLITE`, else the NuGet cache's copy; with neither, the test fails unless EMUSEN_SQLITE_SKIP names a reason.

use emusen_sqlite::{Connection, Sqlite};
use std::sync::Arc;

fn sqlite() -> Option<Arc<Sqlite>> {
    emusen_sqlite::for_tests()
}

#[test]
fn a_read_only_connection_reads_refuses_a_write_and_makes_no_file() {
    let Some(sqlite) = sqlite() else {
        return;
    };
    let dir = std::env::temp_dir().join(format!("emusen-sqlite-ro-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db.sqlite").to_string_lossy().into_owned();

    let missing = Connection::open_read_only(sqlite.clone(), &path).err().expect("a missing file is refused");
    assert_eq!(missing.code, 14);
    assert!(!std::path::Path::new(&path).exists());

    let writer = Connection::open(sqlite.clone(), &path).unwrap();
    writer.execute("CREATE TABLE t (v INTEGER); INSERT INTO t VALUES (7);").unwrap();
    drop(writer);

    let reader = Connection::open_read_only(sqlite, &path).unwrap();
    let mut seen = Vec::new();
    reader.query("SELECT v FROM t", &[], |row| seen.push(row.int64(0))).unwrap();
    assert_eq!(seen, [7]);
    let refused = reader.execute("INSERT INTO t VALUES (8)").err().expect("a write is refused");
    assert_eq!(refused.code, 8, "{}", refused.message);
    drop(reader);
    let _ = std::fs::remove_dir_all(&dir);
}
