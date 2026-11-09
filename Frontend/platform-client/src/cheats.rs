//! The player's cheat database (emusen_platform.h, `emusen_cheats_database_*`; `man cheat`): a tree of `.cht` files
//! scanned once, asked by system, by game and by text, and pruned to the systems a core can use. Requests and answers
//! are the header's JSON and cross as `serde_json::Value`.

use crate::error::Result;
use crate::out::{ANSWER_ROOM, json, once, to_json};
use crate::platform::Platform;
use crate::raw;
use serde_json::Value;
use std::ptr::NonNull;
use std::sync::Arc;

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
    /// A plan applied inside its folder, `{"directory", "removing": [{system, path}]}`: `{"removed", "failed"}`, each
    /// failure a folder's words. Asked once, since a second call would find the folders gone.
    pub fn cheats_prune_apply(&self, request: &Value) -> Result<Value> {
        let r = to_json(request);
        let room = ANSWER_ROOM + 2 * r.len() + 512 * request["removing"].as_array().map_or(0, Vec::len);
        let bytes = once(|o, l| unsafe { self.lib.emusen_cheats_prune_apply(r.as_ptr(), r.len(), o, l) }, room, || self.last_error())?;
        json("cheats_prune_apply", &bytes)
    }
}
