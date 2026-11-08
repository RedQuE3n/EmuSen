//! The `emusen.scrape` service: the process's one scraper, lent by its host or opened here - see EmuSen_LibraryTools.md §8.

use std::path::Path;
use crate::scrape::{answer, fail, failed, text};
use serde_json::Value;
use emusen_galaxia::models::AppSettings;
use emusen_galaxia::tree::{self, Directory};
use emusen_galaxia::{config, setting_folders};
use emusen_library::Library;
use emusen_scrape::{Error, Scraper, abi};
use emusen_sqlite::Sqlite;
use std::ffi::c_void;
use std::sync::{Arc, RwLock, RwLockReadGuard};

pub const SERVICE: &str = "emusen.scrape";
pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 0;

/// The host's theme loader: the media types its active theme draws for a system, by the length-query idiom.
pub type ThemeKinds = unsafe extern "C" fn(*mut c_void, *const u8, usize, *mut u8, usize) -> i64;

/// `emusen_scraping_lend_info`, as `emusen_platform.h` lays it out.
#[repr(C)]
pub struct Lend {
    pub size: u32,
    pub scraper: *mut Scraper,
    pub library: *mut Library,
    pub systems: *const u8,
    pub systems_len: usize,
    pub theme_kinds: Option<ThemeKinds>,
    pub ctx: *mut c_void,
}

/// `emusen_scrape_service.h`'s table, as the module that calls it declares it.
pub use emusen_scrape_module::ScrapeServiceV1;

/// The service's one table, for every shell.
pub static TABLE: ScrapeServiceV1 = ScrapeServiceV1 {
    size: std::mem::size_of::<ScrapeServiceV1>() as u32,
    get: service_get,
    change: service_change,
    start: service_start,
    progress: service_progress,
    wait: service_wait,
    pause: service_pause,
    cancel: service_cancel,
    library_get: service_library_get,
    games: service_games,
    theme_kinds: service_theme_kinds,
    last_error: crate::scrape::emusen_scrape_last_error,
};

/// Whose scraper the service speaks for.
enum Slot {
    /// Nothing lent and nothing opened yet.
    Empty,
    /// The host's own, while it lends it.
    Lent { scraper: *const Scraper, library: *const Library, systems: Vec<String>, theme: Option<(ThemeKinds, *mut c_void)> },
    /// The host scrapes, with no scraper open now: nothing is opened in its place.
    Withdrawn,
    /// This library's own, opened from the settings file in a process whose host lent nothing.
    Own { scraper: Box<Scraper>, library: &'static Library },
}

// SAFETY: a lent handle is this library's own Scraper or Library, kept alive by the host until a lend that waits on the lock.
unsafe impl Send for Slot {}
unsafe impl Sync for Slot {}

static SLOT: RwLock<Slot> = RwLock::new(Slot::Empty);

/// What a call is answered with: the scraper, its library and the systems a scan names.
struct Held<'a> {
    scraper: &'a Scraper,
    library: &'a Library,
    systems: Vec<String>,
    theme: Option<(ThemeKinds, *mut c_void)>,
}

fn every_system() -> Vec<String> {
    emusen_systems::table().map(|t| t.iter().map(|s| s.id.clone()).collect()).unwrap_or_default()
}

fn setting(value: &Option<String>) -> Option<String> {
    value.clone().filter(|s| !s.trim().is_empty())
}

/// The tree, the settings file and its ROM folder, as the library tool reads them.
fn settings_file() -> (tree::Tree, AppSettings, Option<String>) {
    let tree = tree::tree().clone();
    let (settings, _) = config::load_or_new::<AppSettings>(&tree, None, emusen_galaxia::models::APP_SETTINGS_FILE);
    let roms = setting_folders::checked(settings.rom_directory.as_deref(), &tree::user_home()).path;
    (tree, settings, roms)
}

/// The settings file's folders, as `MainWindow.NativeSettings` gives them; each system its own engine, since no runtime catalog is here.
fn own_settings() -> (String, String, String) {
    let (tree, settings, roms) = settings_file();
    let art = setting(&settings.artwork_directory).or_else(|| tree.directory(Directory::Artwork).map(|d| d.to_string_lossy().into_owned()));
    let cores: Vec<[String; 1]> = every_system().into_iter().map(|id| [id]).collect();
    let out = serde_json::json!({
        "roms": roms,
        "art": art,
        "esde_media": setting(&settings.esde_media_directory),
        "openemu": settings.open_emu_fallback,
        "user_agent": format!("EmuSen/{}", env!("CARGO_PKG_VERSION")),
        "developer_files": true,
        "cores": cores,
    })
    .to_string();
    let media = tree.directory(Directory::Media).unwrap_or_default().to_string_lossy().into_owned();
    let games = tree.directory(Directory::Library).unwrap_or_default().join(emusen_galaxia::stores::GAMES_DB).to_string_lossy().into_owned();
    (media, games, out)
}

fn open_own() -> Result<Slot, Error> {
    let beside = Path::new(&tree::base_directory()).join(emusen_sqlite::SHIPPED_NAME).to_string_lossy().into_owned();
    let sqlite = Arc::new(Sqlite::of_process(&[beside]).map_err(Error::Io)?);
    let (media, games, settings) = own_settings();
    let library: &'static Library = Box::leak(Box::new(Library::open(sqlite.clone(), &games).map_err(library_error)?));
    let scraper = Box::new(Scraper::open(sqlite, &media, Some(library), &[], &settings)?);
    Ok(Slot::Own { scraper, library })
}

fn library_error(e: emusen_library::Error) -> Error {
    match e {
        emusen_library::Error::Database(f) => Error::Database(f),
        emusen_library::Error::Busy => Error::Busy(e.words()),
        emusen_library::Error::Io(w) | emusen_library::Error::Newer(w) => Error::Io(w),
        other => Error::BadRequest(other.words()),
    }
}

/// The slot read, opening this library's own scraper first where nothing was lent or opened.
fn held() -> Result<RwLockReadGuard<'static, Slot>, Error> {
    {
        let slot = SLOT.read().unwrap_or_else(|p| p.into_inner());
        if !matches!(*slot, Slot::Empty) {
            return Ok(slot);
        }
    }
    {
        let mut slot = SLOT.write().unwrap_or_else(|p| p.into_inner());
        if matches!(*slot, Slot::Empty) {
            *slot = open_own()?;
        }
    }
    Ok(SLOT.read().unwrap_or_else(|p| p.into_inner()))
}

fn with<T>(call: impl FnOnce(Held<'_>) -> Result<T, Error>) -> Result<T, Error> {
    let slot = held()?;
    let held = match &*slot {
        Slot::Lent { scraper, library, systems, theme } => Held { scraper: unsafe { &**scraper }, library: unsafe { &**library }, systems: systems.clone(), theme: *theme },
        Slot::Own { scraper, library } => Held { scraper, library, systems: every_system(), theme: None },
        Slot::Withdrawn => return Err(Error::Busy("EmuSen's scraper is not open now; it opens with the library, or when Scraping is switched on in Preferences.".into())),
        Slot::Empty => unreachable!("held() fills an empty slot"),
    };
    call(held)
}

/// Lends the host's scraper to every shell, null withdrawing it; its handles live until the next lend returns (§8).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scraping_lend(lend: *const Lend) -> i32 {
    let next = if lend.is_null() {
        Slot::Withdrawn
    } else {
        let lend = unsafe { &*lend };
        if (lend.size as usize) < std::mem::size_of::<Lend>() {
            return fail(abi::NULL, "the lend is shorter than version 1.0's");
        }
        if lend.scraper.is_null() {
            Slot::Withdrawn
        } else if lend.library.is_null() {
            return fail(abi::NO_LIBRARY, "a scraper was lent without its library");
        } else {
            let systems = match unsafe { text(lend.systems, lend.systems_len) } {
                Ok("") => every_system(),
                Ok(t) => match serde_json::from_str::<Value>(t) {
                    Ok(Value::Array(ids)) => ids.iter().filter_map(Value::as_str).map(str::to_string).collect(),
                    _ => return fail(abi::BAD_REQUEST, "the lend's systems are not a list of system ids"),
                },
                Err(status) => return status,
            };
            Slot::Lent { scraper: lend.scraper, library: lend.library, systems, theme: lend.theme_kinds.map(|f| (f, lend.ctx)) }
        }
    };
    let mut slot = SLOT.write().unwrap_or_else(|p| p.into_inner());
    *slot = next;
    0
}

unsafe fn request<'a>(data: *const u8, len: usize) -> Result<&'a str, i64> {
    unsafe { text(data, len) }.map_err(|s| s as i64)
}

unsafe extern "C" fn service_get(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(11, r, out, out_len, || with(|h| h.scraper.get_request(r))) }
}

unsafe extern "C" fn service_change(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(12, r, out, out_len, || with(|h| h.scraper.change_request(r))) }
}

unsafe extern "C" fn service_start(r: *const u8, len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    match with(|h| h.scraper.start_request(r)) {
        Ok(n) => n as i64,
        Err(e) => failed(e) as i64,
    }
}

unsafe extern "C" fn service_progress(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(13, r, out, out_len, || with(|h| h.scraper.progress_request(r))) }
}

extern "C" fn service_wait(serial: u64, timeout_ms: u32) -> i64 {
    match with(|h| Ok(h.scraper.wait(serial, timeout_ms.min(250)))) {
        Ok(s) => s as i64,
        Err(e) => failed(e) as i64,
    }
}

extern "C" fn service_pause(paused: i32) -> i32 {
    with(|h| Ok(h.scraper.pause(paused != 0) as i32)).unwrap_or_else(failed)
}

extern "C" fn service_cancel() -> i32 {
    with(|h| Ok(h.scraper.cancel() as i32)).unwrap_or_else(failed)
}

unsafe extern "C" fn service_library_get(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(14, r, out, out_len, || with(|h| h.library.get_request(r).map_err(library_error))) }
}

/// The settings' ROM folder scanned on a database of its own, as `RomLibrary.ScanNative` does, each game with its ES-DE system.
fn games(h: &Held<'_>, filter: &str) -> Result<String, Error> {
    let (tree, _, roms) = settings_file();
    let roms = roms.unwrap_or_default();
    let scan = Library::open(h.scraper.sqlite(), ":memory:").map_err(library_error)?;
    let members = vec![
        ("roots", Value::Array(vec![Value::String(String::from(roms))])),
        ("systems", Value::Array(h.systems.iter().map(|s| Value::String(String::from(s))).collect())),
        ("packages", tree.directory(Directory::Packages).map_or(Value::Null, |d| Value::String(String::from(d.to_string_lossy())))),
        ("compressed", tree.directory(Directory::Compressed).map_or(Value::Null, |d| Value::String(String::from(d.to_string_lossy())))),
    ];
    let mut request = String::new();
    request.push_str(&serde_json::to_string(&Value::Object(members.into_iter().map(|(k, v)| (String::from(k), v)).collect())).expect("a value is JSON"));
    scan.scan_request(&request).map_err(library_error)?;
    let found = serde_json::from_str::<Value>(&scan.games_request(filter).map_err(library_error)?).map_err(|e| Error::BadRequest(format!("{e:?}")))?;
    let Value::Object(mut top) = found else { return Err(Error::BadRequest("The library's games are not an object.".into())) };
    for (k, v) in top.iter_mut() {
        if k != "games" {
            continue;
        }
        if let Value::Array(list) = v {
            for g in list.iter_mut() {
                let esde = g.get("system").and_then(Value::as_str).and_then(emusen_systems::find).map(|s| s.esde_system.clone()).filter(|e| !e.is_empty());
                if let Value::Object(fields) = g {
                    fields.insert(String::from("esde"), esde.map_or(Value::Null, |s| Value::String(String::from(s))));
                }
            }
        }
    }
    let mut out = String::new();
    out.push_str(&serde_json::to_string(&Value::Object(top)).expect("a value is JSON"));
    Ok(out)
}

unsafe extern "C" fn service_games(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(15, r, out, out_len, || with(|h| games(&h, r))) }
}

/// The host theme loader's kinds for a system, asked of it; NOT_SUPPORTED where the host loads no themes.
fn theme_kinds(h: &Held<'_>, system: &str) -> Result<String, Error> {
    let Some((call, ctx)) = h.theme else { return Err(Error::BadRequest("No theme loader is lent here.".into())) };
    let mut buffer = vec![0u8; 1024];
    loop {
        let n = unsafe { call(ctx, system.as_ptr(), system.len(), buffer.as_mut_ptr(), buffer.len()) };
        if n < 0 {
            return Err(Error::BadRequest(format!("The theme has no view for {system}.")));
        }
        if n as usize <= buffer.len() {
            return String::from_utf8(buffer[..n as usize].to_vec()).map_err(|_| Error::BadRequest("The theme's kinds are not UTF-8.".into()));
        }
        buffer.resize(n as usize, 0);
    }
}

unsafe extern "C" fn service_theme_kinds(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let system = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    match with(|h| Ok(h.theme.is_some())) {
        Ok(false) => return fail(abi::NOT_SUPPORTED, "no theme loader is lent here: the active theme's kinds are known only inside EmuSen, so name --kinds instead") as i64,
        Err(e) => return failed(e) as i64,
        Ok(true) => {}
    }
    unsafe { answer(16, system, out, out_len, || with(|h| theme_kinds(&h, system))) }
}

/// The `scrape` command as data for a window that fronts it, over the service's own table (EmuSen_LibraryTools.md §9).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scraping_command(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    let service = emusen_scrape_module::Service(&TABLE);
    unsafe {
        answer(17, r, out, out_len, || {
            emusen_scrape_module::ask(&service, r).map_err(|(status, words)| if status == abi::BUSY as i64 { Error::Busy(words) } else { Error::BadRequest(words) })
        })
    }
}

/// Whether this library opened a scraper of its own, for the tests that hold a lend to one per process.
pub fn opened_its_own() -> bool {
    matches!(*SLOT.read().unwrap_or_else(|p| p.into_inner()), Slot::Own { .. })
}
