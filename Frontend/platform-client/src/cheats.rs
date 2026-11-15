//! The player's cheat database (emusen_platform.h, `emusen_cheats_database_*`; `man cheat`): a tree of `.cht` files
//! scanned once, asked by system, by game and by text, pruned to the systems a core can use, and installed from an
//! archive; a `.cht` text read as a game's own list. Requests and answers
//! are the header's JSON and cross as `serde_json::Value`.

use crate::error::Result;
use crate::out::{json, to_json};
use crate::platform::Platform;
use crate::raw;
use serde_json::Value;
use std::ptr::NonNull;
use std::sync::Arc;

/// Where the libretro cheat database is downloaded from, the header's `EMUSEN_CHEATS_DATABASE_URL`.
pub fn database_url() -> &'static str {
    let named = raw::EMUSEN_CHEATS_DATABASE_URL;
    std::str::from_utf8(&named[..named.len() - 1]).unwrap_or_default()
}

/// One file of the database: its system's folder, its game, its path.
pub type Entry = (String, String, String);

/// A database as scanned from its folder; never changed after.
pub struct CheatsDatabase {
    platform: Arc<Platform>,
    db: NonNull<raw::emusen_cheats_database>,
}

// The header: scanned once and never changed after, so any thread may ask it.
unsafe impl Send for CheatsDatabase {}
unsafe impl Sync for CheatsDatabase {}

impl CheatsDatabase {
    /// The database under `directory`; a folder that is absent or cannot be listed whole holds no files.
    pub fn open(platform: &Arc<Platform>, directory: &str) -> Result<CheatsDatabase> {
        let db = unsafe { platform.lib.emusen_cheats_database_open(directory.as_ptr(), directory.len()) };
        let db = NonNull::new(db).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_BAD_STRING, platform.last_error()))?;
        Ok(CheatsDatabase { platform: platform.clone(), db })
    }

    /// Every file, in the database's order.
    pub fn entries(&self) -> Result<Vec<Entry>> {
        let lib = &self.platform.lib;
        let bytes = self.platform.read(|o, l| unsafe { lib.emusen_cheats_database_entries(self.db.as_ptr(), o, l) })?.unwrap_or_default();
        json("cheats_database_entries", &bytes)
    }

    /// A question: `{"op": "systems"}`, `{"op": "games", "system", "filter"}` or `{"op": "find", "text", "limit"}`.
    pub fn query(&self, request: &Value) -> Result<Value> {
        let r = to_json(request);
        let lib = &self.platform.lib;
        let bytes = self
            .platform
            .read(|o, l| unsafe { lib.emusen_cheats_database_query(self.db.as_ptr(), r.as_ptr(), r.len(), o, l) })?
            .unwrap_or_default();
        json("cheats_database_query", &bytes)
    }

    /// A prune's plan from `{"keep": [system, ...]}`: what would go, what stays, or the reason nothing would.
    pub fn prune_plan(&self, request: &Value) -> Result<Value> {
        let r = to_json(request);
        let lib = &self.platform.lib;
        let bytes = self
            .platform
            .read(|o, l| unsafe { lib.emusen_cheats_prune_plan(self.db.as_ptr(), r.as_ptr(), r.len(), o, l) })?
            .unwrap_or_default();
        json("cheats_prune_plan", &bytes)
    }
}

impl Drop for CheatsDatabase {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_cheats_database_free(self.db.as_ptr()) };
    }
}

impl Platform {
    /// A `.cht` text read: `{"path", "system"}`, `{"text", "system"}` or `{"text", "codec", "space", "patch"}` in; `{"cheats", "skipped",
    /// "file"}`, `file` the cheats as a game's own list, the CheatFile model's document.
    pub fn cheats_parse(&self, request: &Value) -> Result<Value> {
        let r = to_json(request);
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_cheats_parse(r.as_ptr(), r.len(), o, l) })?.unwrap_or_default();
        json("cheats_parse", &bytes)
    }

    /// A downloaded database's archive installed under `directory`, made whole beside it first so a refusal leaves the
    /// database as it was: `{"installed", "skipped"}`. The library installs once and answers the fill from that.
    pub fn cheats_install(&self, zip: &[u8], directory: &str) -> Result<Value> {
        let bytes = self
            .read(|o, l| unsafe { self.lib.emusen_cheats_install(zip.as_ptr(), zip.len(), directory.as_ptr(), directory.len(), o, l) })?
            .unwrap_or_default();
        json("cheats_install", &bytes)
    }

    /// A plan applied inside its folder, `{"directory", "removing": [{system, path}]}`: `{"removed", "failed"}`, each
    /// failure a folder's words. The library applies it once and answers the fill from that.
    pub fn cheats_prune_apply(&self, request: &Value) -> Result<Value> {
        let r = to_json(request);
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_cheats_prune_apply(r.as_ptr(), r.len(), o, l) })?.unwrap_or_default();
        json("cheats_prune_apply", &bytes)
    }
}
