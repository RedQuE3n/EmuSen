//! Scraping through the emusen.scrape service 2.0, its one surface: the process's one scraper opened and closed here,
//! and the table emusen_scraping_service hands out wrapped once, its failures read with the table's own last_error.
//! Requests and answers are EmuSen_Backend.md §30.3's JSON, carried as `serde_json::Value` until a screen types them.

use crate::error::Result;
use crate::out::{json, text, to_json, words};
use crate::platform::{Lent, Platform};
use crate::raw;
use serde_json::Value;
use std::sync::Arc;

/// What the process's scraper is opened with; none of the theme loader yet (EmuSen_Qt.md §1.1.2).
#[derive(Clone, Debug, Default)]
pub struct ScraperConfig {
    /// The media store's folder, media.db in it.
    pub root: String,
    /// games.db, the one the library handle opens; none for no library.
    pub games_db: Option<String>,
    /// The program's embedded developer blob, or none.
    pub embedded: Vec<u8>,
    /// The scraper's settings as JSON.
    pub settings: Value,
    /// The systems the build opens; empty for every system of the table.
    pub systems: Vec<String>,
}

/// The emusen.scrape table, for the process's one scraper.
pub struct ScrapeService {
    platform: Arc<Platform>,
    table: &'static raw::emusen_scrape_service_v2,
}

/// The table's entry `$f` when the provider's table reaches it and has it.
macro_rules! entry {
    ($self:ident, $f:ident) => {{
        let end = std::mem::offset_of!(raw::emusen_scrape_service_v2, $f) + std::mem::size_of::<usize>();
        match if $self.table.size as usize >= end { $self.table.$f } else { None } {
            Some(f) => f,
            None => {
                let words = format!("The scrape service's table has no {}.", stringify!($f));
                return Err(crate::error::status(raw::EMUSEN_SCRAPE_NOT_SUPPORTED, words));
            }
        }
    }};
}

impl Platform {
    /// Opens the process's one scraper, closing any before it; its serial, above zero.
    pub fn scraper_open(self: &Arc<Platform>, config: &ScraperConfig, sqlite: Option<&Lent>) -> Result<u64> {
        let games_db = config.games_db.clone().unwrap_or_default();
        let settings = to_json(&config.settings);
        let systems = to_json(&config.systems);
        let (e, el) = if config.embedded.is_empty() { (std::ptr::null(), 0) } else { (config.embedded.as_ptr(), config.embedded.len()) };
        let raw_config = raw::emusen_scraping_config {
            size: std::mem::size_of::<raw::emusen_scraping_config>() as u32,
            root: config.root.as_ptr(),
            root_len: config.root.len(),
            games_db: text(Some(&games_db)).0,
            games_db_len: games_db.len(),
            sqlite: sqlite.map_or(std::ptr::null_mut(), Lent::handle),
            embedded: e,
            embedded_len: el,
            settings: settings.as_ptr(),
            settings_len: settings.len(),
            systems: systems.as_ptr(),
            systems_len: systems.len(),
            theme_kinds: None,
            ctx: std::ptr::null_mut(),
        };
        let n = unsafe { self.lib.emusen_scraping_open(&raw_config) };
        let service = self.scrape_service()?;
        crate::out::check(n, || service.words()).map(|n| n as u64)
    }

    /// Closes the scraper of `serial` while it is still the one open (true), or any with 0.
    pub fn scraper_close(&self, serial: u64) -> Result<bool> {
        self.status(unsafe { self.lib.emusen_scraping_close(serial) }.into()).map(|n| n == 1)
    }

    /// The emusen.scrape table, which lives as long as the library is loaded.
    pub fn scrape_service(self: &Arc<Platform>) -> Result<ScrapeService> {
        let table = unsafe { self.lib.emusen_scraping_service() };
        let table = unsafe { table.as_ref() }.ok_or_else(|| crate::error::status(raw::EMUSEN_SCRAPE_NULL, self.last_error()))?;
        Ok(ScrapeService { platform: self.clone(), table })
    }

    /// The scrape command as data, for a window that fronts it: {"args", "paths", "sources"}.
    pub fn scrape_command(self: &Arc<Platform>, request: &Value) -> Result<Value> {
        let request = to_json(request);
        let service = self.scrape_service()?;
        let bytes = crate::out::answer(
            |o, l| unsafe { self.lib.emusen_scraping_command(request.as_ptr(), request.len(), o, l) },
            || service.words(),
        )?;
        json("scraping_command", &bytes)
    }
}

impl ScrapeService {
    fn words(&self) -> String {
        match self.table.last_error {
            Some(f) => words(|o, l| unsafe { f(o, l) }),
            None => String::new(),
        }
    }

    fn ask(&self, name: &'static str, request: &Value, f: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64) -> Result<Value> {
        let request = to_json(request);
        let bytes = crate::out::answer(|o, l| unsafe { f(request.as_ptr(), request.len(), o, l) }, || self.words())?;
        json(name, &bytes)
    }

    /// One read, {"what": ...}.
    pub fn get(&self, request: &Value) -> Result<Value> {
        let f = entry!(self, get);
        self.ask("get", request, f)
    }

    /// One change, {"op": ...}; a change that asks the server blocks for that one request.
    pub fn change(&self, request: &Value) -> Result<Value> {
        let f = entry!(self, change);
        self.ask("change", request, f)
    }

    /// The games queued and a run started.
    pub fn start(&self, request: &Value) -> Result<i64> {
        let f = entry!(self, start);
        let request = to_json(request);
        crate::out::check(unsafe { f(request.as_ptr(), request.len()) }, || self.words())
    }

    /// The run as it stands, its results after serial `since`.
    pub fn progress(&self, since: u64) -> Result<Value> {
        let f = entry!(self, progress);
        self.ask("progress", &serde_json::json!({ "since": since }), f)
    }

    /// Waits at most `timeout_ms`, held to 250 ms by the service, for the run's serial to pass `serial`; the serial then.
    pub fn wait(&self, serial: u64, timeout_ms: u32) -> Result<u64> {
        let f = entry!(self, wait);
        crate::out::check(unsafe { f(serial, timeout_ms) }, || self.words()).map(|n| n as u64)
    }

    /// The run paused or resumed; true when that changed it.
    pub fn pause(&self, paused: bool) -> Result<bool> {
        let f = entry!(self, pause);
        crate::out::check(unsafe { f(paused as i32) }.into(), || self.words()).map(|n| n == 1)
    }

    /// The run cancelled; true when one was going.
    pub fn cancel(&self) -> Result<bool> {
        let f = entry!(self, cancel);
        crate::out::check(unsafe { f() }.into(), || self.words()).map(|n| n == 1)
    }

    /// The platform library this table is served by.
    pub fn platform(&self) -> &Arc<Platform> {
        &self.platform
    }
}
