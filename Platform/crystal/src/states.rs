//! `states.db`, the C# `StateRecords`: the runtime's record of each save state it wrote, keyed by the state's full path
//! and stamped with its size and time, the rows of Mistress's `records.db` copied in once (D8), and the sidecar JSON
//! builds before 2026-09-26 wrote beside a state read where no row is (EmuSen_Backend.md §14.12, §29.5).

use std::sync::Arc;

use emusen_dotnet::json::Value;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_sqlite::{Bound, Connection, Sqlite};

pub const FILE_NAME: &str = "states.db";
pub const RECORDS_FILE_NAME: &str = "records.db";

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

pub struct StateRecords {
    db: Connection,
}

impl StateRecords {
    /// The file opened, made and migrated, and records.db's rows copied once.
    pub fn open(sqlite: Arc<Sqlite>, at: &str) -> Result<StateRecords, Refused> {
        let full = path::full_path(at);
        let folder = path::directory_name(Style::HOST, &full).unwrap_or_default();
        std::fs::create_dir_all(&folder).map_err(|e| (crate::abi::status::IO, emusen_dotnet::file::io_message(&e, &folder)))?;
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
        copy_from_records(&db, &path::combine(Style::HOST, &folder, RECORDS_FILE_NAME))?;
        Ok(StateRecords { db })
    }

    /// The record while the state is the file it was written for; with no row, the sidecar an earlier build wrote.
    pub fn read(&self, state_path: &str) -> Result<Option<StateRecord>, Refused> {
        let key = path::full_path(state_path);
        let Some((bytes, modified)) = stat(&key) else { return Ok(None) };
        let mut row: Option<(i64, i64, StateRecord)> = None;
        self.db
            .query(
                "SELECT state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM state_record WHERE path = $path",
                &[("$path", Bound::Text(&key))],
                |r| {
                    if row.is_none() {
                        let record = StateRecord {
                            console: r.text(2),
                            core: r.text(3),
                            state_version: r.int64(4) as i32,
                            build: r.text(5),
                            saved_at: emusen_galaxia::clock::round_trip(&r.text(6)).unwrap_or_else(|| r.text(6)),
                            rom_file: r.text(7),
                            rom_md5: r.text_or_null(8),
                            rom_bytes: r.int64(9),
                        };
                        row = Some((r.int64(0), r.int64(1), record));
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
        let key = path::full_path(state_path);
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
        run(&self.db, "DELETE FROM state_record WHERE path = $path", &[("$path", Bound::Text(&path::full_path(state_path)))])
    }
}

/// D8's one migration: every row of records.db's state_record copied, none over a row already here, once.
fn copy_from_records(db: &Connection, records: &str) -> Result<(), Refused> {
    let source = Bound::Text(RECORDS_FILE_NAME);
    if scalar(db, "SELECT COUNT(*) FROM copied WHERE source = $source", &[("$source", Bound::Text(RECORDS_FILE_NAME))])?.unwrap_or(0) > 0 {
        return Ok(());
    }
    let at = emusen_galaxia::clock::utc_now();
    if emusen_galaxia::dotnet_files::file_exists(records) {
        run(db, "ATTACH DATABASE $file AS records", &[("$file", Bound::Text(records))])?;
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

/// A member by its name without regard to case, the last of several, as `PropertyNameCaseInsensitive` binds it.
fn member<'a>(v: &'a Value, name: &str) -> Option<&'a Value> {
    match v {
        Value::Object(members) => members.iter().rev().find(|(k, _)| k.text.eq_ignore_ascii_case(name)).map(|(_, v)| v),
        _ => None,
    }
}

/// The sidecar an earlier build wrote, `StateRecord.ReadSidecar`: none for no sidecar or one that does not bind.
pub fn read_sidecar(state_path: &str) -> Option<StateRecord> {
    let at = path::change_extension(Style::HOST, state_path, ".json");
    let text = std::fs::read_to_string(&at).ok()?;
    let doc = emusen_dotnet::json::parse(text.trim_start_matches('\u{feff}')).ok()?;
    let text_of = |name: &str| -> Option<Option<String>> {
        match member(&doc, name)? {
            Value::String(t) => Some(Some(t.text.clone())),
            Value::Null => Some(None),
            _ => None,
        }
    };
    let required = |name: &str| -> Option<String> { text_of(name).map(Option::unwrap_or_default) };
    let number = |name: &str| -> Option<Option<&str>> {
        match member(&doc, name) {
            None => Some(None),
            Some(Value::Number(t)) => Some(Some(t.as_str())),
            Some(_) => None,
        }
    };
    let state_version: i32 = number("StateVersion")??.parse().ok()?;
    let rom_bytes: i64 = match number("RomBytes")? {
        None => 0,
        Some(t) => t.parse().ok()?,
    };
    let saved_at = match member(&doc, "SavedAt")? {
        Value::String(t) => emusen_galaxia::clock::round_trip(&t.text)?,
        _ => return None,
    };
    let rom_md5 = match member(&doc, "RomMd5") {
        None => None,
        Some(_) => text_of("RomMd5")?,
    };
    Some(StateRecord {
        console: required("Console")?,
        core: required("Core")?,
        state_version,
        build: required("Build")?,
        saved_at,
        rom_file: required("RomFile")?,
        rom_md5,
        rom_bytes,
    })
}
