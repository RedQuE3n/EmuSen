//! The library with 120 consoles, as a player with many cores installed has it (decided 2026-11-08): the real clients' answers
//! with synthetic consoles added: 40 under one maker in all, ten under none, the rest under the table's other makers, a game each.

use emusen_crystal_client::shapes::SystemRow;
use emusen_platform_client::library::GameFile;
use emusen_qt_models::esde_shelves::{MediaIndex, Plays};
use emusen_qt_models::gamelists::Collection;
use emusen_qt_models::library::{Engine, LibrarySource};
use serde_json::Value;
use std::path::Path;

/// How many consoles the crowded library holds, how many of them one maker has, and how many have no maker.
pub const CONSOLES: usize = 120;
const ONE_MAKER: usize = 40;
const NO_MAKER: usize = 10;

/// The clients' answers, with `systems` and `games` added to.
pub struct Crowd<'a> {
    inner: &'a dyn LibrarySource,
    systems: Vec<SystemRow>,
    games: Vec<GameFile>,
}

impl<'a> Crowd<'a> {
    /// The added consoles copy a shelved console's row of each maker, under a made-up name, id, year and hue.
    pub fn new(inner: &'a dyn LibrarySource, root: &Path) -> Result<Crowd<'a>, String> {
        let table = inner.systems()?;
        let games = inner.games()?;
        let shelved: Vec<&SystemRow> = table.iter().filter(|s| games.iter().any(|g| g.system.as_ref() == Some(&s.id))).collect();
        let first = shelved.first().ok_or("no console has a game")?;
        let maker = first.manufacturer.clone();
        let mut makers = table.iter().map(|s| s.manufacturer.clone()).filter(|m| !m.is_empty() && *m != maker).collect::<Vec<_>>();
        makers.dedup();
        let same = shelved.iter().filter(|s| s.manufacturer == maker).count();
        let added = CONSOLES - shelved.len();
        let mut systems = Vec::new();
        let mut files = Vec::new();
        for n in 1..=added {
            let manufacturer = if n <= ONE_MAKER - same {
                maker.clone()
            } else if n <= ONE_MAKER - same + NO_MAKER {
                String::new()
            } else {
                makers.get(n % makers.len().max(1)).cloned().unwrap_or_default()
            };
            let id = format!("synthetic{n:03}");
            let name = format!("Synthetic {n:03}");
            let mut row = (*first).clone();
            row.look.hue = (n * 37 % 360) as f64;
            row.release_year = 1980 + (n % 40) as i32;
            (row.id, row.name, row.display_name, row.console, row.esde_system, row.esde_full_name, row.manufacturer) =
                (id.clone(), name.clone(), name.clone(), name.clone(), id.clone(), name.clone(), manufacturer);
            row.extensions = vec![format!(".s{n:03}")];
            files.push(GameFile { path: root.join(format!("Roms/{name}.s{n:03}")).to_string_lossy().into_owned(), system: Some(id), ..GameFile::default() });
            systems.push(row);
        }
        Ok(Crowd { inner, systems, games: files })
    }
}

impl LibrarySource for Crowd<'_> {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.inner.systems()?.into_iter().chain(self.systems.iter().cloned()).collect())
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok(self.inner.games()?.into_iter().chain(self.games.iter().cloned()).collect())
    }
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        self.inner.favourites(games)
    }
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        self.inner.recent(games)
    }
    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        self.inner.engines(systems)
    }
    fn media(&self, systems: &[SystemRow], games: &[GameFile], types: &[&str]) -> Result<Vec<(String, String)>, String> {
        self.inner.media(systems, games, types)
    }
    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        self.inner.played(games)
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        let order = self.inner.shelves()?;
        Ok(if order.is_empty() { order } else { order.into_iter().chain(self.systems.iter().map(|s| s.id.clone())).collect() })
    }
    fn settings(&self) -> Result<Value, String> {
        self.inner.settings()
    }
    fn plays(&self, games: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        self.inner.plays(games)
    }
    fn media_index(&self, systems: &[String]) -> Result<MediaIndex, String> {
        self.inner.media_index(systems)
    }
    fn collections(&self) -> Result<Vec<Collection>, String> {
        self.inner.collections()
    }
}
