//! `emusen-scrape`: scraping below every frontend, a component of its own beside the library and Crystal and never in
//! either (EmuSen_Backend.md D13). A handle is one media store and its `media.db`: it keeps what ScreenScraper and
//! OpenEmu's sources answered, runs the queue on the player's word alone, honours the quota, and answers where a game's
//! pictures are. Requests and answers are JSON, as `emusen_scrape.h` describes them; this crate decides, the platform
//! library's C layer only converts. See EmuSen_Backend.md §2.4 and §30.

pub mod abi;
pub mod answer;
pub mod artwork;
pub mod client;
pub mod clock;
pub mod consoles;
pub mod credentials;
mod json;
pub mod kinds;
pub mod layout;
pub mod media;
pub mod metadata;
pub mod openemu;
pub mod quota;
pub mod redact;
pub mod rules;
pub mod run;
pub mod store;
pub mod zone;

use clock::Clock;
use emusen_dotnet::json::Value;
use emusen_sqlite::{Failure, Sqlite};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

/// Why a call failed; each is one of the header's statuses, with its words, always redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Newer(String),
    Database(Failure),
    BadRequest(String),
    Busy(String),
    Io(String),
    NoLibrary,
}

impl Error {
    pub fn status(&self) -> i32 {
        match self {
            Error::Newer(_) => abi::NEWER_FILE,
            Error::Database(_) => abi::DATABASE,
            Error::BadRequest(_) => abi::BAD_REQUEST,
            Error::Busy(_) => abi::BUSY,
            Error::Io(_) => abi::IO,
            Error::NoLibrary => abi::NO_LIBRARY,
        }
    }

    pub fn words(&self) -> String {
        redact::redact(&match self {
            Error::Newer(w) | Error::BadRequest(w) | Error::Busy(w) | Error::Io(w) => w.clone(),
            Error::Database(f) => f.message.clone(),
            Error::NoLibrary => "No library was lent to the scraper.".to_string(),
        })
    }
}

impl From<Failure> for Error {
    fn from(failure: Failure) -> Self {
        Error::Database(failure)
    }
}

/// The seams of §30.3 that tests alone reach.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Test {
    pub connect: Option<SocketAddr>,
    pub roots: Option<String>,
    pub clock: Option<i64>,
    pub offline: bool,
}

/// The settings a host gives at open and when Preferences closes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub roms: Option<String>,
    pub art: Option<String>,
    pub esde_media: Option<String>,
    pub openemu: bool,
    pub user_agent: Option<String>,
    /// Whether the developer file may be read from the machine's config directory; a test harness says no.
    pub developer_files: bool,
    /// The systems each engine runs together, by id: the runtime's fact (EmuSen_Backend.md §30.12).
    pub cores: Vec<Vec<String>>,
    /// Each packaged game's file and its package's folder, as the host's library scan found them (EmuSen_GamePackages.md §18.3).
    pub packages: Vec<(String, String)>,
    pub test: Test,
}

impl Settings {
    fn read(text: &str) -> Result<Settings, Error> {
        if text.trim().is_empty() {
            return Ok(Settings { openemu: true, developer_files: true, ..Settings::default() });
        }
        Settings::merge(Settings { openemu: true, developer_files: true, ..Settings::default() }, &json::parse(text)?)
    }

    /// The members a request names laid over what was set.
    fn merge(mut s: Settings, r: &Value) -> Result<Settings, Error> {
        if r.get("roms").is_some() {
            s.roms = json::text_or_none(r, "roms")?;
        }
        if r.get("art").is_some() {
            s.art = json::text_or_none(r, "art")?;
        }
        if r.get("esde_media").is_some() {
            s.esde_media = json::text_or_none(r, "esde_media")?;
        }
        s.openemu = json::flag(r, "openemu", s.openemu)?;
        if r.get("user_agent").is_some() {
            s.user_agent = json::text_or_none(r, "user_agent")?;
        }
        s.developer_files = json::flag(r, "developer_files", s.developer_files)?;
        if let Some(cores) = r.get("cores") {
            s.cores = match cores {
                Value::Array(groups) => groups
                    .iter()
                    .map(|g| match g {
                        Value::Array(ids) => ids.iter().map(|i| i.as_str().map(str::to_string).ok_or_else(|| Error::BadRequest("The request's cores holds a system that is not text.".into()))).collect(),
                        _ => Err(Error::BadRequest("The request's cores is not a list of lists.".into())),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                Value::Null => Vec::new(),
                _ => return Err(Error::BadRequest("The request's cores is not a list of lists.".into())),
            };
        }
        if let Some(p) = r.get("packages") {
            s.packages = match p {
                Value::Object(members) => members
                    .iter()
                    .map(|(k, v)| v.as_str().map(|f| (k.text.clone(), f.to_string())).ok_or_else(|| Error::BadRequest("The request's packages holds a folder that is not text.".into())))
                    .collect::<Result<Vec<_>, _>>()?,
                Value::Null => Vec::new(),
                _ => return Err(Error::BadRequest("The request's packages is not a map of game files to folders.".into())),
            };
        }
        if let Some(t) = r.get("test") {
            if !t.is_null() {
                let connect = json::text_or_none(t, "connect")?.map(|c| c.parse::<SocketAddr>().map_err(|_| Error::BadRequest("The request's test.connect is not an address.".into()))).transpose()?;
                s.test = Test { connect, roots: json::text_or_none(t, "roots")?, clock: json::integer_or_none(t, "clock")?, offline: json::flag(t, "offline", false)? };
            }
        }
        Ok(s)
    }

    /// The package a library file is the game of, as the host's scan found it; none for a file no package claims.
    pub fn package_of(&self, rom: &str) -> Option<&str> {
        self.packages.iter().rev().find(|(game, _)| game == rom).map(|(_, folder)| folder.as_str())
    }

    /// The game's folder below its console's, by the ROM folder the settings name.
    pub fn folder_of(&self, rom: &str) -> String {
        emusen_galaxia::game_folders::of(self.roms.as_deref(), rom)
    }
}

/// One media store, its `media.db`, its quota and what its host lent it.
pub struct Scraper {
    store: Mutex<store::Store>,
    pub clock: Clock,
    pub quota: quota::Manager,
    settings: RwLock<Settings>,
    embedded: Vec<u8>,
    pub(crate) sqlite: Arc<Sqlite>,
    library: Option<&'static emusen_library::Library>,
    pub(crate) runs: run::Runs,
    art: RwLock<artwork::Index>,
    root: String,
}

impl Drop for Scraper {
    fn drop(&mut self) {
        self.stop_workers();
        self.stop_failover();
    }
}

impl Scraper {
    /// Opens the store at `root` with the SQLite and the library lent for the handle's life. The library, when given,
    /// must outlive the handle; the C layer's caller promises it.
    pub fn open(sqlite: Arc<Sqlite>, root: &str, library: Option<&'static emusen_library::Library>, embedded: &[u8], settings: &str) -> Result<Scraper, Error> {
        let settings = Settings::read(settings)?;
        let clock = match settings.test.clock {
            Some(start) => Clock::test(start),
            None => Clock::System,
        };
        let store = store::Store::open(sqlite.clone(), root)?;
        let quota = quota::Manager::new(&store, &clock)?;
        let art = artwork::Index::scan(settings.art.as_deref(), &consoles::cores(&settings.cores));
        let scraper = Scraper {
            store: Mutex::new(store),
            clock,
            quota,
            settings: RwLock::new(settings),
            embedded: embedded.to_vec(),
            sqlite,
            library,
            runs: run::Runs::default(),
            art: RwLock::new(art),
            root: root.to_string(),
        };
        scraper.put_in_folders()?;
        Ok(scraper)
    }

    pub(crate) fn store(&self) -> MutexGuard<'_, store::Store> {
        self.store.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn root(&self) -> String {
        self.root.clone()
    }

    /// The failover's own folder, apart from the player's art: `<store>/openemu/<console>/`.
    pub fn openemu_root(&self) -> String {
        emusen_galaxia::dotnet_path::combine(emusen_galaxia::dotnet_path::Style::HOST, &self.root, "openemu")
    }

    pub fn art(&self) -> std::sync::RwLockReadGuard<'_, artwork::Index> {
        self.art.read().unwrap_or_else(|p| p.into_inner())
    }

    pub(crate) fn add_art(&self, console: Option<String>, stem: &str, file: &str) {
        self.art.write().unwrap_or_else(|p| p.into_inner()).add(console, stem, file);
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap_or_else(|p| p.into_inner()).clone()
    }

    pub fn library(&self) -> Result<&'static emusen_library::Library, Error> {
        self.library.ok_or(Error::NoLibrary)
    }

    /// The store's one layout step, taken once the settings name the ROM folder that names each game's folder.
    fn put_in_folders(&self) -> Result<usize, Error> {
        let settings = self.settings();
        if settings.roms.as_deref().is_none_or(|r| emusen_dotnet::text::is_blank(Some(r))) {
            return Ok(0);
        }
        let store = self.store();
        layout::put_in_folders(&store, &|p| settings.folder_of(p), self.clock.now())
    }

    pub fn developer(&self) -> Option<credentials::Developer> {
        let embedded = (!self.embedded.is_empty()).then_some(self.embedded.as_slice());
        credentials::developer(self.settings().developer_files, embedded)
    }

    /// `emusen_scrape_get`'s request and answer.
    pub fn get_request(&self, request: &str) -> Result<String, Error> {
        reads::get(self, &json::parse(request)?)
    }

    /// `emusen_scrape_change`'s request and answer.
    pub fn change_request(&self, request: &str) -> Result<String, Error> {
        changes::change(self, &json::parse(request)?)
    }

    pub(crate) fn apply_settings(&self, r: &Value) -> Result<(), Error> {
        let merged = Settings::merge(self.settings(), r)?;
        let art = artwork::Index::scan(merged.art.as_deref(), &consoles::cores(&merged.cores));
        *self.settings.write().unwrap_or_else(|p| p.into_inner()) = merged;
        *self.art.write().unwrap_or_else(|p| p.into_inner()) = art;
        self.quota.settings_changed();
        self.put_in_folders()?;
        Ok(())
    }
}

mod changes;
mod reads;
mod requests;
