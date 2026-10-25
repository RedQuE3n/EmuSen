//! `emusen-library`: the player's library below every frontend, a component of its own beside Crystal and never in it
//! (EmuSen_Backend.md D3). A handle is one `games.db`; it scans the roots it is given, routing each file among the
//! systems it is told the build opens, answers a dump's identity, and keeps the player's records. Requests and answers
//! are JSON, as `emusen_library.h` describes them; this crate decides, the platform library's C layer only converts.
//! See EmuSen_Backend.md §2.4, §3.7 and §28.

pub mod abi;
pub mod clock;
pub mod db;
mod identity;
mod json;
pub mod records;
pub mod scan;

pub use identity::Identity;

use emusen_sqlite::{Connection, Failure, Sqlite};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Why a call failed; each is one of the header's statuses, with its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Newer(String),
    Database(Failure),
    BadRequest(String),
    Cancelled,
    Busy,
    Io(String),
}

impl Error {
    pub fn status(&self) -> i32 {
        match self {
            Error::Newer(_) => abi::NEWER_FILE,
            Error::Database(_) => abi::DATABASE,
            Error::BadRequest(_) => abi::BAD_REQUEST,
            Error::Cancelled => abi::CANCELLED,
            Error::Busy => abi::BUSY,
            Error::Io(_) => abi::IO,
        }
    }

    pub fn words(&self) -> String {
        match self {
            Error::Newer(w) | Error::BadRequest(w) | Error::Io(w) => w.clone(),
            Error::Database(f) => f.message.clone(),
            Error::Cancelled => "The scan was cancelled.".to_string(),
            Error::Busy => "A scan or a reclaim is already running on this library.".to_string(),
        }
    }
}

impl From<Failure> for Error {
    fn from(failure: Failure) -> Self {
        Error::Database(failure)
    }
}

/// What a running scan or reclaim has done, read from any thread without waiting.
#[derive(Default)]
pub struct Progress {
    running: AtomicBool,
    cancel: AtomicBool,
    /// 0 none, 1 the walk, 2 routing, 3 hashing.
    pub(crate) phase: AtomicU8,
    pub(crate) files: AtomicUsize,
    pub(crate) games: AtomicUsize,
    pub(crate) hashed: AtomicUsize,
    pub(crate) to_hash: AtomicUsize,
}

impl Progress {
    pub(crate) fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    fn start(&self) -> Result<(), Error> {
        if self.running.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return Err(Error::Busy);
        }
        self.cancel.store(false, Ordering::Relaxed);
        for counter in [&self.files, &self.games, &self.hashed, &self.to_hash] {
            counter.store(0, Ordering::Relaxed);
        }
        Ok(())
    }

    fn finish(&self) {
        self.phase.store(0, Ordering::Relaxed);
        self.running.store(false, Ordering::Release);
    }

    /// The progress as `emusen_library_progress` answers it.
    pub fn to_json(&self) -> String {
        let phase = match self.phase.load(Ordering::Relaxed) {
            1 => "walk",
            2 => "route",
            3 => "hash",
            _ => "",
        };
        format!(
            "{{\"running\":{},\"phase\":\"{phase}\",\"files\":{},\"games\":{},\"hashed\":{},\"to_hash\":{}}}",
            self.running.load(Ordering::Relaxed),
            self.files.load(Ordering::Relaxed),
            self.games.load(Ordering::Relaxed),
            self.hashed.load(Ordering::Relaxed),
            self.to_hash.load(Ordering::Relaxed)
        )
    }
}

/// One `games.db`, its last scan and what a running call has done.
pub struct Library {
    db: Mutex<Connection>,
    last: Mutex<Option<Arc<scan::Scan>>>,
    pub progress: Progress,
}

impl Library {
    /// Opens `games.db` with the SQLite given, migrating it to this build's schema.
    pub fn open(sqlite: Arc<Sqlite>, path: &str) -> Result<Library, Error> {
        Ok(Library { db: Mutex::new(db::open(sqlite, path)?), last: Mutex::new(None), progress: Progress::default() })
    }

    pub(crate) fn db(&self) -> Result<MutexGuard<'_, Connection>, Error> {
        Ok(self.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    pub(crate) fn last_scan(&self) -> Option<Arc<scan::Scan>> {
        self.last.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Scans the roots for the systems named, keeping the result; how many games it found.
    pub fn scan(&self, roots: &[String], systems: &[String]) -> Result<usize, Error> {
        self.scan_with(roots, &scan::Beside::default(), systems)
    }

    /// The scan with `Packages` and `Compressed` beside the roots (1.2).
    pub fn scan_with(&self, roots: &[String], beside: &scan::Beside, systems: &[String]) -> Result<usize, Error> {
        self.progress.start()?;
        let result = scan::scan_with(roots, beside, systems, &self.progress);
        self.progress.finish();
        let found = result?;
        let count = found.games.len();
        *self.last.lock().unwrap_or_else(|p| p.into_inner()) = Some(Arc::new(found));
        Ok(count)
    }

    /// Stops a running scan or reclaim at its next file.
    pub fn cancel(&self) {
        self.progress.cancel.store(true, Ordering::Relaxed);
    }

    /// The last scan, as `scan::narrow` leaves it for a system, or an empty one with no directory before any.
    pub fn games(&self, system: Option<&str>) -> (scan::Outcome, Vec<String>, Vec<scan::Game>) {
        match self.last_scan() {
            None => (scan::Outcome::NoDirectory, Vec::new(), Vec::new()),
            Some(found) => {
                let (outcome, games) = scan::narrow(&found, system);
                (outcome, found.roots.clone(), games.into_iter().cloned().collect())
            }
        }
    }

    /// Of `games` (the last scan's where none are given), those with a last-played time, newest first by the instant
    /// each names, at most `limit`.
    pub fn recent(&self, limit: usize, games: Option<Vec<String>>) -> Result<Vec<String>, Error> {
        let games = match games {
            Some(games) => games,
            None => match self.last_scan() {
                Some(found) => found.games.iter().map(|g| g.path.clone()).collect(),
                None => return Ok(Vec::new()),
            },
        };
        let rows: std::collections::HashMap<String, records::Record> = records::all(&*self.db()?)?.into_iter().collect();
        let mut played: Vec<(i64, &str)> = games
            .iter()
            .filter_map(|g| rows.get(g).and_then(|r| r.last_played.as_deref()).map(|t| (clock::instant(t).unwrap_or(i64::MIN), g.as_str())))
            .collect();
        played.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(played.into_iter().take(limit).map(|(_, p)| p.to_string()).collect())
    }

    /// A reclaim run as a scan is, refused while one runs.
    pub fn reclaim_guarded(&self, games: Option<Vec<String>>) -> Result<usize, Error> {
        self.progress.start()?;
        let result = self.reclaim(games);
        self.progress.finish();
        result
    }

    /// `emusen_library_scan`'s request.
    pub fn scan_request(&self, request: &str) -> Result<usize, Error> {
        let r = json::parse(request)?;
        let beside = scan::Beside { packages: json::text_or_none(&r, "packages")?, compressed: json::text_or_none(&r, "compressed")? };
        self.scan_with(&json::strings(&r, "roots")?, &beside, &json::strings(&r, "systems")?)
    }

    /// `emusen_library_games`' filter and answer.
    pub fn games_request(&self, filter: &str) -> Result<String, Error> {
        let system = if filter.trim().is_empty() { None } else { json::text_or_none(&json::parse(filter)?, "system")? };
        let (outcome, roots, games) = self.games(system.as_deref());
        let mut out = format!("{{\"status\":\"{}\",\"roots\":", outcome.word());
        json::write_strings(&mut out, roots.iter().map(String::as_str));
        out.push_str(",\"games\":[");
        for (i, g) in games.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"path\":");
            out.push_str(&emusen_dotnet::json::quoted(&g.path));
            out.push_str(",\"system\":");
            json::write_text_or_null(&mut out, g.system.as_deref());
            out.push_str(",\"package\":");
            json::write_text_or_null(&mut out, g.package.as_deref());
            out.push('}');
        }
        out.push_str("]}");
        Ok(out)
    }

    /// `emusen_library_identity`'s request and answer.
    pub fn identity_request(&self, request: &str) -> Result<String, Error> {
        let r = json::parse(request)?;
        let path = json::text(&r, "path")?;
        let systems = match r.get("systems") {
            None | Some(emusen_dotnet::json::Value::Null) => None,
            Some(_) => Some(json::strings(&r, "systems")?),
        };
        let sha1 = json::flag(&r, "sha1")?;
        let id = self.identity(&path, systems.as_deref(), json::flag(&r, "crc32")?, sha1, json::flag(&r, "record")?)?;
        let mut out = String::from("{\"path\":");
        out.push_str(&emusen_dotnet::json::quoted(&id.path));
        out.push_str(&format!(",\"bytes\":{},\"modified\":{},\"md5\":", id.bytes, id.modified));
        out.push_str(&emusen_dotnet::json::quoted(&id.md5));
        out.push_str(",\"crc32\":");
        json::write_text_or_null(&mut out, id.crc32.as_deref());
        if sha1 {
            out.push_str(",\"sha1\":");
            json::write_text_or_null(&mut out, id.sha1.as_deref());
        }
        out.push_str(",\"system\":");
        json::write_text_or_null(&mut out, id.system.as_deref());
        out.push_str(&format!(",\"cached\":{}}}", id.cached));
        Ok(out)
    }

    /// `emusen_library_get`'s request and answer.
    pub fn get_request(&self, request: &str) -> Result<String, Error> {
        json::get(self, &json::parse(request)?)
    }

    /// `emusen_library_change`'s request and answer.
    pub fn change_request(&self, request: &str) -> Result<String, Error> {
        json::change(self, &json::parse(request)?)
    }
}
