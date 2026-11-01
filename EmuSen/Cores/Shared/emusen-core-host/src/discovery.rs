//! The v1 engines in the cores directories, listed from their sidecars alone, and opened under the trust rules when a
//! game is opened on one; the C# host's `CoreDiscovery`, rule for rule, with §7.2's rules 1 to 3 added
//! (EmuSen_CoreAPI.md §7, EmuSen_Backend.md §27.5). The player's directories come after the installation's, their
//! libraries hashed at the scan and those without a sidecar listed as not yet read (EmuSen_Backend.md §29.40.4); then
//! their installed libretro cores, each a folder of [`entry`](crate::entry)'s, ordered among a system's engines by
//! the hosted list's rule (EmuSen_LibretroHost.md §10k, EmuSen_Cores.md §4.8).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use emusen_native::json;

use crate::descriptors::{CoreSystem, deep_equal};
use crate::entry::{Entry, Waiting};
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
    /// The installed libretro core whose proxy copy the sidecar describes; none for every other library.
    pub entry: Option<Entry>,
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
        let expected = match &self.entry {
            Some(entry) => {
                let libretro = self.sidecar.info.libretro.as_ref();
                crate::entry::check(entry, libretro, trust).map_err(|h| Refusal { words: h.words, approval: h.approval })?;
                libretro.map(|l| l.sha256.as_str())
            }
            None if !trust.vouches(path, &hash) => {
                return Err(Refusal { words: format!("{name} is in no cores manifest and has not been approved; it was not loaded"), approval: true });
            }
            None => None,
        };
        let loaded = match expected {
            Some(sha256) => Library::load_expecting(path, sha256),
            None => Library::load(path),
        };
        let lib = loaded.map_err(|e| Refusal::of(format!("{name}: {}", e.host_words())))?;
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
        if let Some(entry) = &self.entry {
            crate::entry::standing(entry, self.sidecar.info.libretro.as_ref(), trust)
        } else if self.player {
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

    /// §7.1's reading, once the player approved it: `load`, then its sidecar written beside it by the build's one
    /// writer. The path of the sidecar, or why not.
    pub fn read(&self, trust: &Trust) -> Result<PathBuf, String> {
        self.load(trust)?;
        crate::sidecar::write(&self.library_path, false)
    }

    /// The approved library loaded and checked: rule 3, the hash it was listed with, the load and rule 4.
    pub fn load(&self, trust: &Trust) -> Result<&'static Library, String> {
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
        Ok(lib)
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
pub(crate) fn scan_hash(path: &Path) -> Option<String> {
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

/// A libretro core the hosted list names: the systems it may run, and those it is the default engine of ahead of a
/// native core (`emusen_systems::libretro_cores`, EmuSen_Cores.md §4.7 and §4.8).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostedCore {
    pub stem: String,
    pub systems: Vec<String>,
    pub default_for: Vec<String>,
}

/// The engines in a list of cores directories.
pub struct Discovery {
    pub include_development: bool,
    found: Vec<Discovered>,
    unread: Vec<Unread>,
    waiting: Vec<Waiting>,
    hosted: Vec<HostedCore>,
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
        let mut d = Discovery { include_development, found: Vec::new(), unread: Vec::new(), waiting: Vec::new(), hosted: Vec::new() };
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
                d.admit(sidecar, mine, None);
            }
            if mine {
                d.scan_entries(dir);
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

    fn admit(&mut self, sidecar: Sidecar, player: bool, entry: Option<Entry>) {
        if sidecar.abi.starts_with("1.") && (!sidecar.development || self.include_development) && !self.found.iter().any(|c| c.sidecar.info.id == sidecar.info.id) {
            self.found.push(Discovered { sidecar, player, entry, opened: OnceLock::new() });
        }
    }

    /// A player's directory's libretro entries: each with its proxy's sidecar listed as an engine, and each whose files
    /// on disk have no sidecar of theirs (none, one of another build of the core, or one of a proxy copy since refreshed)
    /// listed as waiting to be read; an approval of the core's hash reads it.
    fn scan_entries(&mut self, dir: &Path) {
        for entry in Entry::list(dir).unwrap_or_default() {
            let sidecar = Sidecar::read(&crate::sidecar::path_for(&entry.broker_path()));
            let hash = crate::entry::core_hash(&entry);
            let proxy = scan_hash(&entry.broker_path());
            let current = matches!((&sidecar, &hash, &proxy), (Some(s), Some(h), Some(p)) if s.info.libretro.as_ref().is_some_and(|l| l.sha256.eq_ignore_ascii_case(h)) && s.sha256.eq_ignore_ascii_case(p));
            if let Some(sha256) = hash.filter(|_| !current) {
                self.waiting.push(Waiting { entry: entry.clone(), sha256 });
            }
            if let Some(sidecar) = sidecar {
                self.admit(sidecar, true, Some(entry));
            }
        }
    }

    /// The hosted list the rule orders a system's engines by and holds each entry to; none treats every entry as a
    /// hosted core named nowhere.
    pub fn with_hosted(mut self, hosted: Vec<HostedCore>) -> Discovery {
        self.hosted = hosted;
        self
    }

    /// The entries whose core waits for an approval to be read, in the scan's order.
    pub fn waiting(&self) -> &[Waiting] {
        &self.waiting
    }

    /// The systems the hosted list's line names for an entry, none where the list names it nowhere.
    pub fn hosted_systems(&self, entry: &Entry) -> &[String] {
        self.listed(entry).map_or(&[], |h| h.systems.as_slice())
    }

    fn listed(&self, entry: &Entry) -> Option<&HostedCore> {
        self.hosted.iter().find(|h| h.stem == entry.stem)
    }

    /// Whether an engine offers a system: offered at all, and for an entry the hosted list names, one its line names.
    pub fn offers(&self, core: &Discovered, system: &CoreSystem) -> bool {
        self.offered(system) && core.entry.as_ref().and_then(|e| self.listed(e)).is_none_or(|h| h.systems.contains(&system.id))
    }

    /// The rule's place of an engine among a system's: the hosted core the list makes its default, then the native
    /// cores, then the other hosted cores.
    fn rank(&self, core: &Discovered, system: Option<&str>) -> u8 {
        match &core.entry {
            None => 1,
            Some(e) if system.is_some_and(|s| self.listed(e).is_some_and(|h| h.default_for.iter().any(|d| d == s))) => 0,
            Some(_) => 2,
        }
    }

    /// Engines in the rule's order for `system`, each group in discovery's order; the first is the default.
    pub fn in_rule_order<'a>(&self, mut engines: Vec<&'a Discovered>, system: Option<&str>) -> Vec<&'a Discovered> {
        engines.sort_by_key(|c| self.rank(c, system));
        engines
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

    /// The engines whose offered systems claim `extension`, without regard to case, in the rule's order for the system
    /// each runs it as.
    pub fn for_extension<'a>(&'a self, extension: &str) -> std::vec::IntoIter<&'a Discovered> {
        let e = extension.to_lowercase();
        let system = |c: &'a Discovered| c.sidecar.info.systems.iter().find(|s| self.offers(c, s) && s.extensions.contains(&e));
        let mut found: Vec<(&Discovered, &CoreSystem)> = self.found.iter().filter_map(|c| system(c).map(|s| (c, s))).collect();
        found.sort_by_key(|(c, s)| self.rank(c, Some(&s.id)));
        found.into_iter().map(|(c, _)| c).collect::<Vec<_>>().into_iter()
    }

    /// The engines that offer a system, by its id, in the rule's order.
    pub fn for_system<'a>(&'a self, id: &str) -> std::vec::IntoIter<&'a Discovered> {
        let found = self.found.iter().filter(|c| c.sidecar.info.systems.iter().any(|s| s.id == id && self.offers(c, s))).collect();
        self.in_rule_order(found, Some(id)).into_iter()
    }

    pub fn by_engine_name(&self, name: &str) -> Option<&Discovered> {
        self.found.iter().find(|c| c.engine_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sidecar(path: &Path, id: &str, systems: &[(&str, &str)]) {
        let systems: Vec<String> = systems.iter().map(|(s, e)| format!(r#"{{"id":"{s}","name":"{s}","extensions":["{e}"]}}"#)).collect();
        let library = path.file_name().unwrap().to_str().unwrap().trim_end_matches(SUFFIX);
        let text = format!(r#"{{"sidecar":1,"library":"{library}","sha256":"","abi":"1.17","info":{{"id":"{id}","name":"{id}","systems":[{}]}}}}"#, systems.join(","));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn a_systems_engines_follow_the_hosted_lists_rule_and_an_entry_offers_only_its_lines_systems() {
        let root = std::env::temp_dir().join(format!("core-host-rule-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (native, player) = (root.join("lib"), root.join("Cores"));
        sidecar(&native.join("libnephrite.so.core.json"), "nephrite", &[("md", ".md")]);
        sidecar(&native.join("libsapphire.so.core.json"), "sapphire", &[("ps2", ".iso")]);
        let entry = |stem: &str| crate::sidecar::path_for(&Entry::new(&player, stem).unwrap().broker_path());
        sidecar(&entry("genesis_plus_gx"), "genesis_plus_gx", &[("md", ".md"), ("gg", ".gg")]);
        sidecar(&entry("pcee2"), "pcee2", &[("ps2", ".iso")]);
        let hosted = vec![
            HostedCore { stem: "pcee2".into(), systems: vec!["ps2".into()], default_for: vec!["ps2".into()] },
            HostedCore { stem: "genesis_plus_gx".into(), systems: vec!["md".into()], default_for: vec![] },
        ];
        let d = Discovery::scan_with_player(&[native], &[player], false).with_hosted(hosted);
        let ids = |it: std::vec::IntoIter<&Discovered>| it.map(|c| c.sidecar.info.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(d.for_system("md")), ["nephrite", "genesis_plus_gx"], "native first");
        assert_eq!(ids(d.for_system("ps2")), ["pcee2", "sapphire"], "the list's one default ahead of the native core");
        assert_eq!(ids(d.for_extension(".ISO")), ["pcee2", "sapphire"]);
        assert!(d.for_system("gg").next().is_none(), "a system its list line does not name is not offered");
        assert!(d.found().iter().filter(|c| c.entry.is_some()).all(|c| c.entry.as_ref().unwrap().stem == c.sidecar.info.id));
        let _ = std::fs::remove_dir_all(&root);
    }
}
