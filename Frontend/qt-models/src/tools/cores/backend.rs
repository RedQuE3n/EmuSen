//! What Cores asks of the back end, as one trait: the runtime's `cores`, `systems`, settings, `core_approve`,
//! `core_revoke`, `firmware_status` and `rescan`, and the core tool's jobs, pulled and drained, never pushed
//! (EmuSen_Qt.md §1.1). `Clients` answers over the two client crates; `Scripted` answers from a file that holds a world,
//! as Mistress's recordings of Cores script theirs (EmuSen_Qt.md §7.7).

use emusen_crystal_client::shapes::{CoreRow, FirmwareFile, SystemRow};
use emusen_crystal_client::{Runtime, Scope};
use emusen_platform_client::galaxia::consts::DIR_CORES;
use emusen_platform_client::tools::ToolJob;
use emusen_platform_client::Platform;
use serde_json::Value;
use std::sync::Arc;

/// A running job of the core tool: its events taken without waiting, true once it has ended and all are taken.
pub trait Job {
    fn drain(&mut self, out: &mut Vec<Value>) -> Result<bool, String>;
}

/// The back end as Cores reads and changes it.
pub trait Backend {
    fn systems(&self) -> Result<Vec<SystemRow>, String>;
    fn cores(&self) -> Result<Vec<CoreRow>, String>;
    /// A console's stored engine, none where it stores none.
    fn engine(&self, console: &str) -> Option<String>;
    fn set_engine(&self, console: &str, engine: &str) -> Result<(), String>;
    fn approve(&self, sha256: &str) -> Result<(), String>;
    fn revoke(&self, sha256: &str) -> Result<(), String>;
    /// A system's firmware files and their status; none where the runtime cannot say.
    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>>;
    fn rescan(&self);
    fn start(&self, request: &Value) -> Result<Box<dyn Job>, String>;
    /// The cores folder's disk: its free and total bytes.
    fn space(&self) -> Option<(u64, u64)>;
    /// A core library's size in bytes.
    fn size(&self, library: &str) -> Option<u64>;
}

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The back end through the frontend API and the platform library.
pub struct Clients {
    pub runtime: Runtime,
    pub platform: Arc<Platform>,
}

struct Running(ToolJob);

impl Job for Running {
    fn drain(&mut self, out: &mut Vec<Value>) -> Result<bool, String> {
        self.0.drain(out).map_err(words)
    }
}

impl Backend for Clients {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.runtime.systems().map_err(words)
    }

    fn cores(&self) -> Result<Vec<CoreRow>, String> {
        self.runtime.cores().map_err(words)
    }

    fn engine(&self, console: &str) -> Option<String> {
        self.runtime.settings_get(0, Scope::Console, Some(console), super::ENGINE_KEY).ok().flatten().filter(|e| !e.is_empty())
    }

    fn set_engine(&self, console: &str, engine: &str) -> Result<(), String> {
        self.runtime.settings_set(0, Scope::Console, Some(console), super::ENGINE_KEY, Some(engine)).map_err(words)
    }

    fn approve(&self, sha256: &str) -> Result<(), String> {
        self.runtime.core_approve(sha256).map_err(words)
    }

    fn revoke(&self, sha256: &str) -> Result<(), String> {
        self.runtime.core_revoke(sha256).map_err(words)
    }

    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        self.runtime.firmware_status(system).ok().flatten().map(|s| s.firmware)
    }

    fn rescan(&self) {
        let _ = self.runtime.rescan();
    }

    fn start(&self, request: &Value) -> Result<Box<dyn Job>, String> {
        ToolJob::start(&self.platform, request).map(|j| Box::new(Running(j)) as Box<dyn Job>).map_err(words)
    }

    fn space(&self) -> Option<(u64, u64)> {
        let folder = self.platform.directory(DIR_CORES).ok()??;
        disk(&folder)
    }

    fn size(&self, library: &str) -> Option<u64> {
        std::fs::metadata(library).ok().map(|m| m.len())
    }
}

/// The free and total bytes of the disk holding `path`, or of its nearest folder that exists.
pub fn disk(path: &std::path::Path) -> Option<(u64, u64)> {
    let at = path.ancestors().find(|p| p.exists())?;
    let text = std::ffi::CString::new(at.as_os_str().as_encoded_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a NUL-terminated path and a zeroed struct the call fills.
    if unsafe { libc::statvfs(text.as_ptr(), &mut stat) } != 0 {
        return None;
    }
    let unit = stat.f_frsize as u64;
    Some((stat.f_bavail as u64 * unit, stat.f_blocks as u64 * unit))
}

/// A world held in a file: the runtime's listings and settings, the disk, and each job's events and whether it ends.
pub struct Scripted {
    world: Value,
    settings: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

struct Script {
    events: Vec<Value>,
    ends: bool,
}

impl Job for Script {
    fn drain(&mut self, out: &mut Vec<Value>) -> Result<bool, String> {
        out.append(&mut self.events);
        Ok(self.ends)
    }
}

impl Scripted {
    /// The world in the file at `path`: `systems` and `cores` as the runtime lists them, `settings` by
    /// `<console>/<key>`, `space` as `[free, total]`, `sizes` by library, and `jobs` by the request's `op` (`*` for any
    /// other) as `{"events": [...], "ends": bool}`.
    pub fn open(path: &str) -> Result<Scripted, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let world: Value = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
        let settings = world.get("settings").and_then(Value::as_object).map(|m| m.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned()))).collect()).unwrap_or_default();
        Ok(Scripted { world, settings: std::sync::Mutex::new(settings) })
    }

    fn list(&self, name: &str) -> Value {
        self.world.get(name).cloned().unwrap_or(Value::Array(Vec::new()))
    }
}

impl Backend for Scripted {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        serde_json::from_value(self.list("systems")).map_err(words)
    }

    fn cores(&self) -> Result<Vec<CoreRow>, String> {
        serde_json::from_value(self.list("cores")).map_err(words)
    }

    fn engine(&self, console: &str) -> Option<String> {
        self.settings.lock().ok()?.get(&format!("{console}/{}", super::ENGINE_KEY)).cloned()
    }

    fn set_engine(&self, console: &str, engine: &str) -> Result<(), String> {
        self.settings.lock().map_err(words)?.insert(format!("{console}/{}", super::ENGINE_KEY), engine.to_owned());
        Ok(())
    }

    fn approve(&self, _sha256: &str) -> Result<(), String> {
        Ok(())
    }

    fn revoke(&self, _sha256: &str) -> Result<(), String> {
        Ok(())
    }

    fn firmware(&self, _system: &str) -> Option<Vec<FirmwareFile>> {
        None
    }

    fn rescan(&self) {}

    fn start(&self, request: &Value) -> Result<Box<dyn Job>, String> {
        let op = request.get("op").and_then(Value::as_str).unwrap_or_default();
        let jobs = self.world.get("jobs");
        let job = jobs.and_then(|j| j.get(op)).or_else(|| jobs.and_then(|j| j.get("*"))).ok_or_else(|| format!("no job {op}"))?;
        let events = job.get("events").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(Box::new(Script { events, ends: job.get("ends").and_then(Value::as_bool).unwrap_or(true) }))
    }

    fn space(&self) -> Option<(u64, u64)> {
        let s = self.world.get("space")?.as_array()?;
        Some((s.first()?.as_u64()?, s.get(1)?.as_u64()?))
    }

    fn size(&self, library: &str) -> Option<u64> {
        self.world.get("sizes")?.get(library)?.as_u64()
    }
}
