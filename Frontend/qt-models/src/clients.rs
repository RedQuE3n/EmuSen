//! The library view's source over the two client crates: the systems and the shelves from the frontend API, the games
//! from a scan of the library handle, whose roots are the player's ROM folder (AppSettings) and whose systems are the
//! shelves' (emusen_library.h), the records from games.db, where Galaxia names it, and each game's cover and hero art from the media
//! store through the scraper's sources-alone handle (emusen.scrape `media`, EmuSen_Backend.md §40.5).

use crate::esde_shelves::{MediaIndex, Plays};
use crate::gamelists::Collection;
use crate::library::{without_port_mark, Engine, LibrarySource};
use emusen_crystal_client::shapes::{CoreRow, SystemRow};
use emusen_crystal_client::{Runtime, Scope};
use emusen_platform_client::galaxia::consts::{DIR_COMPRESSED, DIR_MEDIA, DIR_PACKAGES, FILE_GAMES, FILE_SETTINGS, MODEL_APP_SETTINGS};
use emusen_platform_client::library::{GameFile, Library, ScanRequest};
use emusen_platform_client::scrape::{ScrapeService, ScraperConfig};
use emusen_platform_client::{Lent, Platform};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// The back end as the library view reads it.
pub struct ClientSource {
    pub runtime: Runtime,
    pub platform: Arc<Platform>,
    pub library: Library,
    /// The SQLite lent to the library and the scraper, kept while they are open.
    _sqlite: Lent,
    /// The scraper for the covers, opened for its sources alone except while a scrape runs; none where it could not open.
    scraper: Option<Scraper>,
    /// The last media index and the scraper's stamps it was walked at, so an unchanged tree is not walked again.
    walked: Mutex<Option<(String, MediaIndex)>>,
}

/// The process's one scraper as a source holds it: its table, what it was opened with for its sources alone, and the
/// serial of the open it made last, which a scrape changes (emusen_scrape_service.h).
struct Scraper {
    service: ScrapeService,
    config: ScraperConfig,
    serial: Mutex<u64>,
}

impl Drop for ClientSource {
    fn drop(&mut self) {
        if let Some(scraper) = &self.scraper {
            let _ = self.platform.scraper_close(*scraper.serial.lock().unwrap_or_else(|p| p.into_inner()));
        }
    }
}

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl ClientSource {
    /// The sources-alone scraper, for a game's metadata and media; none where it could not open.
    pub fn scraper(&self) -> Option<&ScrapeService> {
        self.scraper.as_ref().map(|s| &s.service)
    }

    /// A run started on `request` (emusen_scrape_service.h `start`): the scraper opened again with its store, the player's
    /// ScreenScraper developer file read, and `test` as its test seam where one is given; how many games it queued
    /// (EmuSen_Backend.md §30.3).
    pub fn scrape_start(&self, request: &Value, test: Option<&Value>) -> Result<i64, String> {
        let scraper = self.scraper.as_ref().ok_or("the scraper could not be opened")?;
        let mut config = scraper.config.clone();
        config.settings["sources_alone"] = json!(false);
        config.settings["developer_files"] = json!(true);
        if let Some(test) = test {
            config.settings["test"] = test.clone();
        }
        self.reopen(scraper, &config)?;
        match scraper.service.start(request) {
            Ok(0) => Err("the scraper queued nothing".into()),
            Ok(n) => Ok(n),
            Err(why) => Err(why.to_string()),
        }
    }

    /// The scraper opened again on `config`, closing the one this source opened.
    fn reopen(&self, scraper: &Scraper, config: &ScraperConfig) -> Result<(), String> {
        let mut serial = scraper.serial.lock().unwrap_or_else(|p| p.into_inner());
        *serial = self.platform.scraper_open(config, Some(&self._sqlite)).map_err(words)?;
        Ok(())
    }

    /// Waits at most `ms` for the run's serial to pass `serial`; the serial then.
    pub fn scrape_wait(&self, serial: u64, ms: u32) -> Result<u64, String> {
        self.scraper().ok_or("the scraper could not be opened")?.wait(serial, ms).map_err(words)
    }

    /// The run as it stands, every result of it.
    pub fn scrape_progress(&self) -> Result<Value, String> {
        self.scraper().ok_or("the scraper could not be opened")?.progress(0).map_err(words)
    }

    /// The run paused, or resumed; true when that changed it.
    pub fn scrape_pause(&self, paused: bool) -> Result<bool, String> {
        self.scraper().ok_or("the scraper could not be opened")?.pause(paused).map_err(words)
    }

    /// The run stopped; what was fetched is kept.
    pub fn scrape_cancel(&self) -> Result<bool, String> {
        self.scraper().ok_or("the scraper could not be opened")?.cancel().map_err(words)
    }

    /// The run over: the scraper opened again for its sources alone, its store let go for the Scraper tool and DianaSh.
    pub fn scrape_end(&self) -> Result<(), String> {
        let scraper = self.scraper.as_ref().ok_or("the scraper could not be opened")?;
        self.reopen(scraper, &scraper.config)
    }

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

/// The player's settings with one key changed, saved through Galaxia's model, which writes every other key as it read it.
pub fn store_app_setting(platform: &Platform, key: &str, value: Value) -> Result<(), String> {
    change_app_settings(platform, |settings| settings[key] = value).map(|_| ())
}

/// The player's settings changed by `change` and saved in one write, as `store_app_setting` saves one key; gives what was saved.
pub fn change_app_settings(platform: &Platform, change: impl FnOnce(&mut Value)) -> Result<Value, String> {
    let file = platform.file(FILE_SETTINGS).map_err(words)?;
    let name = file.file_name().and_then(|n| n.to_str()).ok_or_else(|| format!("{} names no settings file", file.display()))?;
    let mut settings = match app_settings(platform)? {
        Value::Null => platform.model_new(MODEL_APP_SETTINGS).map_err(words)?,
        read => read,
    };
    change(&mut settings);
    platform.model_save(MODEL_APP_SETTINGS, None, name, &settings).map_err(words)?;
    Ok(settings)
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

    fn hidden(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        let edits = self.library.get(&json!({ "what": "all_edits" })).map_err(words)?;
        let hidden = |g: &&GameFile| edits.get(&g.path).is_some_and(|e| crate::edits::flag(e, crate::edits::HIDDEN));
        Ok(games.iter().filter(hidden).map(|g| g.path.clone()).collect())
    }

    fn names(&self, games: &[GameFile]) -> Result<Vec<(String, String)>, String> {
        let edits = self.library.get(&json!({ "what": "all_edits" })).map_err(words)?;
        let named = |g: &GameFile| edits.get(&g.path)?.get(crate::edits::NAME)?.as_str().filter(|n| !n.trim().is_empty()).map(|n| (g.path.clone(), n.to_owned()));
        Ok(games.iter().filter_map(named).collect())
    }

    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        let paths: Vec<String> = games.iter().map(|g| g.path.clone()).collect();
        self.library.recent(u32::MAX, &paths).map_err(words)
    }

    fn media(&self, systems: &[SystemRow], games: &[GameFile], types: &[&str]) -> Result<Vec<(String, String)>, String> {
        let Some(scraper) = self.scraper() else { return Ok(Vec::new()) };
        let mut out = Vec::new();
        for g in games {
            let Some(system) = systems.iter().find(|s| Some(&s.id) == g.system.as_ref()) else { continue };
            for kind in types {
                let found = scraper.get(&json!({ "what": "media", "path": g.path, "system": system.esde_system, "type": kind })).map_err(words)?;
                if let Some(path) = found.get("path").and_then(Value::as_str) {
                    out.push((g.path.clone(), path.to_owned()));
                    break;
                }
            }
        }
        Ok(out)
    }

    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        let records = self.library.get(&json!({ "what": "records" })).map_err(words)?;
        let at = |g: &GameFile| records.get(&g.path)?.get("last_played")?.as_str().and_then(crate::words::parse_utc);
        Ok(games.iter().filter_map(|g| at(g).map(|t| (g.path.clone(), t))).collect())
    }

    fn started(&self, path: &str) -> Result<(), String> {
        let at = crate::words::utc(crate::words::now());
        self.library.change(&json!({ "op": "started", "path": path, "at": at })).map(|_| ()).map_err(words)
    }

    fn toggle_favourite(&self, path: &str) -> Result<(), String> {
        self.library.change(&json!({ "op": "toggle_favourite", "path": path })).map(|_| ()).map_err(words)
    }

    fn shelves(&self) -> Result<Vec<String>, String> {
        Ok(self.runtime.shelves().map_err(words)?.into_iter().map(|s| s.system).collect())
    }

    fn settings(&self) -> Result<Value, String> {
        app_settings(&self.platform)
    }

    fn metadata(&self, games: &[GameFile]) -> Result<Vec<(String, crate::esde_shelves::Metadata)>, String> {
        let Some(scraper) = self.scraper() else { return Ok(Vec::new()) };
        let names = app_settings(&self.platform)?.get("ScrapeGameNames").and_then(Value::as_bool).unwrap_or(false);
        let one = |g: &GameFile| {
            let answer = scraper.get(&json!({ "what": "metadata", "path": g.path, "scraped_names": names }));
            answer.map(|m| (g.path.clone(), crate::esde_shelves::metadata_of(&m)))
        };
        games.iter().map(one).collect::<Result<_, _>>().map_err(words)
    }

    fn firmware(&self, system: &str) -> Result<Vec<emusen_crystal_client::shapes::FirmwareFile>, String> {
        self.runtime.firmware_status(system).map(|s| s.map(|s| s.firmware).unwrap_or_default()).map_err(words)
    }

    fn plays(&self, games: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        let records = self.library.get(&json!({ "what": "records" })).map_err(words)?;
        let number = |r: &Value, key: &str| r.get(key).and_then(Value::as_i64).unwrap_or(0);
        let of = |g: &GameFile| records.get(&g.path).map(|r| (g.path.clone(), Plays { count: number(r, "play_count"), seconds: number(r, "play_seconds") }));
        Ok(games.iter().filter_map(of).collect())
    }

    fn collections(&self) -> Result<Vec<Collection>, String> {
        let all = self.library.get(&json!({ "what": "collections" })).map_err(words)?;
        let mut out = Vec::new();
        for c in all.as_array().into_iter().flatten() {
            let Some(id) = c.get("id").and_then(Value::as_i64) else { continue };
            let members = self.library.get(&json!({ "what": "members", "id": id })).map_err(words)?;
            let members = members.as_array().map(|m| m.iter().filter_map(|p| p.as_str().map(str::to_owned)).collect()).unwrap_or_default();
            out.push(Collection { id, name: c.get("name").and_then(Value::as_str).unwrap_or("").to_owned(), members });
        }
        Ok(out)
    }

    fn cheats(&self, console: &str, path: &str) -> Result<usize, String> {
        Ok(crate::cheats::List::of(&self.platform, console, path)?.map_or(0, |l| l.entries().len()))
    }

    fn played_for(&self, path: &str, seconds: f64, max_hours: i32) -> Result<Option<f64>, String> {
        let kept = self.library.change(&json!({ "op": "played", "path": path, "seconds": seconds, "max_hours": max_hours })).map_err(words)?;
        Ok(kept.as_f64())
    }

    fn folder_links(&self) -> Result<Vec<(String, String)>, String> {
        let edits = self.library.get(&json!({ "what": "all_edits" })).map_err(words)?;
        let link = |(folder, e): (&String, &Value)| {
            let link = e.get("folderlink")?.as_str().filter(|l| !l.is_empty())?;
            Some((folder.clone(), lexical(&std::path::Path::new(folder).join(link))))
        };
        Ok(edits.as_object().map(|by| by.iter().filter_map(link).collect()).unwrap_or_default())
    }

    fn media_index(&self, systems: &[String]) -> Result<MediaIndex, String> {
        let Some(scraper) = self.scraper() else { return Ok(MediaIndex::default()) };
        let settings = app_settings(&self.platform)?;
        let store = self.platform.directory(DIR_MEDIA).map_err(words)?;
        let esde = settings.get("EsdeMediaDirectory").and_then(Value::as_str).filter(|d| !d.trim().is_empty()).map(std::path::PathBuf::from);
        let roots: Vec<std::path::PathBuf> = store.into_iter().chain(esde).collect();
        let mut stamps = roots.iter().map(|r| r.to_string_lossy().into_owned()).collect::<Vec<_>>().join("\n");
        for system in systems {
            let stamp = scraper.get(&json!({ "what": "stamp", "system": system })).map_err(words)?;
            stamps.push_str(&format!("\n{system}={stamp}"));
        }
        let mut walked = self.walked.lock().map_err(words)?;
        if let Some((_, index)) = walked.as_ref().filter(|(at, _)| *at == stamps) {
            return Ok(index.clone());
        }
        let kinds = scraper.get(&json!({ "what": "kinds" })).map_err(words)?;
        let kinds: Vec<(&str, &str)> = kinds
            .as_array()
            .map(|all| all.iter().filter_map(|k| Some((k.get("type")?.as_str()?, k.get("folder")?.as_str()?))).collect())
            .unwrap_or_default();
        let mut index = MediaIndex::default();
        for root in &roots {
            for system in systems {
                for (kind, folder) in &kinds {
                    index.walk(root, system, kind, folder);
                }
            }
        }
        *walked = Some((stamps, index.clone()));
        Ok(index)
    }

    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        let cores = self.runtime.cores().map_err(words)?;
        let mut out = Vec::new();
        for s in systems {
            // The runtime's Setting(console, "Engine"): the stored choice, else the engine row's default.
            let chosen = self.runtime.settings_get(0, Scope::Console, Some(crate::consoles::settings_console(s)), crate::consoles::ENGINE_KEY).map_err(words)?;
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
        let sqlite = lent_sqlite()?;
        let runtime = runtime_on(&api, config, Some(&sqlite))?;
        let games_db = platform.file(FILE_GAMES).map_err(words)?;
        let library = Library::open(&platform, &games_db, &sqlite).map_err(words)?;
        let scraper = open_scraper(&platform, &runtime, &games_db.to_string_lossy(), &sqlite);
        Ok(ClientSource { runtime, platform, library, _sqlite: sqlite, scraper, walked: Mutex::new(None) })
    }
}

/// The scraper as a handle for finding pictures alone, with the player's folders as Mistress gives them; none where the
/// platform library could not open one, so the screens show generated cards.
fn open_scraper(platform: &Arc<Platform>, runtime: &Runtime, games_db: &str, sqlite: &Lent) -> Option<Scraper> {
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
    Some(Scraper { service: platform.scrape_service().ok()?, config, serial: Mutex::new(serial) })
}

/// A path with its `.` and `..` resolved by name alone, as .NET's GetFullPath resolves a folder link.
fn lexical(path: &std::path::Path) -> String {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out.to_string_lossy().into_owned()
}

/// A game's own crop, games.db's, read on the front end's SQLite; none where it has none.
pub fn game_crop(platform: &Arc<Platform>, path: &str) -> Result<Option<crate::display::Crop>, String> {
    let sqlite = lent_sqlite()?;
    let library = Library::open(platform, &platform.file(FILE_GAMES).map_err(words)?, &sqlite).map_err(words)?;
    Ok(crate::display::Crop::from_record(&library.get(&json!({ "what": "crop", "path": path })).map_err(words)?))
}

/// The system's soname of SQLite, which the Qt front end lends to the platform library and to the runtime (EmuSen_Qt.md §1.1.2).
pub const SQLITE: &str = "libsqlite3.so.0";

fn lent_sqlite() -> Result<Lent, String> {
    Lent::open(SQLITE).map_err(words)
}

/// A runtime on `config`, lent the front end's SQLite, so it keeps its state records (EmuSen_Qt.md §7.9): how every
/// model makes its runtime. Where SQLite cannot be loaded the runtime is lent none and games still open; the second value
/// then says that no state records are kept.
pub fn runtime(api: &Arc<emusen_crystal_client::Api>, config: &emusen_crystal_client::Config) -> Result<(Runtime, Option<String>), String> {
    runtime_lending(api, config, lent_sqlite())
}

/// As `runtime`, lent `sqlite` as loaded, or nothing where it failed to load.
pub fn runtime_lending(
    api: &Arc<emusen_crystal_client::Api>,
    config: &emusen_crystal_client::Config,
    sqlite: Result<Lent, String>,
) -> Result<(Runtime, Option<String>), String> {
    match sqlite {
        Ok(sqlite) => Ok((runtime_on(api, config, Some(&sqlite))?, None)),
        Err(why) => Ok((runtime_on(api, config, None)?, Some(format!("The states' records are not kept, since SQLite could not be loaded: {why}")))),
    }
}

fn runtime_on(api: &Arc<emusen_crystal_client::Api>, config: &emusen_crystal_client::Config, sqlite: Option<&Lent>) -> Result<Runtime, String> {
    // SAFETY: a Lent is a loader's handle never closed, so it outlives every runtime.
    let lent = sqlite.and_then(|s| unsafe { emusen_crystal_client::LentLibrary::from_handle(s.handle()) });
    Runtime::lent(api, config, lent).map_err(words)
}

/// The clients on the platform library, their runtime made on `runtime_config` with the cores folder `cores`, as every
/// model's runtime is, so the library lists the player's cores the session opens games on (EmuSen_Qt.md §7.9).
pub fn source_for(platform_library: &str, cores: &str) -> Result<ClientSource, String> {
    let platform = Platform::load(platform_library).map_err(words)?;
    ClientSource::open(platform_library, &runtime_config(&platform, cores)?)
}

/// The runtime's configuration a program's models share: the cores folder a test or a build names, with the player's
/// own from Galaxia beside it; the runtime's defaults with none named.
pub fn runtime_config(platform: &Platform, cores: &str) -> Result<emusen_crystal_client::Config, String> {
    if cores.is_empty() {
        return Ok(emusen_crystal_client::Config::default());
    }
    let player = platform.directory(emusen_platform_client::galaxia::consts::DIR_CORES).map_err(words)?;
    Ok(emusen_crystal_client::Config {
        cores: Some(vec![cores.to_owned()]),
        player_cores: Some(player.into_iter().map(|p| p.to_string_lossy().into_owned()).collect()),
        ..emusen_crystal_client::Config::default()
    })
}
