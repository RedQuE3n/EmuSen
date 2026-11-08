//! `states.db`, the C# `StateRecords`: the runtime's record of each save state it wrote, keyed by the state's full path
//! and stamped with its size and time, the rows of Mistress's `records.db` copied in once (D8), and the sidecar JSON
//! builds before 2026-09-26 wrote beside a state read where no row is (EmuSen_Backend.md §14.12, §29.5).

use std::path::Path;
use emusen_galaxia::path_key;
use std::sync::Arc;

use emusen_sqlite::{Bound, Connection, Sqlite};

pub const FILE_NAME: &str = emusen_galaxia::stores::STATES_DB;
pub const RECORDS_FILE_NAME: &str = emusen_galaxia::stores::RECORDS_DB;

const MIGRATIONS: [&str; 1] = ["CREATE TABLE state_record (
    path            TEXT    PRIMARY KEY,
    state_bytes     INTEGER NOT NULL,
    state_modified  INTEGER NOT NULL,
    console         TEXT    NOT NULL,
    core            TEXT    NOT NULL,
    state_version   INTEGER NOT NULL,
    build           TEXT    NOT NULL,
    saved_at        TEXT    NOT NULL,
    rom_file        TEXT    NOT NULL,
    rom_md5         TEXT,
    rom_bytes       INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE copied (source TEXT PRIMARY KEY, at TEXT NOT NULL);"];

/// .NET's ticks from 0001-01-01 at 1970-01-01.
const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;

/// Who wrote a save state, `StateRecord`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRecord {
    pub console: String,
    pub core: String,
    pub state_version: i32,
    pub build: String,
    /// The round-trip text of when it was saved, as the C# writes `SavedAt.ToString("o")`.
    pub saved_at: String,
    pub rom_file: String,
    pub rom_md5: Option<String>,
    pub rom_bytes: i64,
}

/// A refusal in the words the C#'s exceptions reach the API with.
pub type Refused = (i32, String);

fn sqlite_failure(f: emusen_sqlite::Failure) -> Refused {
    (emusen_session_service::abi::status::FAULT, format!("SqliteException: {}", f.message))
}

/// A state file's size and its last write in .NET's ticks, or none where it cannot be read.
fn stat(at: &str) -> Option<(i64, i64)> {
    let m = std::fs::metadata(at).ok().filter(|m| m.is_file())?;
    let ticks = emusen_galaxia::clock::ticks_of(m.modified().ok()?) + UNIX_EPOCH_TICKS;
    Some((m.len() as i64, ticks))
}

fn scalar(db: &Connection, sql: &str, parameters: &[(&str, Bound)]) -> Result<Option<i64>, Refused> {
    let mut found = None;
    db.query(sql, parameters, |row| {
        if found.is_none() {
            found = Some(if row.is_null(0) { None } else { Some(row.int64(0)) });
        }
    })
    .map_err(sqlite_failure)?;
    Ok(found.flatten())
}

fn run(db: &Connection, sql: &str, parameters: &[(&str, Bound)]) -> Result<(), Refused> {
    db.query(sql, parameters, |_| {}).map(|_| ()).map_err(sqlite_failure)
}

/// Work in a transaction as Microsoft.Data.Sqlite begins one, rolled back where it fails.
fn transaction(db: &Connection, work: impl FnOnce(&Connection) -> Result<(), Refused>) -> Result<(), Refused> {
    db.execute("PRAGMA read_uncommitted = 0;").map_err(sqlite_failure)?;
    db.execute("BEGIN IMMEDIATE;").map_err(sqlite_failure)?;
    match work(db) {
        Ok(()) => db.execute("COMMIT;").map(|_| ()).map_err(sqlite_failure),
        Err(e) => {
            let _ = db.execute("ROLLBACK;");
            Err(e)
        }
    }
}

/// A row of `SELECT state_bytes, state_modified, console, ... rom_bytes` as its record.
fn record_of(r: &emusen_sqlite::Row) -> StateRecord {
    StateRecord {
        console: r.text(2),
        core: r.text(3),
        state_version: r.int64(4) as i32,
        build: r.text(5),
        saved_at: emusen_galaxia::clock::round_trip(&r.text(6)).unwrap_or_else(|| r.text(6)),
        rom_file: r.text(7),
        rom_md5: r.text_or_null(8),
        rom_bytes: r.int64(9),
    }
}

pub struct StateRecords {
    db: Connection,
}

impl StateRecords {
    /// The file opened, made and migrated, and records.db's rows copied once.
    pub fn open(sqlite: Arc<Sqlite>, at: &str) -> Result<StateRecords, Refused> {
        let full = path_key::key_text(at);
        let folder = Path::new(&full).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
        std::fs::create_dir_all(&folder).map_err(|e| (crate::abi::status::IO, format!("{}: {}", folder, e)))?;
        let mut db = Connection::open(sqlite, at).map_err(sqlite_failure)?;
        db.set_timeout(emusen_sqlite::COMMAND_TIMEOUT);
        let mut version = scalar(&db, "PRAGMA user_version", &[])?.unwrap_or(0);
        if version > MIGRATIONS.len() as i64 {
            return Err((emusen_session_service::abi::status::FAULT, format!("InvalidDataException: {FILE_NAME} is schema {version}; this build knows {}.", MIGRATIONS.len())));
        }
        while version < MIGRATIONS.len() as i64 {
            transaction(&db, |db| {
                db.execute(MIGRATIONS[version as usize]).map_err(sqlite_failure)?;
                db.execute(&format!("PRAGMA user_version = {}", version + 1)).map_err(sqlite_failure)?;
                Ok(())
            })?;
            version += 1;
        }
        db.execute("PRAGMA journal_mode = WAL").map_err(sqlite_failure)?;
        copy_from_records(&db, &Path::new(&folder).join(RECORDS_FILE_NAME))?;
        Ok(StateRecords { db })
    }

    /// The record while the state is the file it was written for; with no row, the sidecar an earlier build wrote.
    pub fn read(&self, state_path: &str) -> Result<Option<StateRecord>, Refused> {
        let key = path_key::key_text(state_path);
        let Some((bytes, modified)) = stat(&key) else { return Ok(None) };
        let mut row: Option<(i64, i64, StateRecord)> = None;
        self.db
            .query(
                "SELECT state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM state_record WHERE path = $path",
                &[("$path", Bound::Text(&key))],
                |r| {
                    if row.is_none() {
                        row = Some((r.int64(0), r.int64(1), record_of(r)));
                    }
                },
            )
            .map_err(sqlite_failure)?;
        Ok(match row {
            Some((b, m, record)) => (b == bytes && m == modified).then_some(record),
            None => read_sidecar(&key),
        })
    }

    pub fn write(&self, state_path: &str, r: &StateRecord) -> Result<(), Refused> {
        let key = path_key::key_text(state_path);
        let Some((bytes, modified)) = stat(&key) else { return Ok(()) };
        run(
            &self.db,
            "INSERT INTO state_record (path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes)
VALUES ($path, $bytes, $modified, $console, $core, $version, $build, $saved, $rom, $md5, $romBytes)
ON CONFLICT(path) DO UPDATE SET state_bytes = excluded.state_bytes, state_modified = excluded.state_modified, console = excluded.console,
core = excluded.core, state_version = excluded.state_version, build = excluded.build, saved_at = excluded.saved_at, rom_file = excluded.rom_file,
rom_md5 = excluded.rom_md5, rom_bytes = excluded.rom_bytes",
            &[
                ("$path", Bound::Text(&key)),
                ("$bytes", Bound::Integer(bytes)),
                ("$modified", Bound::Integer(modified)),
                ("$console", Bound::Text(&r.console)),
                ("$core", Bound::Text(&r.core)),
                ("$version", Bound::Integer(r.state_version as i64)),
                ("$build", Bound::Text(&r.build)),
                ("$saved", Bound::Text(&r.saved_at)),
                ("$rom", Bound::Text(&r.rom_file)),
                ("$md5", r.rom_md5.as_deref().map_or(Bound::Null, Bound::Text)),
                ("$romBytes", Bound::Integer(r.rom_bytes)),
            ],
        )
    }

    pub fn forget(&self, state_path: &str) -> Result<(), Refused> {
        run(&self.db, "DELETE FROM state_record WHERE path = $path", &[("$path", Bound::Text(&path_key::key_text(state_path)))])
    }
}

/// D8's one migration: every row of records.db's state_record copied, none over a row already here, once.
fn copy_from_records(db: &Connection, records: &Path) -> Result<(), Refused> {
    let source = Bound::Text(RECORDS_FILE_NAME);
    if scalar(db, "SELECT COUNT(*) FROM copied WHERE source = $source", &[("$source", Bound::Text(RECORDS_FILE_NAME))])?.unwrap_or(0) > 0 {
        return Ok(());
    }
    let at = emusen_galaxia::clock::utc_now();
    if emusen_galaxia::files::file_exists(records) {
        run(db, "ATTACH DATABASE $file AS records", &[("$file", Bound::Text(&records.to_string_lossy()))])?;
        let copied = transaction(db, |db| {
            if scalar(db, "SELECT COUNT(*) FROM records.sqlite_master WHERE type = 'table' AND name = 'state_record'", &[])?.unwrap_or(0) > 0 {
                db.execute(
                    "INSERT OR IGNORE INTO state_record (path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes)
SELECT path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM records.state_record",
                )
                .map_err(sqlite_failure)?;
            }
            run(db, "INSERT INTO copied (source, at) VALUES ($source, $at)", &[("$source", Bound::Text(RECORDS_FILE_NAME)), ("$at", Bound::Text(&at))])
        });
        let detached = db.execute("DETACH DATABASE records").map(|_| ()).map_err(sqlite_failure);
        return copied.and(detached);
    }
    run(db, "INSERT INTO copied (source, at) VALUES ($source, $at)", &[("$source", source), ("$at", Bound::Text(&at))])
}

/// A state's record as a package keeps it, and as an earlier build's sidecar held it: each member by name in any case,
/// the four texts required and nullable, the state's size and time only beside a packaged state (EmuSen_GamePackages.md §18.1).
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct KeptRecord {
    #[serde(deserialize_with = "Option::deserialize")]
    console: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    core: Option<String>,
    state_version: i32,
    #[serde(deserialize_with = "Option::deserialize")]
    build: Option<String>,
    saved_at: String,
    #[serde(deserialize_with = "Option::deserialize")]
    rom_file: Option<String>,
    #[serde(default)]
    rom_md5: Option<String>,
    #[serde(default)]
    rom_bytes: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state_modified: Option<i64>,
}

/// A package's record of one of its states: the record's members and the state's size and time, one line of JSON
/// beside the state, so that a package moved or renamed keeps it (EmuSen_GamePackages.md §18.1).
pub fn package_record_text(r: &StateRecord, state_bytes: i64, state_modified: i64) -> String {
    let kept = KeptRecord {
        console: Some(r.console.clone()),
        core: Some(r.core.clone()),
        state_version: r.state_version,
        build: Some(r.build.clone()),
        saved_at: r.saved_at.clone(),
        rom_file: Some(r.rom_file.clone()),
        rom_md5: r.rom_md5.clone(),
        rom_bytes: r.rom_bytes,
        state_bytes: Some(state_bytes),
        state_modified: Some(state_modified),
    };
    serde_json::to_string(&kept).expect("a record is JSON") + "\n"
}

fn write_packaged(record_at: &Path, state_path: &str, r: &StateRecord) -> Result<(), Refused> {
    let Some((bytes, modified)) = stat(&path_key::key_text(state_path)) else { return Ok(()) };
    emusen_galaxia::atomic::write(record_at, package_record_text(r, bytes, modified).as_bytes()).map_err(|words| (crate::abi::status::IO, words))
}

/// A package's record while its state is the file it was written for; none where it is not, or does not bind.
pub fn read_packaged(record_at: &Path, state_path: &str) -> Option<StateRecord> {
    let (bytes, modified) = stat(&path_key::key_text(state_path))?;
    let kept = parse_file(record_at)?;
    (kept.state_bytes == Some(bytes) && kept.state_modified == Some(modified)).then(|| bind(kept)).flatten()
}

/// A state's record wherever it is kept: a packaged state's in its package, read first, then states.db's.
pub fn read(records: Option<&StateRecords>, state_path: &str) -> Result<Option<StateRecord>, Refused> {
    if let Some(at) = emusen_galaxia::saves::package_record_path(state_path)
        && emusen_galaxia::files::file_exists(&at)
    {
        return Ok(read_packaged(&at, state_path));
    }
    records.map_or(Ok(None), |r| r.read(state_path))
}

/// A state's record kept: a packaged state's in its package, any other's in states.db.
pub fn write(records: Option<&StateRecords>, state_path: &str, r: &StateRecord) -> Result<(), Refused> {
    match emusen_galaxia::saves::package_record_path(state_path) {
        Some(at) => write_packaged(&at, state_path, r),
        None => records.map_or(Ok(()), |db| db.write(state_path, r)),
    }
}

/// A state's record forgotten, from its package and from states.db.
pub fn forget(records: Option<&StateRecords>, state_path: &str) -> Result<(), Refused> {
    if let Some(at) = emusen_galaxia::saves::package_record_path(state_path) {
        match std::fs::remove_file(&at) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err((crate::abi::status::IO, format!("{}: {}", at.display(), e))),
        }
    }
    records.map_or(Ok(()), |r| r.forget(state_path))
}

/// The record states.db, else the records.db it was copied from, holds for a state while it is the file it was
/// written for, both read only and neither written: `pkg migrate`'s source (EmuSen_GamePackages.md §18.4).
pub fn kept_record(sqlite: Arc<Sqlite>, library_folder: &str, state_path: &str) -> Option<StateRecord> {
    let key = path_key::key_text(state_path);
    let (bytes, modified) = stat(&key)?;
    for file in [FILE_NAME, RECORDS_FILE_NAME] {
        let at = Path::new(library_folder).join(file);
        if !emusen_galaxia::files::file_exists(&at) {
            continue;
        }
        let Ok(db) = Connection::open_read_only(sqlite.clone(), &at.to_string_lossy()) else { continue };
        let mut found: Option<StateRecord> = None;
        let _ = db.query(
            "SELECT state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM state_record WHERE path = $path",
            &[("$path", Bound::Text(&key))],
            |r| {
                if found.is_none() && r.int64(0) == bytes && r.int64(1) == modified {
                    found = Some(record_of(r));
                }
            },
        );
        if found.is_some() {
            return found;
        }
    }
    read_sidecar(&key)
}

/// A record copied into a package for a state copied there: written beside the copy with the copy's size and time.
pub fn write_package_copy(copy: &str, r: &StateRecord) -> Result<(), String> {
    let at = emusen_galaxia::saves::package_record_path(copy).ok_or_else(|| format!("{copy} is not in a package's states folder"))?;
    write_packaged(&at, copy, r).map_err(|(_, words)| words)
}

/// The sidecar an earlier build wrote, `StateRecord.ReadSidecar`: none for no sidecar or one that does not bind.
pub fn read_sidecar(state_path: &str) -> Option<StateRecord> {
    bind(parse_file(&Path::new(state_path).with_extension("json"))?)
}

fn parse_file(at: &Path) -> Option<KeptRecord> {
    let text = std::fs::read_to_string(at).ok()?;
    emusen_galaxia::config_json::read(text.trim_start_matches('\u{feff}')).ok()
}

/// A kept record as `StateRecord`; none where its time is not one a record holds.
fn bind(kept: KeptRecord) -> Option<StateRecord> {
    Some(StateRecord {
        console: kept.console.unwrap_or_default(),
        core: kept.core.unwrap_or_default(),
        state_version: kept.state_version,
        build: kept.build.unwrap_or_default(),
        saved_at: emusen_galaxia::clock::round_trip(&kept.saved_at)?,
        rom_file: kept.rom_file.unwrap_or_default(),
        rom_md5: kept.rom_md5,
        rom_bytes: kept.rom_bytes,
    })
}
