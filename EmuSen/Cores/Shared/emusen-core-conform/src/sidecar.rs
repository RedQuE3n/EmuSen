//! A library's sidecar, `<library>.core.json`, as the host's discovery reads it (EmuSen_CoreAPI.md §19.2): written by
//! loading the library once, which is the build step's work.

use std::path::{Path, PathBuf};

use emusen_native::core::jsonw::Json;
use emusen_native::core::schema;

use crate::api::Lib;

/// Writes the sidecar beside the library and returns its path; refuses a library whose info or schema does not validate.
pub fn write(library: &Path) -> Result<PathBuf, String> {
    let lib = Lib::open(library)?;
    let version = unsafe { (lib.f.abi_version)() };
    if version >> 16 != 1 {
        return Err(format!("{} speaks core ABI major {}, not 1", library.display(), version >> 16));
    }
    let info = lib.info().map_err(|e| format!("info failed with {e}"))?;
    let settings = lib.settings().map_err(|e| format!("settings_schema failed with {e}"))?;
    let problems: Vec<String> = schema::validate(schema::INFO, &info).into_iter().chain(schema::validate(schema::SETTINGS, &settings)).collect();
    if !problems.is_empty() {
        return Err(format!("its descriptors do not validate: {}", problems.join("; ")));
    }
    let bytes = std::fs::read(library).map_err(|e| format!("{}: {e}", library.display()))?;
    let name = library.file_name().and_then(|n| n.to_str()).ok_or("a library name that is not UTF-8")?;
    let mut head = Json::new();
    head.begin_object()
        .field_int("sidecar", 1)
        .field_str("library", name)
        .field_str("sha256", &crate::digest::sha256_hex(&bytes))
        .field_str("abi", &format!("{}.{}", version >> 16, version & 0xFFFF))
        .field_uint("capabilities", lib.capabilities())
        .end_object();
    let head = head.finish();
    // The descriptors are placed as the library wrote them, so that the host compares the very text it will be given.
    let text = format!("{},\"info\":{info},\"settings\":{settings}}}\n", &head[..head.len() - 1]);
    let path = PathBuf::from(format!("{}.core.json", library.display()));
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}
