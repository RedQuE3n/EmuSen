//! Settings ▸ Consoles' source over the frontend API (EmuSen_Qt.md §7.7.E): the runtime's systems, cores, settings
//! schema and values, firmware status and overview, and the card manager's list, each through `emusen-crystal-client`.

use crate::consoles::ConsoleSource;
use emusen_crystal_client::shapes::{CoreRow, FirmwareFile, FirmwareItem, SettingRow, SystemRow};
use emusen_crystal_client::consts::{CARDS_LIST, CARDS_NEW, CARDS_OPEN};
use emusen_crystal_client::{Api, Runtime, Scope};
use emusen_platform_client::galaxia::consts::DIR_CORES;
use emusen_platform_client::Platform;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Clone)]
pub struct ClientConsoles {
    pub runtime: Runtime,
}

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

struct CachedClients {
    library: String,
    platform: Arc<Platform>,
    api: Option<Arc<Api>>,
    consoles: Option<(String, Vec<Option<SystemTime>>, ClientConsoles)>,
}

struct SharedWorker {
    tx: Option<std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for SharedWorker {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(h) = self.handle.take() {
            if std::thread::current().id() != h.thread().id() {
                let _ = h.join();
            }
        }
    }
}

/// Long-lived shared `Platform`, `Api`, `ClientConsoles` runtime, and FIFO local worker reused across model calls.
#[derive(Clone, Default)]
pub struct ConsolesClients {
    inner: Arc<Mutex<Option<CachedClients>>>,
    worker: Arc<Mutex<Option<Arc<SharedWorker>>>>,
}

fn cores_mtimes(platform: &Platform, cores: &str) -> Vec<Option<SystemTime>> {
    let dir_mtime = |p: &std::path::Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let mut out = Vec::with_capacity(3);
    if !cores.is_empty() {
        out.push(dir_mtime(std::path::Path::new(cores)));
    }
    if let Ok(Some(player)) = platform.directory(DIR_CORES) {
        out.push(dir_mtime(&player));
        out.push(dir_mtime(&player.join("libretro")));
    }
    out
}

impl ConsolesClients {
    /// Queues `work` on this client pool's persistent FIFO background thread.
    pub fn spawn(&self, work: impl FnOnce() + Send + 'static) {
        let tx = {
            let mut w = self.worker.lock().unwrap_or_else(|p| p.into_inner());
            if w.is_none() {
                let (tx, rx) = std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
                let handle = std::thread::spawn(move || {
                    while let Ok(task) = rx.recv() {
                        task();
                    }
                });
                *w = Some(Arc::new(SharedWorker { tx: Some(tx), handle: Some(handle) }));
            }
            w.as_ref().and_then(|s| s.tx.clone())
        };
        if let Some(tx) = tx {
            let _ = tx.send(Box::new(work));
        }
    }
    /// Returns the shared `Arc<Platform>` for `platform_library`, loading it at most once per library path.
    pub fn platform(&self, platform_library: &str) -> Result<Arc<Platform>, String> {
        {
            let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(c) = guard.as_ref().filter(|c| c.library == platform_library) {
                return Ok(c.platform.clone());
            }
        }
        let platform = Platform::load(platform_library).map_err(words)?;
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(c) = guard.as_ref().filter(|c| c.library == platform_library) {
            return Ok(c.platform.clone());
        }
        *guard = Some(CachedClients { library: platform_library.to_owned(), platform: platform.clone(), api: None, consoles: None });
        Ok(platform)
    }

    /// Returns a shared `ClientConsoles` runtime, checking core-directory mtimes only when `check_dirs` is set.
    pub fn consoles(&self, platform_library: &str, cores: &str, check_dirs: bool) -> Result<ClientConsoles, String> {
        let platform = self.platform(platform_library)?;
        let mtimes = if check_dirs { Some(cores_mtimes(&platform, cores)) } else { None };
        let cached_api = {
            let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(c) = guard.as_ref().filter(|c| c.library == platform_library) {
                if let Some((ref c_cores, ref c_mtimes, ref consoles)) = c.consoles {
                    if c_cores == cores && mtimes.as_ref().is_none_or(|m| m == c_mtimes) {
                        return Ok(consoles.clone());
                    }
                }
                c.api.clone()
            } else {
                None
            }
        };
        let api = match cached_api {
            Some(api) => api,
            None => Api::load(platform_library).map_err(words)?,
        };
        let consoles = ClientConsoles::open_on(&platform, &api, cores)?;
        let fresh_mtimes = mtimes.unwrap_or_else(|| cores_mtimes(&platform, cores));
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(c) = guard.as_mut().filter(|c| c.library == platform_library) {
            c.api = Some(api);
            c.consoles = Some((cores.to_owned(), fresh_mtimes, consoles.clone()));
        }
        Ok(consoles)
    }
}

impl ClientConsoles {
    /// Opens a console runtime on an already-loaded `Platform` and `Api` handle.
    pub fn open_on(platform: &Platform, api: &Arc<Api>, cores: &str) -> Result<ClientConsoles, String> {
        let config = crate::clients::runtime_config(platform, cores)?;
        Ok(ClientConsoles { runtime: crate::clients::runtime(api, &config)?.0 })
    }

    /// The runtime opened on the platform library, with the cores folder a test or a build names (`clients::runtime_config`).
    pub fn open(platform_library: &str, cores: &str) -> Result<ClientConsoles, String> {
        let platform = Platform::load(platform_library).map_err(words)?;
        let api = Api::load(platform_library).map_err(words)?;
        Self::open_on(&platform, &api, cores)
    }

    /// A console's setting stored, or forgotten with none.
    pub fn set(&self, console: &str, key: &str, value: Option<&str>) -> Result<(), String> {
        self.runtime.settings_set(0, Scope::Console, Some(console), key, value).map_err(words)
    }

    /// A formatted card made at `path` by the card manager, which answers in words whether it was.
    pub fn new_card(&self, path: &str) -> Result<(), String> {
        let answer = self.runtime.cards_op(CARDS_NEW, &json!({ "path": path })).map_err(words)?.unwrap_or(Value::Null);
        if answer["done"] == true { Ok(()) } else { Err(answer["words"].as_str().unwrap_or("The card was not made.").to_owned()) }
    }

    /// One of the card manager's changes (copy, move, delete, import): whether it was done, and its words, or the
    /// refusal's.
    pub fn cards_change(&self, op: u32, args: &Value) -> (bool, String) {
        match self.runtime.cards_op(op, args) {
            Ok(Some(answer)) => (answer["done"] == true, answer["words"].as_str().unwrap_or_default().to_owned()),
            Ok(None) => (false, String::new()),
            Err(e) => (false, e.to_string()),
        }
    }
}

impl ConsoleSource for ClientConsoles {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.runtime.systems().map_err(words)
    }

    fn cores(&self) -> Vec<CoreRow> {
        self.runtime.cores().unwrap_or_default()
    }

    fn schema(&self, console: &str) -> Vec<SettingRow> {
        self.runtime.settings_schema(0, Some(console), None).unwrap_or_default()
    }

    fn setting(&self, console: &str, key: &str) -> Option<String> {
        self.runtime.settings_get(0, Scope::Console, Some(console), key).ok().flatten()
    }

    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        self.runtime.firmware_status(system).ok().flatten().map(|s| s.firmware)
    }

    fn overview(&self) -> Vec<FirmwareItem> {
        self.runtime.firmware_overview().map(|o| o.systems.into_iter().flat_map(|s| s.items).collect()).unwrap_or_default()
    }

    fn cards(&self, console: &str) -> Value {
        self.runtime.cards_op(CARDS_LIST, &json!({ "console": console })).ok().flatten().unwrap_or(Value::Null)
    }

    fn open_card(&self, path: &str) -> Value {
        self.runtime.cards_op(CARDS_OPEN, &json!({ "path": path })).ok().flatten().unwrap_or(Value::Null)
    }
}

/// The consoles as a file holds them in place of the runtime, as Mistress's recordings script their world: `systems`,
/// `cores`, each console's `schema` and stored `settings`, each system's `firmware`, the `overview`'s files and each
/// console's `cards` (EmuSen_Qt.md §7.7.E). A change is kept in the rows read, never written to the file.
pub struct ScriptedConsoles {
    world: Value,
}

impl ScriptedConsoles {
    pub fn open(path: &str) -> Result<ScriptedConsoles, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Ok(ScriptedConsoles { world: serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))? })
    }

    fn of<T: serde::de::DeserializeOwned + Default>(&self, value: &Value) -> T {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    /// Optional scripted audio output devices for tests, overriding the machine's live devices when present.
    pub fn outputs(&self) -> Option<Vec<String>> {
        self.world["outputs"].as_array().map(|list| list.iter().filter_map(Value::as_str).map(str::to_owned).collect())
    }
}

impl ConsoleSource for ScriptedConsoles {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.of(&self.world["systems"]))
    }

    fn cores(&self) -> Vec<CoreRow> {
        self.of(&self.world["cores"])
    }

    fn schema(&self, console: &str) -> Vec<SettingRow> {
        self.of(&self.world["schema"][console])
    }

    fn setting(&self, console: &str, key: &str) -> Option<String> {
        self.world["settings"][console][key].as_str().map(str::to_owned)
    }

    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        Some(self.of(&self.world["firmware"][system]))
    }

    fn overview(&self) -> Vec<FirmwareItem> {
        self.of(&self.world["overview"])
    }

    fn cards(&self, console: &str) -> Value {
        self.world["cards"][console].clone()
    }

    fn open_card(&self, path: &str) -> Value {
        self.world["opened"][path].clone()
    }
}
