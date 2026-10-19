//! A dump's identity, and the rows that follow a game across a rename: the hash cache read before a file is hashed,
//! the game's row given the hash, and the rows of games the scan no longer finds matched by contents with games that
//! have none. Mistress's `IdentifyLater` and `ReclaimMovedGames`, out of the window. See EmuSen_Settings_Reference.md
//! §4.37 and EmuSen_Backend.md §28.4.

use crate::{Error, Library, records};
use emusen_galaxia::{dotnet_files, rom_hash};
use std::collections::HashSet;
use std::sync::atomic::Ordering;

/// What `identity` answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub path: String,
    pub bytes: i64,
    pub modified: i64,
    pub md5: String,
    pub crc32: Option<String>,
    pub sha1: Option<String>,
    pub system: Option<String>,
    pub cached: bool,
}

impl Library {
    /// The dump at `path`: its MD5 from the cache while the file is the size and age it was hashed at, else read; with
    /// `crc32` the file is read for its CRC-32 too, with `sha1` (1.1) for its SHA-1 as well; with `record` the cache and
    /// the game's row are written.
    pub fn identity(&self, path: &str, systems: Option<&[String]>, crc32: bool, sha1: bool, record: bool) -> Result<Identity, Error> {
        let Some((bytes, modified)) = dotnet_files::file_info(path) else { return Err(Error::Io(format!("Could not find file '{path}'."))) };
        let known = records::known_hash(&*self.db()?, path, bytes, modified)?;
        let (md5, crc, sha, cached) = match known {
            Some(md5) if !crc32 && !sha1 => (md5, None, None, true),
            known => {
                let hashes = rom_hash::hashes_with(path, None, sha1, &|| false).map_err(|e| Error::Io(emusen_dotnet::file::io_message(&e, path)))?;
                (hashes.md5, Some(hashes.crc32), hashes.sha1, known.is_some())
            }
        };
        if record {
            let db = self.db()?;
            if !cached {
                records::store_hash(&db, path, bytes, modified, &md5)?;
            }
            records::identify(&db, path, &md5, bytes)?;
        }
        let system = match systems {
            Some(ids) => crate::scan::route(path, &crate::scan::Systems::named(ids)),
            None => {
                let ids = self.last_scan().map(|s| s.systems.clone()).unwrap_or_default();
                if ids.is_empty() { None } else { crate::scan::route(path, &crate::scan::Systems::named(&ids)) }
            }
        };
        Ok(Identity { path: path.to_string(), bytes, modified, md5, crc32: crc, sha1: sha, system, cached })
    }

    /// The rows whose file is not among `games` (the last scan's where none are given), each moved to the first game of
    /// its size and contents that has no row of its own, the files of an orphan's size alone hashed; how many moved.
    pub fn reclaim(&self, games: Option<Vec<String>>) -> Result<usize, Error> {
        let games = match games {
            Some(games) => games,
            None => match self.last_scan() {
                Some(scan) => scan.games.iter().map(|g| g.path.clone()).collect(),
                None => return Ok(0),
            },
        };
        let present: HashSet<String> = games.iter().cloned().collect();
        let (orphans, candidates) = {
            let db = self.db()?;
            let orphans = records::orphans(&db, &present)?;
            if orphans.is_empty() {
                return Ok(0);
            }
            let sizes: HashSet<i64> = orphans.iter().map(|o| o.2).collect();
            let rows: HashSet<String> = records::all(&db)?.into_iter().map(|(p, _)| p).collect();
            let mut candidates = Vec::new();
            for game in &games {
                if rows.contains(game) {
                    continue;
                }
                let Some((bytes, modified)) = dotnet_files::file_info(game) else { continue };
                if !sizes.contains(&bytes) {
                    continue;
                }
                let known = records::known_hash(&db, game, bytes, modified)?;
                candidates.push((game.clone(), bytes, modified, known));
            }
            (orphans, candidates)
        };
        if candidates.is_empty() {
            return Ok(0);
        }
        self.progress.phase.store(3, Ordering::Relaxed);
        self.progress.to_hash.store(candidates.iter().filter(|c| c.3.is_none()).count(), Ordering::Relaxed);
        let mut hashed = Vec::with_capacity(candidates.len());
        for (path, bytes, modified, known) in candidates {
            if self.progress.cancelled() {
                return Err(Error::Cancelled);
            }
            let md5 = match known {
                Some(md5) => Some(md5),
                None => {
                    self.progress.hashed.fetch_add(1, Ordering::Relaxed);
                    rom_hash::md5(&path).ok()
                }
            };
            hashed.push((path, bytes, modified, md5));
        }
        let db = self.db()?;
        for (path, bytes, modified, md5) in &hashed {
            if let Some(md5) = md5 {
                records::store_hash(&db, path, *bytes, *modified, md5)?;
            }
        }
        let mut claimed: HashSet<&str> = HashSet::new();
        let mut moved = 0;
        for (from, md5, _) in &orphans {
            let Some((to, ..)) = hashed.iter().find(|c| c.3.as_deref() == Some(md5.as_str()) && !claimed.contains(c.0.as_str())) else { continue };
            records::move_game(&db, from, to)?;
            claimed.insert(to);
            moved += 1;
        }
        Ok(moved)
    }
}
