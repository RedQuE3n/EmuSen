//! The `emusen.scrape` service: the process's one scraper, opened by its front end or on first use here - see EmuSen_LibraryTools.md §8.

use crate::games_service::{self, every_system, settings_file};
use emusen_galaxia::tree::Directory;
use emusen_scrape::{Error, Scraper, abi};
use emusen_sqlite::Sqlite;
use serde_json::Value;
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, RwLock, RwLockReadGuard};

pub const SERVICE: &str = "emusen.scrape";
pub const MAJOR: u32 = 2;
pub const MINOR: u32 = 0;

/// The front end's theme loader: the media types its active theme draws for a system, by the length-query idiom.
pub type ThemeKinds = unsafe extern "C" fn(*mut c_void, *const u8, usize, *mut u8, usize) -> i64;

/// `emusen_scraping_config`, as `emusen_platform.h` lays it out.
#[repr(C)]
pub struct Config {
    pub size: u32,
    pub root: *const u8,
    pub root_len: usize,
    pub games_db: *const u8,
    pub games_db_len: usize,
    pub sqlite: *mut c_void,
    pub embedded: *const u8,
    pub embedded_len: usize,
    pub settings: *const u8,
    pub settings_len: usize,
    pub systems: *const u8,
    pub systems_len: usize,
    pub theme_kinds: Option<ThemeKinds>,
    pub ctx: *mut c_void,
}

/// `emusen_scrape_service.h`'s table, as the module that calls it declares it.
pub use emusen_scrape_module::ScrapeServiceV2;

/// The service's one table, for front ends and every shell.
pub static TABLE: ScrapeServiceV2 = ScrapeServiceV2 {
    size: std::mem::size_of::<ScrapeServiceV2>() as u32,
    get: service_get,
    change: service_change,
    start: service_start,
    progress: service_progress,
    wait: service_wait,
    pause: service_pause,
    cancel: service_cancel,
    theme_kinds: service_theme_kinds,
    last_error: service_last_error,
};

/// Whose scraper the service speaks for.
enum Slot {
    /// Nothing opened yet: the first call opens one from the settings file.
    Empty,
    /// The front end closed its scraper, or one failed to open: nothing is opened in its place.
    Closed,
    /// The process's scraper, with the serial its open answered and the front end's theme loader.
    Open { serial: u64, scraper: Box<Scraper>, theme: Option<(ThemeKinds, *mut c_void)>, own: bool },
}

/// The slot and the serial the next open answers.
struct State {
    slot: Slot,
    serial: u64,
}

// SAFETY: the theme loader's context is the front end's, callable from any thread as emusen_platform.h requires.
unsafe impl Send for State {}
unsafe impl Sync for State {}

static STATE: RwLock<State> = RwLock::new(State { slot: Slot::Empty, serial: 0 });

thread_local! {
    static WAITING: crate::reply::Waiting = const { RefCell::new(None) };
}

fn fail(status: i32, words: impl Into<String>) -> i32 {
    crate::last_error::fail(crate::last_error::Slot::Scrape, status, emusen_scrape::redact::redact(&words.into()))
}

fn failed(error: Error) -> i32 {
    fail(error.status(), error.words())
}

unsafe fn text<'a>(data: *const u8, len: usize) -> Result<&'a str, i32> {
    unsafe { crate::reply::text(data, len, |status, words| fail(status, words)) }
}

unsafe fn bytes<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// The length-query idiom over an answer made once, in this layer's slot.
unsafe fn answer(call: u8, request: &str, out: *mut u8, len: usize, make: impl FnOnce() -> Result<String, Error>) -> i64 {
    unsafe { crate::reply::answer(&WAITING, call, request, out, len, make, failed) }
}

fn setting(value: &Option<String>) -> Option<String> {
    value.clone().filter(|s| !s.trim().is_empty())
}

/// The settings file's folders, as `MainWindow.NativeSettings` gives them; each system its own engine, since no runtime catalog is here.
fn own_settings() -> (String, String) {
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
    (tree.directory(Directory::Media).unwrap_or_default().to_string_lossy().into_owned(), out)
}

fn library_error(e: emusen_library::Error) -> Error {
    match e {
        emusen_library::Error::Database(f) => Error::Database(f),
        emusen_library::Error::Busy => Error::Busy(e.words()),
        emusen_library::Error::Io(w) | emusen_library::Error::Newer(w) => Error::Io(w),
        other => Error::BadRequest(other.words()),
    }
}

/// This library's own scraper, on the tree's store and games.db, in a process whose front end opened none.
fn open_own() -> Result<Box<Scraper>, Error> {
    let games = games_service::held().map_err(library_error)?;
    let (media, settings) = own_settings();
    Ok(Box::new(Scraper::open(games.sqlite, &media, Some(games.library), &[], &settings)?))
}

/// The state read, this library's own scraper opened first where nothing was opened or closed.
fn held() -> Result<RwLockReadGuard<'static, State>, Error> {
    {
        let state = STATE.read().unwrap_or_else(|p| p.into_inner());
        if !matches!(state.slot, Slot::Empty) {
            return Ok(state);
        }
    }
    {
        let mut state = STATE.write().unwrap_or_else(|p| p.into_inner());
        if matches!(state.slot, Slot::Empty) {
            let scraper = open_own()?;
            state.serial += 1;
            state.slot = Slot::Open { serial: state.serial, scraper, theme: None, own: true };
        }
    }
    Ok(STATE.read().unwrap_or_else(|p| p.into_inner()))
}

fn with<T>(call: impl FnOnce(&Scraper, Option<(ThemeKinds, *mut c_void)>) -> Result<T, Error>) -> Result<T, Error> {
    let state = held()?;
    match &state.slot {
        Slot::Open { scraper, theme, .. } => call(scraper, *theme),
        _ => Err(Error::Busy("EmuSen's scraper is not open now; it opens with the library, or when Scraping is switched on in Preferences.".into())),
    }
}

/// What an open was given, read before the scraper open now is closed.
struct Opening<'a> {
    root: &'a str,
    games_db: &'a str,
    settings: &'a str,
    embedded: &'a [u8],
    systems: Vec<String>,
    sqlite: Arc<Sqlite>,
    theme: Option<(ThemeKinds, *mut c_void)>,
}

unsafe fn opening<'a>(config: *const Config) -> Result<Opening<'a>, i32> {
    if config.is_null() {
        return Err(fail(abi::NULL, "the config is null"));
    }
    let config = unsafe { &*config };
    if (config.size as usize) < std::mem::size_of::<Config>() {
        return Err(fail(abi::NULL, "the config is shorter than version 1.0's"));
    }
    let root = unsafe { text(config.root, config.root_len) }?;
    if root.is_empty() {
        return Err(fail(abi::NULL, "no media store was named"));
    }
    let systems = match unsafe { text(config.systems, config.systems_len) }? {
        "" => every_system(),
        t => match serde_json::from_str::<Value>(t) {
            Ok(Value::Array(ids)) => ids.iter().filter_map(Value::as_str).map(str::to_string).collect(),
            _ => return Err(fail(abi::BAD_REQUEST, "the systems are not a list of system ids")),
        },
    };
    let sqlite = if config.sqlite.is_null() { games_service::process_sqlite() } else { unsafe { Sqlite::lent(config.sqlite) }.map(Arc::new) };
    Ok(Opening {
        root,
        games_db: unsafe { text(config.games_db, config.games_db_len) }?,
        settings: unsafe { text(config.settings, config.settings_len) }?,
        embedded: unsafe { bytes(config.embedded, config.embedded_len) },
        systems,
        sqlite: sqlite.map_err(|words| fail(abi::NO_SQLITE, words))?,
        theme: config.theme_kinds.map(|f| (f, config.ctx)),
    })
}

/// Opens the process's one scraper for its front end, the one before closed first; its serial, or a status.
///
/// # Safety
/// `config` valid for its `size` bytes and its texts for their lengths; `sqlite` a live handle of SQLite or null; the
/// theme loader callable from any thread with `ctx` until the scraper is closed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scraping_open(config: *const Config) -> i64 {
    let o = match unsafe { opening(config) } {
        Ok(o) => o,
        Err(status) => return status as i64,
    };
    let mut state = STATE.write().unwrap_or_else(|p| p.into_inner());
    state.slot = Slot::Closed;
    let library = if o.games_db.is_empty() {
        None
    } else {
        match games_service::name(o.sqlite.clone(), o.games_db, o.systems) {
            Ok(games) => Some(games.library),
            Err(e) => return failed(library_error(e)) as i64,
        }
    };
    match Scraper::open(o.sqlite, o.root, library, o.embedded, o.settings) {
        Ok(scraper) => {
            state.serial += 1;
            state.slot = Slot::Open { serial: state.serial, scraper: Box::new(scraper), theme: o.theme, own: false };
            state.serial as i64
        }
        Err(e) => failed(e) as i64,
    }
}

/// Closes the scraper an open answered `serial` for, or any with 0; 1 when one closed. The service then refuses rather than open its own.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_scraping_close(serial: u64) -> i32 {
    let mut state = STATE.write().unwrap_or_else(|p| p.into_inner());
    match state.slot {
        Slot::Open { serial: open, .. } if serial == 0 || serial == open => {
            state.slot = Slot::Closed;
            1
        }
        Slot::Empty if serial == 0 => {
            state.slot = Slot::Closed;
            0
        }
        _ => 0,
    }
}

/// The service's table, for a front end: the same one every shell is handed.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_scraping_service() -> *const ScrapeServiceV2 {
    &TABLE
}

unsafe extern "C" fn service_last_error(out: *mut u8, len: usize) -> i64 {
    unsafe { crate::last_error::copy_out(crate::last_error::Slot::Scrape, out, len) }
}

unsafe fn request<'a>(data: *const u8, len: usize) -> Result<&'a str, i64> {
    unsafe { text(data, len) }.map_err(|s| s as i64)
}

unsafe extern "C" fn service_get(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(1, r, out, out_len, || with(|s, _| s.get_request(r))) }
}

unsafe extern "C" fn service_change(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(2, r, out, out_len, || with(|s, _| s.change_request(r))) }
}

unsafe extern "C" fn service_start(r: *const u8, len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    match with(|s, _| s.start_request(r)) {
        Ok(n) => n as i64,
        Err(e) => failed(e) as i64,
    }
}

unsafe extern "C" fn service_progress(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    unsafe { answer(3, r, out, out_len, || with(|s, _| s.progress_request(r))) }
}

extern "C" fn service_wait(serial: u64, timeout_ms: u32) -> i64 {
    match with(|s, _| Ok(s.wait(serial, timeout_ms.min(250)))) {
        Ok(s) => s as i64,
        Err(e) => failed(e) as i64,
    }
}

extern "C" fn service_pause(paused: i32) -> i32 {
    with(|s, _| Ok(s.pause(paused != 0) as i32)).unwrap_or_else(failed)
}

extern "C" fn service_cancel() -> i32 {
    with(|s, _| Ok(s.cancel() as i32)).unwrap_or_else(failed)
}

/// The front end theme loader's kinds for a system, asked of it.
fn theme_kinds(call: ThemeKinds, ctx: *mut c_void, system: &str) -> Result<String, Error> {
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
    match with(|_, theme| Ok(theme.is_some())) {
        Ok(false) => return fail(abi::NOT_SUPPORTED, "no theme loader was named here: the active theme's kinds are known only inside EmuSen, so name --kinds instead") as i64,
        Err(e) => return failed(e) as i64,
        Ok(true) => {}
    }
    unsafe { answer(4, system, out, out_len, || with(|_, theme| theme.map_or(Err(Error::BadRequest("No theme loader was named here.".into())), |(f, ctx)| theme_kinds(f, ctx, system)))) }
}

/// The `scrape` command as data for a window that fronts it, over the services' own tables (EmuSen_LibraryTools.md §9).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scraping_command(r: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let r = match unsafe { request(r, len) } {
        Ok(r) => r,
        Err(s) => return s,
    };
    let service = emusen_scrape_module::Service { scrape: &TABLE, games: &games_service::TABLE };
    unsafe {
        answer(5, r, out, out_len, || {
            emusen_scrape_module::ask(&service, r).map_err(|(status, words)| if status == abi::BUSY as i64 { Error::Busy(words) } else { Error::BadRequest(words) })
        })
    }
}

/// Whether this library opened a scraper of its own, for the tests that hold a process to one scraper.
pub fn opened_its_own() -> bool {
    matches!(STATE.read().unwrap_or_else(|p| p.into_inner()).slot, Slot::Open { own: true, .. })
}
