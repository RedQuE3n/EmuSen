//! An installed libretro core's folder, `<player cores>/libretro/<stem>/`, as the host lists and admits it: the
//! project's proxy and adapter copies vouched for by an installation's manifest, the libretro core by its own hash
//! among the approvals and still that build on disk, and the proxy then loaded with that hash expected, so that no
//! other bytes run (EmuSen_LibretroHost.md §1.1, §1.3, §10e and §10k). The folder's file names are
//! `emusen-core-entry`'s; nothing here names a file of its own.

use std::path::PathBuf;

pub use emusen_core_entry::{Entry, FIRMWARE_ROLES, FirmwareRole, firmware_role, system_folder};
use emusen_core_entry::{HOST_LIBRARY, PROXY_LIBRARY, built_name};

use crate::descriptors::LibretroFile;
use crate::library::Library;
use crate::trust::{Standing, Trust};

/// Why nothing of an entry may start, and whether the player's approval alone would settle it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    pub words: String,
    pub approval: bool,
}

fn held(words: String, approval: bool) -> Held {
    Held { words, approval }
}

fn folder(entry: &Entry) -> String {
    entry.folder.display().to_string()
}

/// The libretro core's hash as the scan reads it: once while the file is unchanged.
pub fn core_hash(entry: &Entry) -> Option<String> {
    crate::discovery::scan_hash(&entry.core_path())
}

/// Rule 1 for an entry: each copy is a library an installation's manifest lists under the name it is built as.
pub fn stale(entry: &Entry, trust: &Trust) -> Option<String> {
    for (copy, library) in [(entry.broker_path(), PROXY_LIBRARY), (entry.adapter_path(), HOST_LIBRARY)] {
        let built = built_name(library);
        let vouched = copy.is_file() && crate::discovery::scan_hash(&copy).is_some_and(|h| trust.installation_lists(&built, &h));
        if !vouched {
            let name = copy.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            return Some(format!("{name} in {} is not this installation's {built}; the entry needs refresh (cores verify), and nothing of it was started", folder(entry)));
        }
    }
    None
}

/// Whether the entry's core, as its sidecar names it, may start: its copies the installation's, the file the one the
/// folder is named for, its hash approved and the file on disk still that build.
pub fn check(entry: &Entry, libretro: Option<&LibretroFile>, trust: &Trust) -> Result<(), Held> {
    if let Some(words) = stale(entry, trust) {
        return Err(held(words, false));
    }
    let Some(core) = libretro else { return Err(held(format!("the sidecar in {} names no libretro core; nothing of it was started", folder(entry)), false)) };
    let path = entry.core_path();
    if path.file_name().and_then(|n| n.to_str()) != Some(core.file.as_str()) || !path.is_file() {
        return Err(held(format!("{} does not hold {}; nothing of it was started", folder(entry), core.file), false));
    }
    if !trust.approved(&core.sha256) {
        return Err(held(format!("the libretro core {} (SHA-256 {}) has not been approved; nothing of it was started", core.file, core.sha256), true));
    }
    if core_hash(entry).as_deref() != Some(core.sha256.as_str()) {
        return Err(held(format!("{} is not the build that was approved (its SHA-256 differs); nothing of it was started", core.file), true));
    }
    Ok(())
}

/// The entry's standing for a listing: needs_refresh, approved or needs_approval.
pub fn standing(entry: &Entry, libretro: Option<&LibretroFile>, trust: &Trust) -> Standing {
    match check(entry, libretro, trust) {
        Ok(()) => Standing::Approved,
        Err(_) if stale(entry, trust).is_some() => Standing::NeedsRefresh,
        Err(_) => Standing::NeedsApproval,
    }
}

/// An entry's stale copies refreshed from the installation directory whose manifest vouches for both, as `cores verify`
/// refreshes them, so an approval stands across an update of EmuSen (§1.3, §10v); whether anything changed, or why the
/// entry still needs refresh.
pub fn refresh(entry: &Entry, trust: &Trust) -> Result<bool, String> {
    let Some(words) = stale(entry, trust) else { return Ok(false) };
    let Some(installation) = trust.vouched_installation() else {
        return Err(format!("{words}; no installation folder holds copies its manifest lists to refresh it from"));
    };
    entry.refresh(installation).map_err(|why| format!("{words}; refreshing it failed: {why}"))?;
    stale(entry, trust).map_or(Ok(true), Err)
}

/// An entry whose core has no sidecar of its build yet: none at all, or one naming another build of it.
#[derive(Clone, Debug)]
pub struct Waiting {
    pub entry: Entry,
    /// The libretro core's hash on disk, which an approval names.
    pub sha256: String,
}

/// §1.1's reading, once the core's hash is approved: the copies checked, the proxy loaded with the hash expected (so
/// the proxy seals and compares the core before anything of it runs) and checked, and its sidecar written beside it.
/// The sidecar's path, or why not.
pub fn read(waiting: &Waiting, trust: &Trust) -> Result<PathBuf, String> {
    let entry = &waiting.entry;
    let file = entry.core_path().file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if !trust.approved(&waiting.sha256) {
        return Err(format!("the libretro core {file} (SHA-256 {}) has not been approved; nothing of it was started", waiting.sha256));
    }
    refresh(entry, trust)?;
    let proxy = entry.broker_path();
    let name = proxy.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    trust.check_file(&proxy).map_err(|why| format!("{name} {why}; it was not loaded"))?;
    let lib = Library::load_expecting(&proxy, &waiting.sha256).map_err(|e| format!("{name}: {}", e.host_words()))?;
    lib.check().map_err(|_| lib.report())?;
    crate::sidecar::write(&proxy, false)
}
