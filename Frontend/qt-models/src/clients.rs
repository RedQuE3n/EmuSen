//! The library view's source over the two client crates: the systems and the shelves from the frontend API, the games
//! from a scan of the library handle, whose roots are the player's ROM folder (AppSettings) and whose systems are the
//! shelves' (emusen_library.h), the records from games.db, where Galaxia names it, and each game's cover from the media
//! store through the scraper's sources-alone handle (emusen.scrape `media`, EmuSen_Backend.md §40.5).

use crate::library::{without_port_mark, Engine, LibrarySource};
use emusen_crystal_client::shapes::{CoreRow, SystemRow};
use emusen_crystal_client::{Runtime, Scope};
use emusen_platform_client::galaxia::consts::{DIR_COMPRESSED, DIR_MEDIA, DIR_PACKAGES, FILE_GAMES, FILE_SETTINGS, MODEL_APP_SETTINGS};
use emusen_platform_client::library::{GameFile, Library, ScanRequest};
use emusen_platform_client::scrape::{ScrapeService, ScraperConfig};
use emusen_platform_client::{Lent, Platform};
use serde_json::{json, Value};
use std::sync::Arc;

/// The back end as the library view reads it.
pub struct ClientSource {
    pub runtime: Runtime,
    pub platform: Arc<Platform>,
    pub library: Library,
    /// The SQLite lent to the library and the scraper, kept while they are open.
    _sqlite: Lent,
    /// The sources-alone scraper's serial and its table, for the covers; none where it could not open.
    scraper: Option<(u64, ScrapeService)>,
}

impl Drop for ClientSource {
    fn drop(&mut self) {
        if let Some((serial, _)) = &self.scraper {
            let _ = self.platform.scraper_close(*serial);
        }
    }
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

    fn covers(&self, systems: &[SystemRow], games: &[GameFile]) -> Result<Vec<(String, String)>, String> {
        let Some((_, scraper)) = &self.scraper else { return Ok(Vec::new()) };
        let mut out = Vec::new();
        for g in games {
            let Some(system) = systems.iter().find(|s| Some(&s.id) == g.system.as_ref()) else { continue };
            let found = scraper.get(&json!({ "what": "media", "path": g.path, "system": system.esde_system, "type": "cover" })).map_err(words)?;
            if let Some(path) = found.get("path").and_then(Value::as_str) {
                out.push((g.path.clone(), path.to_owned()));
            }
        }
        Ok(out)
    }

    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        let records = self.library.get(&json!({ "what": "records" })).map_err(words)?;
        let at = |g: &GameFile| records.get(&g.path)?.get("last_played")?.as_str().and_then(crate::words::parse_utc);
        Ok(games.iter().filter_map(|g| at(g).map(|t| (g.path.clone(), t))).collect())
    }

    fn shelves(&self) -> Result<Vec<String>, String> {
        Ok(self.runtime.shelves().map_err(words)?.into_iter().map(|s| s.system).collect())
    }

    fn settings(&self) -> Result<Value, String> {
        app_settings(&self.platform)
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
        let games_db = platform.file(FILE_GAMES).map_err(words)?;
        let library = Library::open(&platform, &games_db, &sqlite).map_err(words)?;
        let scraper = open_scraper(&platform, &runtime, &games_db.to_string_lossy(), &sqlite);
        Ok(ClientSource { runtime, platform, library, _sqlite: sqlite, scraper })
    }
}

/// The scraper as a handle for finding pictures alone, with the player's folders as Mistress gives them; none where the
/// platform library could not open one, so the screens show generated cards.
fn open_scraper(platform: &Arc<Platform>, runtime: &Runtime, games_db: &str, sqlite: &Lent) -> Option<(u64, ScrapeService)> {
    let settings = app_settings(platform).ok()?;
    let text = |key: &str| settings.get(key).and_then(Value::as_str).filter(|v| !v.is_empty());
    let cores: Vec<Vec<String>> = runtime.cores().ok()?.into_iter().map(|c| c.systems).filter(|s| !s.is_empty()).collect();
    let config = ScraperConfig {
        root: platform.directory(DIR_MEDIA).ok()??.to_string_lossy().into_owned(),
        games_db: Some(games_db.to_owned()),
        settings: json!({
            "roms": text("RomDirectory"), "art": text("ArtworkDirectory"), "esde_media": text("EsdeMediaDirectory"),
            "openemu": settings.get("OpenEmuFallback").and_then(Value::as_bool).unwrap_or(true),
            "developer_files": false, "cores": cores, "sources_alone": true,
        }),
        ..ScraperConfig::default()
    };
    let serial = platform.scraper_open(&config, Some(sqlite)).ok()?;
    Some((serial, platform.scrape_service().ok()?))
}
