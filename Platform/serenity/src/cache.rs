//! Compiled SPIR-V kept in SQLite by a hash of everything a compile reads; a miss, a damaged row or an unusable
//! database compiles instead: the C# `SpirvCache`. See EmuSen_Serenity.md §9.4.
//!
//! The file is the C#'s: one schema, read from the one file both embed, and `user_version` 1, so either opens what
//! the other wrote. The cache is not locked here; a host that shares one between threads holds it in a lock for
//! `find` and `store` and compiles outside it, as the C# does.

use crate::compiler::{Shaderc, Stage};
use crate::sqlite::{self, Bound, Column, Connection, Failure, Sqlite};
use crate::{Error, full};
use emusen_galaxia::dotnet_path::{Style, directory_name};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

pub const FILE_NAME: &str = "spirv-cache.db";
pub const DEFAULT_LIMIT: i64 = 64 << 20;
pub const SCHEMA_VERSION: i64 = 1;
const SPIRV_MAGIC: u32 = 0x0723_0203;
const BUSY_LIMIT: i32 = 3;

/// The one schema, the file `EmuSen.Serenity` embeds.
pub const SCHEMA: &str = include_str!("../../../EmuSen.Serenity/Slang/spirv-cache-schema.sql");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub hits: i32,
    pub misses: i32,
    pub stored: i32,
    pub damaged: i32,
    pub skipped: i32,
    pub evicted: i32,
}

pub struct Cache {
    sqlite: Arc<Sqlite>,
    path: String,
    limit: i64,
    identity: String,
    db: Option<Connection>,
    total: i64,
    busy: i32,
    /// Why the cache is off, or none while it works; builds go on compiling either way.
    pub problem: Option<String>,
    pub counters: Counters,
    /// How old, in seconds, a row's use must be before a hit writes it again.
    pub touch_after: i64,
}

/// A row as the cache reads one: the SPIR-V, its digest, and when it was last used.
type Row = (Vec<u8>, Vec<u8>, i64);

/// What made a statement fail: SQLite, or a row that is not what the schema says.
enum Fault {
    Sqlite(Failure),
    Shape(&'static str),
}

impl From<Failure> for Fault {
    fn from(failure: Failure) -> Fault {
        Fault::Sqlite(failure)
    }
}

/// SHA-256 over each part, length-prefixed, so no two different inputs run together into the same bytes.
pub fn key(identity: &str, text: &str, stage: Stage, name: &str) -> [u8; 32] {
    let mut hash = Sha256::new();
    for part in ["emusen-spirv/1", identity, stage.name(), name, text] {
        hash.update((part.len() as i32).to_le_bytes());
        hash.update(part.as_bytes());
    }
    hash.finalize().into()
}

fn intact(spirv: &[u8], digest: &[u8]) -> bool {
    spirv.len() >= 20 && spirv.len().is_multiple_of(4) && spirv[..4] == SPIRV_MAGIC.to_le_bytes() && Sha256::digest(spirv).as_slice() == digest
}

/// The file moved aside as damaged, with the two files SQLite keeps beside it removed.
fn set_aside(path: &str) -> std::io::Result<()> {
    std::fs::rename(path, format!("{path}.damaged"))?;
    for side in [format!("{path}-wal"), format!("{path}-shm")] {
        if std::path::Path::new(&side).is_file() {
            std::fs::remove_file(side)?;
        }
    }
    Ok(())
}

/// Why the database could not be opened: SQLite's own refusal, with its code, or anything before it.
enum Refused {
    Sqlite(Failure),
    Other(String),
}

impl Refused {
    fn words(self) -> String {
        match self {
            Refused::Sqlite(failure) => failure.message,
            Refused::Other(words) => words,
        }
    }
}

impl From<Failure> for Refused {
    fn from(failure: Failure) -> Refused {
        Refused::Sqlite(failure)
    }
}

fn open_once(sqlite: &Arc<Sqlite>, path: &str) -> Result<Connection, Refused> {
    let whole = full(path).map_err(|error| match error {
        Error::Argument { message, parameter } => Refused::Other(format!("{message} (Parameter '{parameter}')")),
        other => Refused::Other(other.message().to_string()),
    })?;
    if let Some(directory) = directory_name(Style::HOST, &whole).filter(|directory| !directory.is_empty()) {
        std::fs::create_dir_all(&directory).map_err(|error| Refused::Other(format!("{directory}: {error}")))?;
    }
    let db = Connection::open(sqlite.clone(), path)?;
    db.execute("PRAGMA busy_timeout = 250")?;
    db.execute("PRAGMA journal_mode = WAL")?;
    db.execute("PRAGMA synchronous = NORMAL")?;
    let mut version = 0;
    db.run("PRAGMA user_version", &[], |column| {
        if let Column::Integer(found) = column(0, false) {
            version = found;
        }
    })?;
    if version != SCHEMA_VERSION {
        begin(&db)?;
        let replaced = db.execute("DROP TABLE IF EXISTS spirv").and_then(|_| db.execute(SCHEMA)).and_then(|_| db.execute(&format!("PRAGMA user_version = {SCHEMA_VERSION}"))).and_then(|_| db.execute("COMMIT;"));
        if let Err(failure) = replaced {
            let _ = db.execute("ROLLBACK;");
            return Err(failure.into());
        }
    }
    db.execute(SCHEMA)?;
    Ok(db)
}

/// A transaction as Microsoft.Data.Sqlite begins one by default: serializable, taking the write lock at once.
fn begin(db: &Connection) -> Result<(), Failure> {
    db.execute("PRAGMA read_uncommitted = 0;")?;
    db.execute("BEGIN IMMEDIATE;").map(|_| ())
}

/// The rows to let go, least recently used first, deleted in one transaction: the sum left and how many went.
fn evict(db: &Connection, total: i64, target: i64) -> Result<(i64, i32), Fault> {
    begin(db)?;
    let mut doomed: Vec<(Vec<u8>, i64)> = Vec::new();
    let mut left = total;
    let mut shape = None;
    db.run("SELECT key, bytes FROM spirv ORDER BY last_used, key", &[], |column| {
        if left <= target || shape.is_some() {
            return;
        }
        match (column(0, true), column(1, false)) {
            (Column::Blob(key), Column::Integer(bytes)) => {
                doomed.push((key, bytes));
                left = left.wrapping_sub(bytes);
            }
            _ => shape = Some(Fault::Shape("a row's key is not a blob")),
        }
    })?;
    if let Some(fault) = shape {
        return Err(fault);
    }
    let (mut total, mut evicted) = (total, 0);
    for (key, bytes) in &doomed {
        if db.run("DELETE FROM spirv WHERE key = $key", &[("$key", Bound::Blob(key))], |_| {})? > 0 {
            total -= bytes;
            evicted += 1;
        }
    }
    db.execute("COMMIT;")?;
    Ok((total, evicted))
}

impl Cache {
    /// Opens the cache, or leaves it off with why; a file that is not a database is set aside and a new one made.
    pub fn open(sqlite: Arc<Sqlite>, path: &str, limit: i64, identity: &str) -> Cache {
        let mut cache = Cache { sqlite, path: path.to_string(), limit, identity: identity.to_string(), db: None, total: 0, busy: 0, problem: None, counters: Counters::default(), touch_after: 3600 };
        let opened = match open_once(&cache.sqlite, path) {
            // A cache that cannot be read is replaced, not repaired.
            Err(Refused::Sqlite(Failure { code: sqlite::CORRUPT | sqlite::NOT_A_DATABASE, .. })) => match set_aside(path) {
                Ok(()) => open_once(&cache.sqlite, path).map_err(Refused::words),
                Err(error) => Err(format!("{path}: {error}")),
            },
            other => other.map_err(Refused::words),
        };
        match opened {
            Ok(db) => {
                cache.db = Some(db);
                cache.total = cache.total_or_zero();
            }
            Err(words) => cache.problem = Some(format!("{FILE_NAME}: {}", words.split('\n').next().unwrap_or_default())),
        }
        cache
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn working(&self) -> bool {
        self.db.is_some()
    }

    /// The row for a key when it is there and intact; a damaged one is deleted. `now` is Unix seconds.
    pub fn find(&mut self, key: &[u8; 32], now: i64) -> Option<Vec<u8>> {
        let db = self.db.as_ref()?;
        let mut found: Option<Result<Row, Fault>> = None;
        let read = db.run("SELECT spirv, digest, last_used FROM spirv WHERE key = $key", &[("$key", Bound::Blob(key))], |column| {
            if found.is_some() {
                return;
            }
            found = Some(match (column(0, true), column(1, true), column(2, false)) {
                (Column::Blob(spirv), Column::Blob(digest), Column::Integer(last_used)) => Ok((spirv, digest, last_used)),
                _ => Err(Fault::Shape("a row's SPIR-V or digest is not a blob")),
            });
        });
        let (spirv, digest, last_used) = match (read, found) {
            (Err(failure), _) => {
                self.failed(failure.into());
                return None;
            }
            (Ok(_), Some(Err(fault))) => {
                self.failed(fault);
                return None;
            }
            (Ok(_), None) => {
                self.counters.misses += 1;
                return None;
            }
            (Ok(_), Some(Ok(row))) => row,
        };

        if !intact(&spirv, &digest) {
            self.counters.damaged += 1;
            self.counters.misses += 1;
            self.execute("DELETE FROM spirv WHERE key = $key", &[("$key", Bound::Blob(key))]);
            return None;
        }

        self.counters.hits += 1;
        if now.wrapping_sub(last_used) >= self.touch_after {
            self.execute("UPDATE spirv SET last_used = $now WHERE key = $key", &[("$now", Bound::Integer(now)), ("$key", Bound::Blob(key))]);
        }
        Some(spirv)
    }

    /// Keeps a stage's SPIR-V under its key, and lets the least recently used rows go when the bound is passed.
    pub fn store(&mut self, key: &[u8; 32], spirv: &[u8], now: i64) {
        if self.db.is_none() {
            return;
        }
        let digest = Sha256::digest(spirv);
        let added = self.execute(
            "INSERT OR IGNORE INTO spirv (key, bytes, last_used, digest, spirv) VALUES ($key, $bytes, $now, $digest, $spirv)",
            &[("$key", Bound::Blob(key)), ("$bytes", Bound::Integer(spirv.len() as i64)), ("$now", Bound::Integer(now)), ("$digest", Bound::Blob(digest.as_slice())), ("$spirv", Bound::Blob(spirv))],
        );
        if added <= 0 {
            return;
        }
        self.counters.stored += 1;
        self.total += spirv.len() as i64;
        if self.total > self.limit {
            self.trim();
        }
    }

    /// The least recently used rows go until the sum is three quarters of the bound, since other processes may have added to it too.
    fn trim(&mut self) {
        self.total = self.total_or_zero();
        let Some(db) = self.db.as_ref() else { return };
        if self.total <= self.limit {
            return;
        }
        match evict(db, self.total, self.limit / 4 * 3) {
            Ok((total, evicted)) => {
                self.total = total;
                self.counters.evicted += evicted;
            }
            Err(fault) => {
                let _ = db.execute("ROLLBACK;");
                self.failed(fault);
            }
        }
    }

    fn total_or_zero(&mut self) -> i64 {
        let Some(db) = self.db.as_ref() else { return 0 };
        let mut total = 0;
        match db.run("SELECT COALESCE(SUM(bytes), 0) FROM spirv", &[], |column| {
            if let Column::Integer(sum) = column(0, false) {
                total = sum;
            }
        }) {
            Ok(_) => total,
            Err(failure) => {
                self.failed(failure.into());
                0
            }
        }
    }

    /// Rows changed, or -1 when the statement failed and was let go.
    fn execute(&mut self, sql: &str, parameters: &[(&str, Bound)]) -> i32 {
        let Some(db) = self.db.as_ref() else { return -1 };
        match db.run(sql, parameters, |_| {}) {
            Ok(changed) => changed,
            Err(failure) => {
                self.failed(failure.into());
                -1
            }
        }
    }

    /// Busy or locked by another connection is let go for this statement, and the third time turns the cache off, as
    /// anything else does at once; a damaged file is set aside for the next process.
    fn failed(&mut self, fault: Fault) {
        let (code, words) = match fault {
            Fault::Sqlite(failure) => (failure.code, failure.message),
            Fault::Shape(words) => (0, words.to_string()),
        };
        if matches!(code, sqlite::BUSY | sqlite::LOCKED) {
            self.busy += 1;
            if self.busy < BUSY_LIMIT {
                self.counters.skipped += 1;
                return;
            }
        }
        self.problem = Some(format!("{FILE_NAME}: {}", words.split('\n').next().unwrap_or_default()));
        self.db = None;
        if matches!(code, sqlite::CORRUPT | sqlite::NOT_A_DATABASE) {
            let _ = set_aside(&self.path);
        }
    }

    /// Lets go of the database; the cache then compiles everything and keeps nothing.
    pub fn close(&mut self) {
        self.db = None;
    }
}

/// The stage's SPIR-V, from the cache or compiled and kept; the cache is held only to look and to keep, so stages
/// compile side by side. A stage that does not compile is not kept, and fails with the compiler's words.
pub fn compile(cache: &Mutex<Cache>, shaderc: &Shaderc, text: &str, stage: Stage, name: &str, now: i64) -> Result<Vec<u8>, String> {
    let lock = || cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let key = key(lock().identity(), text, stage, name);
    if let Some(found) = lock().find(&key, now) {
        return Ok(found);
    }
    let spirv = shaderc.compile(text, stage, name)?;
    lock().store(&key, &spirv, now);
    Ok(spirv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_input_to_a_compile_is_in_its_key_and_none_runs_into_the_next() {
        let basis = key("shaderc x", "void main() {}", Stage::Vertex, "a.slang");
        assert_eq!(basis, key("shaderc x", "void main() {}", Stage::Vertex, "a.slang"));
        assert_ne!(basis, key("shaderc y", "void main() {}", Stage::Vertex, "a.slang"));
        assert_ne!(basis, key("shaderc x", "void main() { }", Stage::Vertex, "a.slang"));
        assert_ne!(basis, key("shaderc x", "void main() {}", Stage::Fragment, "a.slang"));
        assert_ne!(basis, key("shaderc x", "void main() {}", Stage::Vertex, "b.slang"));
        assert_ne!(key("i", "c", Stage::Vertex, "ab"), key("i", "bc", Stage::Vertex, "a"));
    }

    #[test]
    fn a_row_is_intact_only_as_whole_spirv_words_with_their_own_digest() {
        let mut spirv = SPIRV_MAGIC.to_le_bytes().to_vec();
        spirv.extend([0u8; 16]);
        let digest = Sha256::digest(&spirv);
        assert!(intact(&spirv, &digest));
        assert!(!intact(&spirv[..16], &Sha256::digest(&spirv[..16])));
        assert!(!intact(&[spirv.as_slice(), &[0]].concat(), &digest));
        let mut flipped = spirv.clone();
        flipped[8] ^= 1;
        assert!(!intact(&flipped, &digest));
        flipped[8] ^= 1;
        flipped[0] ^= 1;
        assert!(!intact(&flipped, &Sha256::digest(&flipped)));
    }

    #[test]
    fn the_schema_is_the_one_file_and_names_the_table() {
        assert!(SCHEMA.contains("CREATE TABLE IF NOT EXISTS spirv"));
        assert!(SCHEMA.contains("CREATE INDEX IF NOT EXISTS spirv_by_use"));
        assert_eq!((DEFAULT_LIMIT, SCHEMA_VERSION), (67_108_864, 1));
    }
}
