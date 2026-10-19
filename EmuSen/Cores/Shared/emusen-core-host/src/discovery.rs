//! The v1 engines in the cores directories, listed from their sidecars alone, and opened under the trust rules when a
//! game is opened on one; the C# host's `CoreDiscovery`, rule for rule, with §7.2's rules 1 to 3 added
//! (EmuSen_CoreAPI.md §7, EmuSen_Backend.md §27.5). The player's directories come after the installation's, their
//! libraries hashed at the scan and those without a sidecar listed as not yet read (EmuSen_Backend.md §29.40.4).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use emusen_native::json;

use crate::descriptors::{CoreSystem, deep_equal};
use crate::library::{Library, full_path};
use crate::sidecar::{SUFFIX, Sidecar};
use crate::trust::{Standing, Trust};

/// The variable that lists cores still in development, as the C# host reads it.
pub const DEVELOPMENT_VARIABLE: &str = "EMUSEN_DEVELOPMENT_CORES";

/// Why an open refused, and whether the want of an approval was the reason.
struct Refusal {
    words: String,
    approval: bool,
}

impl Refusal {
    fn of(words: String) -> Refusal {
        Refusal { words, approval: false }
    }
}

/// One engine found by its sidecar; its library is loaded only by `open`.
pub struct Discovered {
    pub sidecar: Sidecar,
    /// Found in one of the player's directories.
    pub player: bool,
    opened: OnceLock<Result<&'static Library, Refusal>>,
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
        self.opened.get_or_init(|| self.open_once(trust)).as_ref().map(|l| *l).map_err(|r| r.words.as_str())
    }

    /// Whether the last open refused for want of the player's approval alone.
    pub fn awaits_approval(&self) -> bool {
        matches!(self.opened.get(), Some(Err(r)) if r.approval)
    }

    fn open_once(&self, trust: &Trust) -> Result<&'static Library, Refusal> {
        let name = self.file_name();
        let path = &self.sidecar.library_path;
        if !path.is_file() {
            return Err(Refusal::of(format!("{name} is missing beside its sidecar")));
        }
        trust.check_file(path).map_err(|why| Refusal::of(format!("{name} {why}; it was not loaded")))?;
        let hash = match loaded_hash(path) {
            Some(h) => h,
            None => crate::sha256::file_hex(path).map_err(|_| Refusal::of(format!("{name} cannot be read; it was not loaded")))?,
        };
        if !hash.eq_ignore_ascii_case(&self.sidecar.sha256) {
            return Err(Refusal::of(format!("{name} is not the library its sidecar describes (its SHA-256 differs); it was not loaded")));
        }
        if !trust.vouches(path, &hash) {
            return Err(Refusal { words: format!("{name} is in no cores manifest and has not been approved; it was not loaded"), approval: true });
        }
        let lib = Library::load(path).map_err(|e| Refusal::of(format!("{name}: {}", e.host_words())))?;
        record_loaded(path, hash);
        let checked = lib.check().map_err(|_| Refusal::of(lib.report()))?;
        let same = json::parse(checked.info.as_bytes()).is_ok_and(|v| deep_equal(&v, &self.sidecar.info_json));
        if !same {
            return Err(Refusal::of(format!("{name}'s info differs from its sidecar's; it is refused as altered")));
        }
        Ok(lib)
    }

    /// Why the last open refused, or the library's own report, or that it has not been opened.
    pub fn report(&self) -> String {
        match self.opened.get() {
            None => "not yet loaded".to_owned(),
            Some(Ok(lib)) => lib.report(),
            Some(Err(why)) => why.words.clone(),
        }
    }

    /// Whether the library is on disk where its sidecar says.
    pub fn available(&self) -> bool {
        self.sidecar.library_path.is_file()
    }

    /// How far the library is trusted, by its sidecar's hash, which a player's library was found to have at the scan.
    pub fn standing(&self, trust: &Trust) -> Standing {
        if self.player {
            if trust.approved(&self.sidecar.sha256) { Standing::Approved } else { Standing::NeedsApproval }
        } else {
            trust.standing(&self.sidecar.library_path, &self.sidecar.sha256)
        }
    }
}

/// A library in a player's directory with no sidecar that describes it, listed by its hash and read only once approved.
pub struct Unread {
    pub library_path: PathBuf,
    pub sha256: String,
}

impl Unread {
    fn file_name(&self) -> String {
        self.library_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }

    /// "not yet read", or why the library this process loaded to read it was refused.
    pub fn report(&self) -> String {
        if !Library::is_loaded(&self.library_path) {
            return "not yet read".to_owned();
        }
        match Library::load(&self.library_path) {
            Ok(lib) => lib.report(),
            Err(e) => format!("{}: {}", self.file_name(), e.host_words()),
        }
    }

    /// §7.1's reading, once the player approved it: rule 3, the hash it was listed with, the load and rule 4, then its
    /// sidecar written beside it by the build's one writer. The path of the sidecar, or why not.
    pub fn read(&self, trust: &Trust) -> Result<PathBuf, String> {
        let name = self.file_name();
        let path = &self.library_path;
        if !trust.approved(&self.sha256) {
            return Err(format!("{name} has not been approved; it was not read"));
        }
        trust.check_file(path).map_err(|why| format!("{name} {why}; it was not loaded"))?;
        let hash = match loaded_hash(path) {
            Some(h) => h,
            None => crate::sha256::file_hex(path).map_err(|_| format!("{name} cannot be read; it was not loaded"))?,
        };
        if !hash.eq_ignore_ascii_case(&self.sha256) {
            return Err(format!("{name} changed after it was listed; it was not loaded"));
        }
        let lib = Library::load(path).map_err(|e| format!("{name}: {}", e.host_words()))?;
        record_loaded(path, hash);
        lib.check().map_err(|_| lib.report())?;
        crate::sidecar::write(path, false)
    }
}

/// What the process knows of a library's hash: the one it had when this process loaded it, which is the image that
/// runs, or one the scan read, good while the file's identity, length and time are those it was read at.
enum Known {
    Loaded(String),
    Scanned(String, Stamp),
}

type Stamp = (u64, u64, u64, i128);

fn stamp_of(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    let time = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as i128).unwrap_or(-1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some((meta.dev(), meta.ino(), meta.len(), time))
    }
    #[cfg(not(unix))]
    Some((0, 0, meta.len(), time))
}

/// The process's hashes by full path (EmuSen_Backend.md §29.26, §29.40.4).
fn hashes() -> &'static Mutex<HashMap<PathBuf, Known>> {
    static HASHES: OnceLock<Mutex<HashMap<PathBuf, Known>>> = OnceLock::new();
    HASHES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn loaded_hash(path: &Path) -> Option<String> {
    match hashes().lock().unwrap_or_else(|e| e.into_inner()).get(&full_path(path)) {
        Some(Known::Loaded(h)) => Some(h.clone()),
        _ => None,
    }
}

/// The hash a library had when it was loaded, kept for the process; the first load's is the image that runs.
fn record_loaded(path: &Path, hash: String) {
    let mut map = hashes().lock().unwrap_or_else(|e| e.into_inner());
    let full = full_path(path);
    if !matches!(map.get(&full), Some(Known::Loaded(_))) {
        map.insert(full, Known::Loaded(hash));
    }
}

/// A player's library's hash for the listing: the loaded image's, or the file's, read once while it is unchanged.
fn scan_hash(path: &Path) -> Option<String> {
    let full = full_path(path);
    let stamp = stamp_of(path)?;
    match hashes().lock().unwrap_or_else(|e| e.into_inner()).get(&full) {
        Some(Known::Loaded(h)) => return Some(h.clone()),
        Some(Known::Scanned(h, s)) if *s == stamp => return Some(h.clone()),
        _ => {}
    }
    let hash = crate::sha256::file_hex(path).ok()?;
    let mut map = hashes().lock().unwrap_or_else(|e| e.into_inner());
    if !matches!(map.get(&full), Some(Known::Loaded(_))) {
        map.insert(full, Known::Scanned(hash.clone(), stamp));
    }
    Some(hash)
}

/// The engines in a list of cores directories.
pub struct Discovery {
    pub include_development: bool,
    found: Vec<Discovered>,
    unread: Vec<Unread>,
}

fn file_names(dir: &Path) -> Option<Vec<String>> {
    let read = std::fs::read_dir(dir).ok()?;
    let mut names: Vec<String> = read.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).filter_map(|e| e.file_name().into_string().ok()).collect();
    names.sort();
    Some(names)
}

impl Discovery {
    /// Reads the sidecars in the installation's `directories`, in their order and each one's files in ordinal order;
    /// loads nothing and hashes nothing. The first of two sidecars with one id is kept.
    pub fn scan(directories: &[PathBuf], include_development: bool) -> Discovery {
        Discovery::scan_with_player(directories, &[], include_development)
    }

    /// As `scan`, then the player's directories: each library there hashed, a sidecar whose hash is not its library's
    /// set aside, and a library no sidecar describes listed as not yet read.
    pub fn scan_with_player(installation: &[PathBuf], player: &[PathBuf], include_development: bool) -> Discovery {
        let mut d = Discovery { include_development, found: Vec::new(), unread: Vec::new() };
        let mut seen: Vec<&Path> = Vec::new();
        for (dir, mine) in installation.iter().map(|d| (d, false)).chain(player.iter().map(|d| (d, true))) {
            if seen.contains(&dir.as_path()) {
                continue;
            }
            seen.push(dir);
            let Some(names) = file_names(dir) else { continue };
            let mut described: Vec<PathBuf> = Vec::new();
            for name in names.iter().filter(|n| n.ends_with(SUFFIX)) {
                let Some(sidecar) = Sidecar::read(&dir.join(name)) else { continue };
                if mine && sidecar.library_path.is_file() {
                    match scan_hash(&sidecar.library_path) {
                        Some(h) if h.eq_ignore_ascii_case(&sidecar.sha256) => {}
                        _ => continue,
                    }
                }
                described.push(sidecar.library_path.clone());
                d.admit(sidecar, mine);
            }
            if mine {
                let suffix = std::env::consts::DLL_SUFFIX;
                for name in names.iter().filter(|n| n.ends_with(suffix) && n.len() > suffix.len()) {
                    let path = dir.join(name);
                    if described.contains(&path) {
                        continue;
                    }
                    if let Some(sha256) = scan_hash(&path) {
                        d.unread.push(Unread { library_path: path, sha256 });
                    }
                }
            }
        }
        d
    }

    fn admit(&mut self, sidecar: Sidecar, player: bool) {
        if sidecar.abi.starts_with("1.") && (!sidecar.development || self.include_development) && !self.found.iter().any(|c| c.sidecar.info.id == sidecar.info.id) {
            self.found.push(Discovered { sidecar, player, opened: OnceLock::new() });
        }
    }

    pub fn found(&self) -> &[Discovered] {
        &self.found
    }

    /// The player's libraries without a sidecar, in the scan's order.
    pub fn unread(&self) -> &[Unread] {
        &self.unread
    }

    /// Whether a library the listing shows has this hash, as `"sha256"` gives it.
    pub fn lists_hash(&self, sha256: &str) -> bool {
        self.found.iter().any(|c| c.sidecar.sha256.eq_ignore_ascii_case(sha256)) || self.unread.iter().any(|u| u.sha256.eq_ignore_ascii_case(sha256))
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
