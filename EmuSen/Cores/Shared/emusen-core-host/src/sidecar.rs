//! A library's sidecar, `<library>.core.json`: its info and settings schema and its SHA-256, so that a core is listed
//! without loading it. Read as the C# host reads it and written as the build writes it (EmuSen_CoreAPI.md §7.1, §19.2).

use std::path::{Path, PathBuf};

use emusen_native::core::jsonw::Json;
use emusen_native::core::schema;
use emusen_native::json::{self, Value};

use crate::descriptors::CoreInfo;
use crate::library::Library;

pub const SUFFIX: &str = ".core.json";

#[derive(Clone, Debug)]
pub struct Sidecar {
    pub sidecar_path: PathBuf,
    pub library_path: PathBuf,
    pub sha256: String,
    pub abi: String,
    pub capabilities: u64,
    /// A core not yet offered to players, which discovery skips unless asked (EmuSen_CoreAPI.md §27).
    pub development: bool,
    pub info: CoreInfo,
    pub info_json: Value,
    /// The settings schema as the sidecar holds it, an empty list where it holds none.
    pub settings: Value,
}

/// The sidecar's path for a library.
pub fn path_for(library: &Path) -> PathBuf {
    PathBuf::from(format!("{}{SUFFIX}", library.display()))
}

/// A string member, absent as `""`; `None` where it is present and not a string, which the C# reader throws on.
fn opt_string(doc: &Value, key: &str) -> Option<String> {
    match doc.get(key) {
        None | Some(Value::Null) => Some(String::new()),
        Some(Value::Str(s)) => Some(s.clone()),
        Some(_) => None,
    }
}

impl Sidecar {
    /// The sidecar at `path`, or `None` for a file that is not a sidecar of this version or names a library that is not
    /// a bare file name; the C# host's rules, field for field.
    pub fn read(path: &Path) -> Option<Sidecar> {
        let bytes = std::fs::read(path).ok()?;
        let doc = json::parse(&bytes).ok()?;
        if !matches!(doc, Value::Object(_)) || doc.get("sidecar") != Some(&Value::Int(1)) {
            return None;
        }
        let name = doc.get("library")?.as_str()?;
        if name.contains('/') || (cfg!(windows) && name.contains('\\')) {
            return None;
        }
        let info_json = doc.get("info")?;
        if !matches!(info_json, Value::Object(_)) {
            return None;
        }
        let sha256 = opt_string(&doc, "sha256")?;
        let abi = opt_string(&doc, "abi")?;
        let capabilities = match doc.get("capabilities") {
            None | Some(Value::Null) => 0,
            Some(Value::Int(n)) if *n >= 0 => *n as u64,
            Some(_) => return None,
        };
        let development = match doc.get("development") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(_) => return None,
        };
        let info = CoreInfo::read(info_json)?;
        let settings = match doc.get("settings") {
            None | Some(Value::Null) => Value::Array(Vec::new()),
            Some(v) => v.clone(),
        };
        let library_path = path.parent().unwrap_or(Path::new("")).join(name);
        Some(Sidecar { sidecar_path: path.to_path_buf(), library_path, sha256, abi, capabilities, development, info, info_json: info_json.clone(), settings })
    }
}

/// Loads the library once, as the build step does, and writes its sidecar beside it; refuses a library whose info or
/// schema does not validate. `development` marks a core not yet offered to players.
pub fn write(library: &Path, development: bool) -> Result<PathBuf, String> {
    let lib = Library::load(library).map_err(|e| e.to_string())?;
    let version = lib.abi_version();
    if version >> 16 != 1 {
        return Err(format!("{} speaks core ABI major {}, not 1", library.display(), version >> 16));
    }
    let info = lib.info_text().map_err(|e| format!("info failed with {e}"))?;
    let settings = lib.settings_text().map_err(|e| format!("settings_schema failed with {e}"))?;
    let problems: Vec<String> = schema::validate(schema::INFO, &info).into_iter().chain(schema::validate(schema::SETTINGS, &settings)).collect();
    if !problems.is_empty() {
        return Err(format!("its descriptors do not validate: {}", problems.join("; ")));
    }
    let sha = crate::sha256::file_hex(library).map_err(|e| format!("{}: {e}", library.display()))?;
    let name = library.file_name().and_then(|n| n.to_str()).ok_or("a library name that is not UTF-8")?;
    let mut head = Json::new();
    head.begin_object().field_int("sidecar", 1).field_str("library", name).field_str("sha256", &sha).field_str("abi", &format!("{}.{}", version >> 16, version & 0xFFFF)).field_uint("capabilities", lib.capabilities());
    if development {
        head.field_bool("development", true);
    }
    head.end_object();
    let head = head.finish();
    // The descriptors are placed as the library wrote them, so that the host compares the very text it will be given.
    let text = format!("{},\"info\":{info},\"settings\":{settings}}}\n", &head[..head.len() - 1]);
    let path = path_for(library);
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}
