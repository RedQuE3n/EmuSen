//! The v1 engines in the cores directories, listed from their sidecars alone, and opened under the trust rules when a
//! game is opened on one; the C# host's `CoreDiscovery`, rule for rule, with §7.2's rules 1 to 3 added
//! (EmuSen_CoreAPI.md §7, EmuSen_Backend.md §27.5).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use emusen_native::json;

use crate::descriptors::{CoreSystem, deep_equal};
use crate::library::Library;
use crate::sidecar::{SUFFIX, Sidecar};
use crate::trust::Trust;

/// The variable that lists cores still in development, as the C# host reads it.
pub const DEVELOPMENT_VARIABLE: &str = "EMUSEN_DEVELOPMENT_CORES";

/// One engine found by its sidecar; its library is loaded only by `open`.
pub struct Discovered {
    pub sidecar: Sidecar,
    opened: OnceLock<Result<&'static Library, String>>,
}

impl Discovered {
    pub fn engine_name(&self) -> &str {
        &self.sidecar.info.display_name
    }

    fn file_name(&self) -> String {
        self.sidecar.library_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }

    /// The library, checked, when it is the one its sidecar describes and the trust rules let it load; otherwise why
    /// not. The first answer is every answer after.
    pub fn open(&self, trust: &Trust) -> Result<&'static Library, &str> {
        self.opened.get_or_init(|| self.open_once(trust)).as_ref().map(|l| *l).map_err(String::as_str)
    }

    fn open_once(&self, trust: &Trust) -> Result<&'static Library, String> {
        let name = self.file_name();
        let path = &self.sidecar.library_path;
        if !path.is_file() {
            return Err(format!("{name} is missing beside its sidecar"));
        }
        trust.check_file(path).map_err(|why| format!("{name} {why}; it was not loaded"))?;
        let hash = crate::sha256::file_hex(path).map_err(|_| format!("{name} cannot be read; it was not loaded"))?;
        if !hash.eq_ignore_ascii_case(&self.sidecar.sha256) {
            return Err(format!("{name} is not the library its sidecar describes (its SHA-256 differs); it was not loaded"));
        }
        if !trust.vouches(path, &hash) {
            return Err(format!("{name} is in no cores manifest and has not been approved; it was not loaded"));
        }
        let lib = Library::load(path).map_err(|e| format!("{name}: {}", e.host_words()))?;
        let checked = lib.check().map_err(|_| lib.report())?;
        let same = json::parse(checked.info.as_bytes()).is_ok_and(|v| deep_equal(&v, &self.sidecar.info_json));
        if !same {
            return Err(format!("{name}'s info differs from its sidecar's; it is refused as altered"));
        }
        Ok(lib)
    }

    /// Why the last open refused, or the library's own report, or that it has not been opened.
    pub fn report(&self) -> String {
        match self.opened.get() {
            None => "not yet loaded".to_owned(),
            Some(Ok(lib)) => lib.report(),
            Some(Err(why)) => why.clone(),
        }
    }

    /// Whether the library is on disk where its sidecar says.
    pub fn available(&self) -> bool {
        self.sidecar.library_path.is_file()
    }
}

/// The engines in a list of cores directories.
pub struct Discovery {
    pub include_development: bool,
    found: Vec<Discovered>,
}

impl Discovery {
    /// Reads the sidecars in `directories`, in their order and each one's files in ordinal order; loads nothing and
    /// hashes nothing. The first of two sidecars with one id is kept.
    pub fn scan(directories: &[PathBuf], include_development: bool) -> Discovery {
        let mut found: Vec<Discovered> = Vec::new();
        let mut seen: Vec<&Path> = Vec::new();
        for dir in directories {
            if seen.contains(&dir.as_path()) {
                continue;
            }
            seen.push(dir);
            let Ok(read) = std::fs::read_dir(dir) else { continue };
            let mut names: Vec<String> = read.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).filter_map(|e| e.file_name().into_string().ok()).filter(|n| n.ends_with(SUFFIX)).collect();
            names.sort();
            for name in names {
                let Some(sidecar) = Sidecar::read(&dir.join(&name)) else { continue };
                if sidecar.abi.starts_with("1.") && (!sidecar.development || include_development) && !found.iter().any(|c| c.sidecar.info.id == sidecar.info.id) {
                    found.push(Discovered { sidecar, opened: OnceLock::new() });
                }
            }
        }
        Discovery { include_development, found }
    }

    pub fn found(&self) -> &[Discovered] {
        &self.found
    }

    /// A system its core marks in development is offered only where development cores are asked for (§27.4).
    pub fn offered(&self, system: &CoreSystem) -> bool {
        !system.development || self.include_development
    }

    /// The engines whose offered systems claim `extension`, without regard to case.
    pub fn for_extension<'a>(&'a self, extension: &str) -> impl Iterator<Item = &'a Discovered> {
        let e = extension.to_lowercase();
        self.found.iter().filter(move |c| c.sidecar.info.systems.iter().any(|s| self.offered(s) && s.extensions.contains(&e)))
    }

    /// The engines that offer a system, by its id.
    pub fn for_system<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Discovered> {
        self.found.iter().filter(move |c| c.sidecar.info.systems.iter().any(|s| s.id == id && self.offered(s)))
    }

    pub fn by_engine_name(&self, name: &str) -> Option<&Discovered> {
        self.found.iter().find(|c| c.engine_name() == name)
    }
}
