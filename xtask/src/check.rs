//! A folder's cores against their sidecars: each library a sidecar names must be beside it with the SHA-256 the
//! sidecar records, the runtime's own rule, so no build or publish leaves cores that would be refused. See
//! EmuSen_RustPlatform.md §28.10.

use crate::files::sha256_hex;
use std::path::Path;

/// Every refusal in `folder`, in words; none when its cores load.
pub fn check(folder: &Path) -> Result<Vec<String>, String> {
    let mut sidecars: Vec<_> = std::fs::read_dir(folder)
        .map_err(|e| format!("{}: {e}", folder.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.to_string_lossy().ends_with(".core.json"))
        .collect();
    sidecars.sort();
    let mut refused = Vec::new();
    for sidecar in sidecars {
        let name = sidecar.file_name().unwrap().to_string_lossy().to_string();
        let doc: serde_json::Value = std::fs::read_to_string(&sidecar).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        let library = doc["library"].as_str().filter(|l| !l.is_empty() && !l.contains(['/', '\\']));
        let expected = doc["sha256"].as_str().filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())).map(str::to_ascii_lowercase);
        let (Some(library), Some(expected)) = (library, expected) else {
            refused.push(format!("{name} in {} names no library or no SHA-256.", folder.display()));
            continue;
        };
        let path = folder.join(library);
        if !path.is_file() {
            refused.push(format!("{name} in {} describes {library}, which is missing; a game on that core would not start.", folder.display()));
            continue;
        }
        let actual = sha256_hex(&path)?;
        if actual != expected {
            refused.push(format!(
                "{library} in {} is not the library its sidecar describes (SHA-256 {actual}, sidecar {expected}); a game on that core would not start. Copy the library from beside the sidecar it was built with, or rebuild - see EmuSen_RustPlatform.md §28.10.",
                folder.display()
            ));
        }
    }
    Ok(refused)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn a_core_that_is_not_its_sidecars_is_refused() {
        let dir = std::env::temp_dir().join(format!("xtask-check-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("libcore.so"), b"abc").unwrap();
        let sum = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        fs::write(dir.join("libcore.so.core.json"), format!(r#"{{"sidecar":1,"library":"libcore.so","sha256":"{}"}}"#, sum.to_uppercase())).unwrap();
        assert!(check(&dir).unwrap().is_empty());
        fs::write(dir.join("libcore.so"), b"abd").unwrap();
        assert!(check(&dir).unwrap()[0].contains("is not the library its sidecar describes"));
        fs::remove_file(dir.join("libcore.so")).unwrap();
        assert!(check(&dir).unwrap()[0].contains("which is missing"));
        fs::write(dir.join("libcore.so.core.json"), r#"{"sidecar":1,"library":"../libcore.so","sha256":"00"}"#).unwrap();
        assert!(check(&dir).unwrap()[0].contains("names no library"));
        let _ = fs::remove_dir_all(&dir);
    }
}
