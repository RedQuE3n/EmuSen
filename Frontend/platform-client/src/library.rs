//! The player's library through emusen_library.h: one games.db, its last scan, and its records. The emusen.games
//! service speaks for the same file to a shell; a front end keeps this handle, as that header says.

use crate::error::{Error, Result};
use crate::out::{check, json, to_json, words};
use crate::platform::{Lent, Platform};
use crate::raw;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::ptr::NonNull;
use std::sync::Arc;

/// What a scan walks: the roots, the systems this build opens (each shelf's), and packages and compressed games beside.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ScanRequest {
    pub roots: Vec<String>,
    pub systems: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed: Option<String>,
}

/// A scan as it stands; `phase` is "walk", "route", "hash" or empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct ScanProgress {
    pub running: bool,
    pub phase: String,
    pub files: u64,
    pub games: u64,
    pub hashed: u64,
    pub to_hash: u64,
}

/// The last scan's games; `status` is "ok", "empty", "not_found" or "no_directory".
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Games {
    pub status: String,
    pub roots: Vec<String>,
    pub games: Vec<GameFile>,
}

/// One game the scan found: its file, its system where routed, its package's folder where it is one's, and since 1.3
/// the folders between its console folder and the file, '/'-separated ("" at the top and for a package's game).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct GameFile {
    pub path: String,
    pub system: Option<String>,
    pub package: Option<String>,
    pub folder: String,
}

/// A game's record of play.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Record {
    pub favourite: bool,
    pub last_played: Option<Value>,
    pub play_count: u64,
    pub play_seconds: f64,
}

/// A collection and how many games it holds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub count: u64,
}

/// One games.db, open.
pub struct Library {
    platform: Arc<Platform>,
    handle: NonNull<raw::emusen_library>,
}

// The header: a scan runs on one thread while progress and cancel are asked from another.
unsafe impl Send for Library {}
unsafe impl Sync for Library {}

impl Library {
    /// Opens games.db at `database` on the SQLite lent, migrating it to this build's schema.
    pub fn open(platform: &Arc<Platform>, database: &Path, sqlite: &Lent) -> Result<Library> {
        let database = database.to_string_lossy();
        let config = raw::emusen_library_config {
            size: std::mem::size_of::<raw::emusen_library_config>() as u32,
            database: database.as_ptr(),
            database_len: database.len(),
            sqlite: sqlite.handle(),
        };
        let mut handle = std::ptr::null_mut();
        let n = unsafe { platform.lib.emusen_library_open(&config, &mut handle) };
        check(n.into(), || library_words(platform))?;
        let handle = NonNull::new(handle).ok_or_else(|| crate::error::status(raw::EMUSEN_LIBRARY_NULL, library_words(platform)))?;
        Ok(Library { platform: platform.clone(), handle })
    }

    fn words(&self) -> String {
        library_words(&self.platform)
    }

    fn call(&self, name: &'static str, request: &str, f: impl Fn(*const u8, usize, *mut u8, usize) -> i64) -> Result<Value> {
        let bytes = crate::out::answer(|o, l| f(request.as_ptr(), request.len(), o, l), || self.words())?;
        json(name, &bytes)
    }

    /// Scans and keeps the result; the games found.
    pub fn scan(&self, request: &ScanRequest) -> Result<u64> {
        let request = to_json(request);
        check(unsafe { self.platform.lib.emusen_library_scan(self.handle.as_ptr(), request.as_ptr(), request.len()) }, || self.words())
            .map(|n| n as u64)
    }

    /// The scan or reclaim running, as it stands.
    pub fn progress(&self) -> Result<ScanProgress> {
        let bytes =
            crate::out::answer(|o, l| unsafe { self.platform.lib.emusen_library_progress(self.handle.as_ptr(), o, l) }, || self.words())?;
        json("progress", &bytes)
    }

    /// Stops a running scan or reclaim at its next file.
    pub fn cancel(&self) -> Result<()> {
        check(unsafe { self.platform.lib.emusen_library_cancel(self.handle.as_ptr()) }.into(), || self.words()).map(drop)
    }

    /// The last scan, narrowed to one system or not.
    pub fn games(&self, system: Option<&str>) -> Result<Games> {
        let filter = system.map(|s| to_json(&serde_json::json!({ "system": s }))).unwrap_or_default();
        let answer = self
            .call("games", &filter, |r, rl, o, l| unsafe { self.platform.lib.emusen_library_games(self.handle.as_ptr(), r, rl, o, l) })?;
        serde_json::from_value(answer).map_err(|e| Error::Shape { call: "games", words: e.to_string() })
    }

    /// What a dump is, {"path", "record": false, ...}: its MD5 from the hash cache while the file is unchanged, else read.
    pub fn identity(&self, request: &Value) -> Result<Value> {
        let request = to_json(request);
        self.call("identity", &request, |r, rl, o, l| unsafe { self.platform.lib.emusen_library_identity(self.handle.as_ptr(), r, rl, o, l) })
    }

    /// One read, {"what": ...}, as the header lists them.
    pub fn get(&self, request: &Value) -> Result<Value> {
        let request = to_json(request);
        self.call("get", &request, |r, rl, o, l| unsafe { self.platform.lib.emusen_library_get(self.handle.as_ptr(), r, rl, o, l) })
    }

    /// One change, {"op": ...}, as the header lists them; null unless the op answers.
    pub fn change(&self, request: &Value) -> Result<Value> {
        let request = to_json(request);
        self.call("change", &request, |r, rl, o, l| unsafe { self.platform.lib.emusen_library_change(self.handle.as_ptr(), r, rl, o, l) })
    }

    /// A game's record of play; none for a game never recorded.
    pub fn record(&self, path: &str) -> Result<Option<Record>> {
        let answer = self.get(&serde_json::json!({ "what": "record", "path": path }))?;
        serde_json::from_value(answer).map_err(|e| Error::Shape { call: "record", words: e.to_string() })
    }

    /// The collections, by name ignoring case.
    pub fn collections(&self) -> Result<Vec<Collection>> {
        let answer = self.get(&serde_json::json!({ "what": "collections" }))?;
        serde_json::from_value(answer).map_err(|e| Error::Shape { call: "collections", words: e.to_string() })
    }

    /// Of the games given, those played, newest first.
    pub fn recent(&self, limit: u32, games: &[String]) -> Result<Vec<String>> {
        let answer = self.get(&serde_json::json!({ "what": "recent", "limit": limit, "games": games }))?;
        serde_json::from_value(answer).map_err(|e| Error::Shape { call: "recent", words: e.to_string() })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_library_close(self.handle.as_ptr()) };
    }
}

fn library_words(platform: &Platform) -> String {
    words(|o, l| unsafe { platform.lib.emusen_library_last_error(o, l) })
}
