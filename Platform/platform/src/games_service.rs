//! The `emusen.games` service: the process's games.db, shared with the scraper, and the ROM folder's games - see EmuSen_LibraryTools.md §8.

use crate::library::{answer, failed, text};
use emusen_galaxia::models::AppSettings;
use emusen_galaxia::tree::{self, Directory};
use emusen_galaxia::{config, setting_folders};
use emusen_library::{Error, Library};
use emusen_sqlite::Sqlite;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub const SERVICE: &str = "emusen.games";
pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 0;

/// `emusen_games_service.h`'s table, as the module that calls it declares it.
pub use emusen_scrape_module::GamesServiceV1;

/// The service's one table, for every shell.
pub static TABLE: GamesServiceV1 = GamesServiceV1 { size: std::mem::size_of::<GamesServiceV1>() as u32, get: service_get, games: service_games, last_error: crate::library::emusen_library_last_error };

/// The process's games.db: its file, the library on it, the SQLite it was opened with and the systems a scan names.
#[derive(Clone)]
pub(crate) struct Games {
    pub path: String,
    pub library: Arc<Library>,
    pub sqlite: Arc<Sqlite>,
    pub systems: Vec<String>,
}

static GAMES: RwLock<Option<Games>> = RwLock::new(None);

/// Every system of the table, for a scan whose front end named none.
pub(crate) fn every_system() -> Vec<String> {
    emusen_systems::table().map(|t| t.iter().map(|s| s.id.clone()).collect()).unwrap_or_default()
}

/// The tree, the settings file and its ROM folder, as the library tool reads them.
pub(crate) fn settings_file() -> (tree::Tree, AppSettings, Option<String>) {
    let tree = tree::tree().clone();
    let (settings, _) = config::load_or_new::<AppSettings>(&tree, None, emusen_galaxia::models::APP_SETTINGS_FILE);
    let roms = setting_folders::checked(settings.rom_directory.as_deref(), &tree::user_home()).path;
    (tree, settings, roms)
}

/// SQLite as this process holds it, else the copy shipped beside the program.
pub(crate) fn process_sqlite() -> Result<Arc<Sqlite>, String> {
    let beside = Path::new(&tree::base_directory()).join(emusen_sqlite::SHIPPED_NAME).to_string_lossy().into_owned();
    Sqlite::of_process(&[beside]).map(Arc::new)
}

/// The games.db a front end names: the one open on that file kept, else opened and made the process's.
pub(crate) fn name(sqlite: Arc<Sqlite>, path: &str, systems: Vec<String>) -> Result<Games, Error> {
    let mut slot = GAMES.write().unwrap_or_else(|p| p.into_inner());
    let games = match slot.take() {
        Some(open) if open.path == path => Games { systems, ..open },
        _ => Games { path: path.to_string(), library: Arc::new(Library::open(sqlite.clone(), path)?), sqlite, systems },
    };
    *slot = Some(games.clone());
    Ok(games)
}

/// The process's games.db, the tree's opened first where no front end named one.
pub(crate) fn held() -> Result<Games, Error> {
    if let Some(games) = GAMES.read().unwrap_or_else(|p| p.into_inner()).clone() {
        return Ok(games);
    }
    let sqlite = process_sqlite().map_err(Error::Io)?;
    let path = tree::tree().directory(Directory::Library).unwrap_or_default().join(emusen_galaxia::stores::GAMES_DB).to_string_lossy().into_owned();
    let mut slot = GAMES.write().unwrap_or_else(|p| p.into_inner());
    if slot.is_none() {
        *slot = Some(Games { library: Arc::new(Library::open(sqlite.clone(), &path)?), path, sqlite, systems: every_system() });
    }
    Ok(slot.clone().expect("filled above"))
}

unsafe extern "C" fn service_get(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { text(r, len) } {
        Ok(r) => r,
        Err(s) => return s as i64,
    };
    unsafe { answer(30, r, out, out_len, || held()?.library.get_request(r)) }
}

/// The settings' ROM folder scanned on a database of its own, as `RomLibrary.ScanNative` does, each game with its ES-DE system.
fn games(held: &Games, filter: &str) -> Result<String, Error> {
    let (tree, _, roms) = settings_file();
    let scan = Library::open(held.sqlite.clone(), ":memory:")?;
    let folder = |d: Directory| tree.directory(d).map_or(Value::Null, |d| Value::String(d.to_string_lossy().into_owned()));
    let request = serde_json::json!({ "roots": [roms.unwrap_or_default()], "systems": held.systems, "packages": folder(Directory::Packages), "compressed": folder(Directory::Compressed) });
    scan.scan_request(&request.to_string())?;
    let mut found = serde_json::from_str::<Value>(&scan.games_request(filter)?).map_err(|e| Error::BadRequest(e.to_string()))?;
    let Some(list) = found.get_mut("games").and_then(Value::as_array_mut) else { return Err(Error::BadRequest("The library's games are not a list.".into())) };
    for g in list.iter_mut() {
        let esde = g.get("system").and_then(Value::as_str).and_then(emusen_systems::find).map(|s| s.esde_system.clone()).filter(|e| !e.is_empty());
        if let Value::Object(fields) = g {
            fields.insert("esde".into(), esde.map_or(Value::Null, Value::String));
        }
    }
    Ok(found.to_string())
}

unsafe extern "C" fn service_games(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { text(r, len) } {
        Ok(r) => r,
        Err(s) => return s as i64,
    };
    match held() {
        Ok(h) => unsafe { answer(31, r, out, out_len, || games(&h, r)) },
        Err(e) => failed(e) as i64,
    }
}
