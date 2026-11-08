//! The library view's source over the two client crates: the systems and the shelves from the frontend API, the games
//! from a scan of the library handle, whose roots are the player's ROM folder (AppSettings) and whose systems are the
//! shelves' (emusen_library.h), and the records from games.db, where Galaxia names it.

use crate::library::{without_port_mark, Engine, LibrarySource};
use emusen_crystal_client::shapes::{CoreRow, SystemRow};
use emusen_crystal_client::{Runtime, Scope};
use emusen_platform_client::galaxia::consts::{DIR_COMPRESSED, DIR_PACKAGES, FILE_GAMES, FILE_SETTINGS, MODEL_APP_SETTINGS};
use emusen_platform_client::library::{GameFile, Library, ScanRequest};
use emusen_platform_client::Platform;
use serde_json::{json, Value};
use std::sync::Arc;

/// The back end as the library view reads it.
pub struct ClientSource {
    pub runtime: Runtime,
    pub platform: Arc<Platform>,
    pub library: Library,
}

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl ClientSource {
    /// The scan's request: the player's ROM folder, the shelves' systems, and the packages and compressed folders.
    pub fn scan_request(&self) -> Result<ScanRequest, String> {
        let settings = app_settings(&self.platform)?;
        let roots =
            settings.get("RomDirectory").and_then(Value::as_str).filter(|r| !r.is_empty()).map(|r| vec![r.to_owned()]).unwrap_or_default();
        let mut systems: Vec<String> = self.runtime.shelves().map_err(words)?.into_iter().map(|s| s.system).collect();
        systems.dedup();
        let folder = |which| self.platform.directory(which).map(|d| d.map(|d| d.to_string_lossy().into_owned()));
        Ok(ScanRequest {
            roots,
            systems,
            packages: folder(DIR_PACKAGES).map_err(words)?,
            compressed: folder(DIR_COMPRESSED).map_err(words)?,
        })
    }
}

/// The player's settings as Galaxia's model reads them, the file Mistress reads; null where there is none yet.
pub fn app_settings(platform: &Platform) -> Result<Value, String> {
    // The settings file sits in the config directory itself, so its name is all the model's load asks.
    let file = platform.file(FILE_SETTINGS).map_err(words)?;
    let name = file.file_name().and_then(|n| n.to_str()).ok_or_else(|| format!("{} names no settings file", file.display()))?;
    Ok(platform.model_load(MODEL_APP_SETTINGS, None, name).map_err(words)?.unwrap_or(Value::Null))
}

impl LibrarySource for ClientSource {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.runtime.systems().map_err(words)
    }

    fn games(&self) -> Result<Vec<GameFile>, String> {
        self.library.scan(&self.scan_request()?).map_err(words)?;
        Ok(self.library.games(None).map_err(words)?.games)
    }

    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        let records = self.library.get(&json!({ "what": "records" })).map_err(words)?;
        let favourite = |g: &&GameFile| records.get(&g.path).and_then(|r| r.get("favourite")).and_then(Value::as_bool).unwrap_or(false);
        Ok(games.iter().filter(favourite).map(|g| g.path.clone()).collect())
    }

    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        let paths: Vec<String> = games.iter().map(|g| g.path.clone()).collect();
        self.library.recent(u32::MAX, &paths).map_err(words)
    }

    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        let cores = self.runtime.cores().map_err(words)?;
        let mut out = Vec::new();
        for s in systems {
            // The runtime's Setting(console, "Engine"): the stored choice, else the engine row's default.
            let chosen = self.runtime.settings_get(0, Scope::Console, Some(&s.id), "Engine").map_err(words)?;
            let Some(id) = chosen.filter(|c| !c.is_empty()).or_else(|| s.engines.first().cloned()) else { continue };
            out.push((s.id.clone(), engine_shown(&id, &cores)));
        }
        Ok(out)
    }
}

/// An engine by the name a choice stores, named from the runtime's cores listing: a libretro core by its corename, an
/// EmuSen core by its info name less the port's mark, and an engine the listing lacks by its stored name.
pub fn engine_shown(id: &str, cores: &[CoreRow]) -> Engine {
    let row = cores.iter().find(|c| c.engine.as_deref() == Some(id));
    let libretro = row.and_then(|r| r.libretro.as_ref());
    let name = match (row, libretro) {
        (_, Some(l)) => l.corename.clone().or_else(|| row.and_then(|r| r.name.clone())).unwrap_or_else(|| id.to_owned()),
        (Some(r), None) => without_port_mark(r.name.as_deref().unwrap_or(id)).to_owned(),
        (None, None) => id.to_owned(),
    };
    Engine { id: id.to_owned(), name, libretro: libretro.is_some() }
}

impl ClientSource {
    /// The clients opened on the platform library at `platform_library`, a runtime made with `config`, and games.db where
    /// Galaxia names it, on the system's SQLite, as the Qt front end lends it (EmuSen_Qt.md §1.1.2).
    pub fn open(platform_library: &str, config: &emusen_crystal_client::Config) -> Result<ClientSource, String> {
        let platform = Platform::load(platform_library).map_err(words)?;
        let api = emusen_crystal_client::Api::load(platform_library).map_err(words)?;
        let runtime = Runtime::new(&api, config).map_err(words)?;
        let sqlite = emusen_platform_client::Lent::open("libsqlite3.so.0").map_err(words)?;
        let library = Library::open(&platform, &platform.file(FILE_GAMES).map_err(words)?, &sqlite).map_err(words)?;
        Ok(ClientSource { runtime, platform, library })
    }
}
