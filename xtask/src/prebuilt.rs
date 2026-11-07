//! A publish for another platform: the libraries CI built for it, taken from `<from>/<rid>/` with their sidecars and a
//! cores.manifest of its own, and the names of the host's native files that publish must drop. See Mars_Native.md §6.3,
//! EmuSen_RustPlatform.md §2.6 and §28.11.

use crate::files::{self, sha256_hex};
use crate::libraries::{Library, Role};
use crate::{host, say};
use std::path::{Path, PathBuf};

pub fn run(libs: &[Library], rid: &str, from: Option<&Path>, stage: &Path) -> Result<(), String> {
    let files_dir = stage.join("files");
    let _ = std::fs::remove_dir_all(&files_dir);
    std::fs::create_dir_all(&files_dir).map_err(|e| format!("{}: {e}", files_dir.display()))?;
    let mut drop = vec!["cores.manifest".to_string()];
    for l in libs {
        for r in ["linux-x64", "win-x64", "osx-arm64"] {
            let file = if l.role == Role::Program { host::program_file(&l.name, r) } else { host::library_file(&l.name, r) };
            drop.push(format!("{file}.core.json"));
            drop.push(file);
        }
    }
    drop.sort();
    drop.dedup();
    let prebuilt = |file: &str| from.map(|f| f.join(rid).join(file)).filter(|p| p.is_file());
    let advice = |file: &str| format!("Build {file} with the Rust cores workflow and pass -p:EmuSenNativePrebuilt=<dir> holding {rid}/{file}.");
    let mut manifest = String::new();
    for l in libs.iter().filter(|l| l.ships && l.builds_for(rid)) {
        let file = if l.role == Role::Program { host::program_file(&l.name, rid) } else { host::library_file(&l.name, rid) };
        let Some(path) = prebuilt(&file) else {
            // The programs cannot start without the platform library, so a publish without it stops (decided 2026-11-07).
            if l.role == Role::Platform {
                return Err(format!("No {file} for {rid}: the programs cannot start without the platform library. {}", advice(&file)));
            }
            say::warn(&format!("No {} library for {rid}: this publish has no engine for {}. {}", l.core, l.console, advice(&file)));
            continue;
        };
        files::stage(&path, &files_dir.join(&file))?;
        let sidecar = PathBuf::from(format!("{}.core.json", path.display()));
        if sidecar.is_file() {
            files::stage(&sidecar, &files_dir.join(format!("{file}.core.json")))?;
        }
        if l.role != Role::Program && l.role != Role::Platform {
            manifest.push_str(&format!("{}  {file}\n", sha256_hex(&path)?));
        }
    }
    files::write_if_different(&stage.join(format!("{rid}.cores.manifest")), &manifest)?;
    files::stage(&stage.join(format!("{rid}.cores.manifest")), &files_dir.join("cores.manifest"))?;
    files::write_if_different(&stage.join("drop.txt"), &(drop.join("\n") + "\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_publish_without_the_platform_library_stops_and_names_it() {
        let dir = std::env::temp_dir().join(format!("xtask-prebuilt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("from/win-x64")).unwrap();
        let libs = crate::libraries::parse("emusen_platform\tPlatform\tplatform\t-\tships\tany\t-\tthe platform library\tthe platform components\n").unwrap();
        let said = run(&libs, "win-x64", Some(&dir.join("from")), &dir.join("stage")).unwrap_err();
        assert!(said.contains("No emusen_platform.dll for win-x64") && said.contains("-p:EmuSenNativePrebuilt"), "{said}");
        std::fs::write(dir.join("from/win-x64/emusen_platform.dll"), b"MZ").unwrap();
        run(&libs, "win-x64", Some(&dir.join("from")), &dir.join("stage")).unwrap();
        assert!(dir.join("stage/files/emusen_platform.dll").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
