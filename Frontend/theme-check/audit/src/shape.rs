//! A shape of the synthetic library for a state (man theme-check, STATES): some of its games, with or without their covers,
//! played or never played, read through the real clients' answers with nothing else changed.

use emusen_crystal_client::shapes::SystemRow;
use emusen_platform_client::library::GameFile;
use emusen_qt_models::esde_shelves::{MediaIndex, Plays};
use emusen_qt_models::gamelists::Collection;
use emusen_qt_models::library::{Engine, LibrarySource};
use serde_json::Value;

/// The clients' answers over the games whose files `keep` names, their covers and their plays kept or left out.
pub struct Shaped<'a> {
    pub inner: &'a dyn LibrarySource,
    pub keep: Vec<String>,
    pub covers: bool,
    pub played: bool,
}

impl LibrarySource for Shaped<'_> {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.inner.systems()
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok(self.inner.games()?.into_iter().filter(|g| self.keep.contains(&g.path)).collect())
    }
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        if self.played { self.inner.favourites(games) } else { Ok(Vec::new()) }
    }
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        if self.played { self.inner.recent(games) } else { Ok(Vec::new()) }
    }
    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        self.inner.engines(systems)
    }
    fn media(&self, systems: &[SystemRow], games: &[GameFile], types: &[&str]) -> Result<Vec<(String, String)>, String> {
        if self.covers { self.inner.media(systems, games, types) } else { Ok(Vec::new()) }
    }
    fn metadata(&self, games: &[GameFile]) -> Result<Vec<(String, Value)>, String> {
        self.inner.metadata(games)
    }
    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        if self.played { self.inner.played(games) } else { Ok(Vec::new()) }
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        self.inner.shelves()
    }
    fn settings(&self) -> Result<Value, String> {
        self.inner.settings()
    }
    fn plays(&self, games: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        if self.played { self.inner.plays(games) } else { Ok(Vec::new()) }
    }
    fn media_index(&self, systems: &[String]) -> Result<MediaIndex, String> {
        if self.covers { self.inner.media_index(systems) } else { Ok(MediaIndex::default()) }
    }
    fn collections(&self) -> Result<Vec<Collection>, String> {
        self.inner.collections()
    }
}
