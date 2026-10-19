//! One library on the core ABI v1: loaded once per path for the process and never unloaded, its required exports
//! resolved, and checked as `EmuSen_CoreAPI.md` §7.2's rule 4 asks before a host calls it (EmuSen_Backend.md §27.3).

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use emusen_native::core::schema;
use emusen_native::core::sys::{CAPABILITY_NAMES, EXPORTS, status};
use emusen_native::json::{self, Value};

use crate::exports::Exports;

/// A descriptor larger than this is refused at load (§7.2).
pub const MAX_DESCRIPTOR_BYTES: i64 = 1 << 20;

/// The host obligations this host meets, the names core info's `host_requires` may use.
pub const HOST_OBLIGATIONS: &[&str] = &["present"];

/// Why a library could not be loaded at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The system's loader refused the file, in its words.
    Unloadable(String),
    /// A required export is absent; `abi` is what `emusen_core_abi_version` answered, where the library has one.
    Missing { export: String, abi: Option<u32> },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Unloadable(e) => write!(f, "could not be loaded: {e}"),
            LoadError::Missing { export, .. } => write!(f, "lacks the required export {export}"),
        }
    }
}

impl LoadError {
    /// The refusal in the words a host reports, which name the version before a missing export, as the C# host does.
    pub fn host_words(&self) -> String {
        match self {
            LoadError::Unloadable(_) => "could not be loaded".to_owned(),
            LoadError::Missing { abi: None, .. } => "has no emusen_core_abi_version; it is not on the core ABI".to_owned(),
            LoadError::Missing { abi: Some(v), .. } if v >> 16 != 1 => format!("speaks core ABI major {}, this host major 1", v >> 16),
            LoadError::Missing { export, .. } => format!("lacks the required export {export}"),
        }
    }
}

/// What rule 4 read of a library that passed it.
#[derive(Debug)]
pub struct Checked {
    pub version: u32,
    pub capabilities: u64,
    pub info: String,
    pub settings: String,
    pub display_name: String,
    pub version_text: String,
}

/// A loaded library; it lives for the process.
pub struct Library {
    lib: libloading::Library,
    path: PathBuf,
    exports: Exports,
    verdict: OnceLock<Result<Checked, String>>,
}

type Registry = Mutex<HashMap<PathBuf, Result<&'static Library, LoadError>>>;

fn registry() -> &'static Registry {
    static LOADED: OnceLock<Registry> = OnceLock::new();
    LOADED.get_or_init(|| Mutex::new(HashMap::new()))
}

/// `path` made absolute with `.` and `..` folded, its links left as they are, as .NET's `Path.GetFullPath` makes it.
pub fn full_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().map(|d| d.join(path)).unwrap_or_else(|_| path.to_path_buf()) };
    let mut out = PathBuf::new();
    for c in absolute.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// Reads a text the length-query way: a call with no buffer, then one of that size; the status where either fails.
pub fn text(call: impl Fn(*mut u8, usize) -> i64) -> Result<String, i64> {
    let n = call(std::ptr::null_mut(), 0);
    if n < 0 {
        return Err(n);
    }
    let mut buf = vec![0u8; n as usize];
    let m = call(buf.as_mut_ptr(), buf.len());
    if m != n {
        return Err(m.min(-1));
    }
    String::from_utf8(buf).map_err(|_| status::BAD_STRING as i64)
}

/// A descriptor text by the length query, refused past 1 MiB or when not strict UTF-8.
fn bounded(call: impl Fn(*mut u8, usize) -> i64) -> Option<String> {
    let n = call(std::ptr::null_mut(), 0);
    if !(0..=MAX_DESCRIPTOR_BYTES).contains(&n) {
        return None;
    }
    let mut buf = vec![0u8; n as usize];
    if call(buf.as_mut_ptr(), buf.len()) != n {
        return None;
    }
    String::from_utf8(buf).ok()
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn strs_of<'a>(v: &'a Value, key: &str) -> Vec<&'a str> {
    v.get(key).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default()
}

impl Library {
    /// The library at `path`, loaded the first time it is asked for with every required export; the same library, or
    /// the same refusal, every time after. Nothing is called but `emusen_core_abi_version` where an export is missing.
    pub fn load(path: &Path) -> Result<&'static Library, LoadError> {
        let full = full_path(path);
        let mut loaded = registry().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(found) = loaded.get(&full) {
            return found.clone();
        }
        let made = Self::open(&full);
        loaded.insert(full, made.clone());
        made
    }

    fn open(full: &Path) -> Result<&'static Library, LoadError> {
        // SAFETY: loading runs the library's initialisers; deciding to load it is the trust rules' job (§7.2).
        let lib = unsafe { libloading::Library::new(full) }.map_err(|e| LoadError::Unloadable(e.to_string()))?;
        // SAFETY: the export's type is the header's.
        match unsafe { Exports::resolve(&lib) } {
            Ok(exports) => Ok(Box::leak(Box::new(Library { lib, path: full.to_path_buf(), exports, verdict: OnceLock::new() }))),
            Err(export) => {
                // SAFETY: as above; the symbol is the header's `uint32_t (void)`.
                let abi = unsafe { lib.get::<unsafe extern "C" fn() -> u32>(b"emusen_core_abi_version\0") }.ok().map(|f| unsafe { f() });
                // A library that failed is never unloaded (§7.2).
                std::mem::forget(lib);
                Err(LoadError::Missing { export, abi })
            }
        }
    }

    /// Whether this process has loaded the library at `path`.
    pub fn is_loaded(path: &Path) -> bool {
        matches!(registry().lock().unwrap_or_else(|e| e.into_inner()).get(&full_path(path)), Some(Ok(_)))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }

    pub fn exports(&self) -> &Exports {
        &self.exports
    }

    pub fn abi_version(&self) -> u32 {
        // SAFETY: a required export, resolved at load.
        unsafe { (self.exports.abi_version)() }
    }

    pub fn capabilities(&self) -> u64 {
        // SAFETY: as above.
        unsafe { (self.exports.capabilities)() }
    }

    pub fn claims(&self, bit: u64) -> bool {
        self.capabilities() & bit == bit
    }

    /// Whether the library exports `name`.
    pub fn has(&self, name: &str) -> bool {
        let mut n = name.as_bytes().to_vec();
        n.push(0);
        // SAFETY: the symbol is only looked up, never called.
        unsafe { self.lib.get::<*const ()>(n.as_slice()) }.is_ok()
    }

    /// Core info as the library writes it now, by the length query.
    pub fn info_text(&self) -> Result<String, i64> {
        text(|o, l| unsafe { (self.exports.info)(o, l) })
    }

    /// The settings schema as the library writes it now.
    pub fn settings_text(&self) -> Result<String, i64> {
        text(|o, l| unsafe { (self.exports.settings_schema)(o, l) })
    }

    /// `firmware_for`'s list for an image, from its bytes alone.
    pub fn firmware_for(&self, image: &[u8]) -> Result<String, i64> {
        text(|o, l| unsafe { (self.exports.firmware_for)(image.as_ptr(), image.len(), o, l) })
    }

    /// `status_text`'s words, or `status N` where the core has none.
    pub fn words(&self, code: i64) -> String {
        let Ok(c) = i32::try_from(code) else { return format!("status {code}") };
        bounded(|o, l| unsafe { (self.exports.status_text)(c, o, l) }).filter(|w| !w.is_empty()).unwrap_or_else(|| format!("status {code}"))
    }

    /// Where the core writes its crash log; the caller names the file.
    pub fn set_crash_log(&self, file: &Path) -> i32 {
        let Ok(c) = CString::new(file.to_string_lossy().as_bytes()) else { return status::BAD_STRING };
        // The core keeps the pointer for the process, so the string is never freed.
        let c: &'static std::ffi::CStr = Box::leak(c.into_boxed_c_str());
        unsafe { (self.exports.set_crash_log)(c.as_ptr()) }
    }

    /// §7.2's rule 4, run once; the reason in the C# host's words where it has the check.
    pub fn check(&self) -> Result<&Checked, &str> {
        self.verdict.get_or_init(|| self.run_checks()).as_ref().map_err(String::as_str)
    }

    /// The report a listing shows: the library and its version, or its file name and why it is refused.
    pub fn report(&self) -> String {
        match self.check() {
            Ok(c) => format!("{}, {} {}, core ABI {}.{}", self.path.display(), c.display_name, c.version_text, c.version >> 16, c.version & 0xFFFF),
            Err(why) => format!("{}: {why}", self.file_name()),
        }
    }

    fn run_checks(&self) -> Result<Checked, String> {
        let version = self.abi_version();
        if version >> 16 != 1 {
            return Err(format!("speaks core ABI major {}, this host major 1", version >> 16));
        }
        let capabilities = self.capabilities();
        for &(name, bit) in EXPORTS {
            let present = self.has(name);
            if bit != 0 && present != (capabilities & bit == bit) {
                return Err(if present { format!("exports {name} but does not claim its capability") } else { format!("lacks {name} but claims its capability") });
            }
        }
        let info = bounded(|o, l| unsafe { (self.exports.info)(o, l) }).ok_or("its info is not UTF-8 text under 1 MiB")?;
        let doc = json::parse(info.as_bytes()).map_err(|e| format!("its info is not JSON: at byte {}", e.at))?;
        if !matches!(doc, Value::Object(_)) {
            return Err("its info is not JSON: core info is not an object".to_owned());
        }
        let abi = format!("{}.{}", version >> 16, version & 0xFFFF);
        if str_of(&doc, "abi") != abi {
            return Err(format!("its info says abi {}, its export {abi}", str_of(&doc, "abi")));
        }
        let id = str_of(&doc, "id");
        if ["id", "name", "version", "license"].iter().any(|k| str_of(&doc, k).is_empty()) {
            return Err("its info lacks id, name, version or license".to_owned());
        }
        if !id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') {
            return Err(format!("its id {id} is not [a-z0-9-]+"));
        }
        let mut claimed: Vec<&str> = CAPABILITY_NAMES.iter().filter(|(b, _)| capabilities & b != 0).map(|(_, n)| *n).collect();
        let mut listed: Vec<&str> = strs_of(&doc, "capabilities").into_iter().filter(|n| CAPABILITY_NAMES.iter().any(|(_, k)| k == n)).collect();
        claimed.sort_unstable();
        claimed.dedup();
        listed.sort_unstable();
        listed.dedup();
        if claimed != listed {
            return Err(format!("its info lists the capabilities {}, its bits {}", listed.join(", "), claimed.join(", ")));
        }
        if let Some(unknown) = strs_of(&doc, "host_requires").into_iter().find(|r| !HOST_OBLIGATIONS.contains(r)) {
            return Err(format!("it requires the host obligation {unknown}, which this host does not know"));
        }
        // §7.2 asks the info to validate against its schema; the C# host checks only the fields above (EmuSen_Backend.md §27.5).
        let problems = schema::validate(schema::INFO, &info);
        if !problems.is_empty() {
            return Err(format!("its info does not validate: {}", problems.join("; ")));
        }
        let settings = bounded(|o, l| unsafe { (self.exports.settings_schema)(o, l) }).ok_or("its settings schema is not UTF-8 text under 1 MiB")?;
        if let Err(e) = json::parse(settings.as_bytes()) {
            return Err(format!("its settings schema is not JSON: at byte {}", e.at));
        }
        let problems = schema::validate(schema::SETTINGS, &settings);
        if !problems.is_empty() {
            return Err(format!("its settings schema does not validate: {}", problems.join("; ")));
        }
        let name = str_of(&doc, "name");
        let display_name = doc.get("display_name").and_then(Value::as_str).unwrap_or(name).to_owned();
        let version_text = str_of(&doc, "version").to_owned();
        Ok(Checked { version, capabilities, info, settings, display_name, version_text })
    }
}
