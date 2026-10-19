//! `games.db`: opened as Microsoft.Data.Sqlite opens it for the C# `GameRecords`, and brought to this build's schema by
//! the same append-only list of migrations, each in its own transaction; a file newer than the list is refused
//! unwritten. See EmuSen_Settings_Reference.md §4.32 and EmuSen_Backend.md §28.3.

use crate::Error;
use emusen_sqlite::{Bound, Connection, Sqlite};
use std::sync::Arc;

/// Each entry takes the file from the version before it; append, never edit. The texts are the C#'s, to the byte,
/// since SQLite keeps a table's creating text in `sqlite_master`.
pub const MIGRATIONS: [&str; 6] = [
    r#"CREATE TABLE game (
    path          TEXT    PRIMARY KEY,
    favourite     INTEGER NOT NULL DEFAULT 0 CHECK (favourite IN (0, 1)),
    last_played   TEXT,
    play_count    INTEGER NOT NULL DEFAULT 0,
    play_seconds  REAL    NOT NULL DEFAULT 0
);
CREATE INDEX game_by_last_played ON game(last_played);"#,
    r#"ALTER TABLE game ADD COLUMN md5 TEXT;
ALTER TABLE game ADD COLUMN bytes INTEGER;
CREATE INDEX game_by_md5 ON game(md5);
CREATE TABLE file_hash (
    path      TEXT    PRIMARY KEY,
    bytes     INTEGER NOT NULL,
    modified  INTEGER NOT NULL,
    md5       TEXT    NOT NULL
);
CREATE INDEX file_hash_by_md5 ON file_hash(md5);"#,
    r#"CREATE TABLE collection (
    id        INTEGER PRIMARY KEY,
    name      TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    created   TEXT    NOT NULL
);
CREATE TABLE collection_game (
    collection  INTEGER NOT NULL REFERENCES collection(id) ON DELETE CASCADE,
    path        TEXT    NOT NULL,
    PRIMARY KEY (collection, path)
);
CREATE INDEX collection_game_by_path ON collection_game(path);"#,
    r#"CREATE TABLE cover_lookup (
    path      TEXT PRIMARY KEY,
    outcome   TEXT NOT NULL CHECK (outcome IN ('Found', 'Unknown', 'NoArt')),
    rom_name  TEXT,
    checked   TEXT NOT NULL
);"#,
    r#"CREATE TABLE game_edit (
    path    TEXT NOT NULL,
    field   TEXT NOT NULL,
    value   TEXT NOT NULL,
    edited  TEXT NOT NULL,
    PRIMARY KEY (path, field)
);"#,
    r#"ALTER TABLE game ADD COLUMN crop_left REAL;
ALTER TABLE game ADD COLUMN crop_right REAL;
ALTER TABLE game ADD COLUMN crop_top REAL;
ALTER TABLE game ADD COLUMN crop_bottom REAL;"#,
];

/// The schema this build writes.
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

/// A transaction as Microsoft.Data.Sqlite begins one by default: serializable, taking the write lock at once.
pub fn begin(db: &Connection) -> Result<(), Error> {
    db.execute("PRAGMA read_uncommitted = 0;")?;
    db.execute("BEGIN IMMEDIATE;")?;
    Ok(())
}

/// Runs `work` in a transaction, committed when it succeeds and rolled back when it fails, as a disposed
/// `SqliteTransaction` rolls back.
pub fn transaction<T>(db: &Connection, work: impl FnOnce(&Connection) -> Result<T, Error>) -> Result<T, Error> {
    begin(db)?;
    match work(db) {
        Ok(value) => {
            db.execute("COMMIT;")?;
            Ok(value)
        }
        Err(e) => {
            let _ = db.execute("ROLLBACK;");
            Err(e)
        }
    }
}

/// The first column of the first row, as `ExecuteScalar` gives it.
pub fn scalar_i64(db: &Connection, sql: &str, parameters: &[(&str, Bound)]) -> Result<Option<i64>, Error> {
    let mut found = None;
    db.query(sql, parameters, |row| {
        if found.is_none() {
            found = Some(if row.is_null(0) { None } else { Some(row.int64(0)) });
        }
    })?;
    Ok(found.flatten())
}

/// The first column of the first row where it is text, as `ExecuteScalar() as string` gives it.
pub fn scalar_text(db: &Connection, sql: &str, parameters: &[(&str, Bound)]) -> Result<Option<String>, Error> {
    let mut found = None;
    db.query(sql, parameters, |row| {
        if found.is_none() {
            found = Some(if row.kind(0) == emusen_sqlite::Kind::Text { Some(row.text(0)) } else { None });
        }
    })?;
    Ok(found.flatten())
}

/// Opens `games.db`, its folder made first, with foreign keys on, and migrates it.
pub fn open(sqlite: Arc<Sqlite>, path: &str) -> Result<Connection, Error> {
    if let Some(folder) = emusen_galaxia::dotnet_path::directory_name(emusen_galaxia::dotnet_path::Style::HOST, path).filter(|f| !f.is_empty()) {
        std::fs::create_dir_all(&folder).map_err(|e| Error::Io(emusen_dotnet::file::io_message(&e, &folder)))?;
    }
    let mut db = Connection::open(sqlite, path)?;
    db.set_timeout(emusen_sqlite::COMMAND_TIMEOUT);
    db.execute("PRAGMA foreign_keys = ON")?;
    migrate(&db)?;
    Ok(db)
}

fn migrate(db: &Connection) -> Result<(), Error> {
    let mut version = scalar_i64(db, "PRAGMA user_version", &[])?.unwrap_or(0);
    if version > SCHEMA_VERSION {
        return Err(Error::Newer(format!("games.db is schema {version}; this build knows {SCHEMA_VERSION}.")));
    }
    while version < SCHEMA_VERSION {
        transaction(db, |db| {
            db.execute(MIGRATIONS[version as usize])?;
            db.execute(&format!("PRAGMA user_version = {}", version + 1))?;
            Ok(())
        })?;
        version += 1;
    }
    Ok(())
}
