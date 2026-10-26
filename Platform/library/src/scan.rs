//! The scan, the C# `RomLibrary.Scan` with each file's shelf: the roots walked as .NET's `EnumerationOptions` have it,
//! the files of a named system's extensions or an archive's kept, a sheet's and a list's files taken as their game's,
//! each file routed among the named systems by `emusen-systems` over the bytes Galaxia reads, and the games ordered by
//! title ignoring case. See EmuSen_Settings_Reference.md §4.11, §4.46 and EmuSen_Backend.md §28.4.

use crate::{Error, Progress};
use emusen_dotnet::text::{compare_ignore_case, equals_ignore_case};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::image_probe::{self, ImageProbe, head_of};
use emusen_galaxia::{archive, claims, dotnet_files, image_sets, package_zip};
use emusen_systems::probe::{self, Candidate, Recognition};
use std::sync::atomic::Ordering;

/// Why a scan can list nothing, `RomLibraryStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    NoDirectory,
    NotFound,
    Empty,
    Ok,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Outcome::NoDirectory => "no_directory",
            Outcome::NotFound => "not_found",
            Outcome::Empty => "empty",
            Outcome::Ok => "ok",
        }
    }
}

/// One game the scan found, the system it is shelved as, none where no named system takes it, and the package it is
/// the game of, none for a file no package claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub path: String,
    pub system: Option<String>,
    pub package: Option<String>,
}

/// The two folders a scan walks beside its roots since game packages (EmuSen_GamePackages.md §3.6): `Packages`, whose
/// package folders are one entry each, and `Compressed`. Neither changes the scan's outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Beside {
    pub packages: Option<String>,
    pub compressed: Option<String>,
}

/// A finished scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scan {
    pub outcome: Outcome,
    pub roots: Vec<String>,
    pub systems: Vec<String>,
    pub games: Vec<Game>,
}

/// The systems a scan routes among: the table's entries the request names, in its order.
pub struct Systems {
    entries: Vec<&'static emusen_systems::System>,
}

impl Systems {
    pub fn named(ids: &[String]) -> Systems {
        Systems { entries: ids.iter().filter_map(|id| emusen_systems::find(id)).collect() }
    }

    /// Every extension a named system takes, then an archive's: `RomLibrary.Extensions`.
    pub fn extensions(&self) -> Vec<String> {
        let mut all: Vec<String> = Vec::new();
        for e in self.entries.iter().flat_map(|s| s.extensions.iter()).map(String::as_str).chain(archive::EXTENSIONS.iter().copied()) {
            if !all.iter().any(|a| equals_ignore_case(a, e)) {
                all.push(e.to_string());
            }
        }
        all
    }

    fn takes(&self, extension: &str) -> bool {
        self.entries.iter().any(|s| s.extensions.iter().any(|e| equals_ignore_case(e, extension)))
    }

    fn candidates(&self, extension: &str) -> Vec<&'static emusen_systems::System> {
        self.entries.iter().copied().filter(|s| s.extensions.iter().any(|e| equals_ignore_case(e, extension))).collect()
    }
}

/// The claims of each candidate that has a recogniser, by its place, the file read through Galaxia's image probe; a
/// probe that cannot be made claims nothing.
fn claims(at: &str, candidates: &[&emusen_systems::System], systems: &Systems) -> Vec<(usize, Recognition)> {
    let Ok(probe) = ImageProbe::of(at, &|e| systems.takes(e)) else { return Vec::new() };
    let ids: Vec<&str> = candidates.iter().map(|s| s.id.as_str()).collect();
    probe::claims(&ids, &mut |length| probe.read(length))
}

/// The extension that decides a file's system: an archive's image entry's, else its own, in lower case.
fn image_extension(at: &str, systems: &Systems) -> String {
    image_probe::image_extension(at, &|e| systems.takes(e))
}

/// `CoreCatalog.IsGameBoyColor`'s reading: the file's own extension, and its byte at 0x143 where it is longer.
fn shelved_as_color(at: &str) -> bool {
    let byte = head_of(at, 0x144).ok().filter(|h| h.len() > 0x143).map(|h| h[0x143]);
    emusen_systems::gb::shelved_as_color(path::extension(Style::HOST, at), byte)
}

/// The system a file is shelved as, among the named systems: the only one taking its image's extension, else the one
/// routing by content and then by folder decides; a Game Boy game by its header.
pub fn route(at: &str, systems: &Systems) -> Option<String> {
    let extension = image_extension(at, systems);
    let candidates = systems.candidates(&extension);
    let chosen = match candidates.len() {
        0 => None,
        1 => Some(candidates[0].id.clone()),
        _ => {
            let claims = claims(at, &candidates, systems);
            let asked = claims.len() != 1;
            let folder = if asked { path::directory_name(Style::HOST, &path::full_path(at)).map(|d| path::file_name(Style::HOST, &d).to_string()) } else { None };
            let named: Vec<Candidate> = candidates.iter().map(|s| Candidate { id: &s.id, name: &s.name, esde_system: &s.esde_system }).collect();
            probe::route(path::file_name(Style::HOST, at), folder.as_deref(), &named, &claims).system
        }
    };
    match chosen.as_deref() {
        Some("gb" | "gbc") if systems.entries.iter().any(|s| s.id == "gbc") && systems.entries.iter().any(|s| s.id == "gb") => {
            Some(if shelved_as_color(at) { "gbc" } else { "gb" }.to_string())
        }
        _ => chosen,
    }
}

/// The system a package's manifest names, among the named systems: the one of its ES-DE name, else of its id.
fn system_named(manifest: &str, systems: &Systems) -> Option<String> {
    let by = |f: &dyn Fn(&emusen_systems::System) -> bool| systems.entries.iter().find(|s| f(s)).map(|s| s.id.clone());
    by(&|s| equals_ignore_case(&s.esde_system, manifest)).or_else(|| by(&|s| equals_ignore_case(&s.id, manifest)))
}

/// Walks the roots, keeping what was found in a `Scan`; `progress` is updated as it goes and asked whether to stop.
pub fn scan(roots: &[String], ids: &[String], progress: &Progress) -> Result<Scan, Error> {
    scan_with(roots, &Beside::default(), ids, progress)
}

/// The scan with packages (EmuSen_GamePackages.md §3.6): `Packages` and `Compressed` walked too where they are
/// folders; nothing inside a package's folder is a game of its own, a file a package claims is not listed, and each
/// package whose game is there is one entry, its path the game's and shelved by the manifest's system. A library
/// path of a manifest is under the first root. No file is read for a claim: the manifests are the index.
pub fn scan_with(roots: &[String], beside: &Beside, ids: &[String], progress: &Progress) -> Result<Scan, Error> {
    let systems = Systems::named(ids);
    let extensions = systems.extensions();
    let given: Vec<String> = roots.iter().filter(|r| !emusen_dotnet::text::is_blank(Some(r))).cloned().collect();
    let mut scan = Scan { outcome: Outcome::NoDirectory, roots: given.clone(), systems: ids.to_vec(), games: Vec::new() };
    if given.is_empty() {
        return Ok(scan);
    }
    let options = dotnet_files::Options { skip_hidden: true, ignore_inaccessible: true };
    let mut missing = false;
    let mut found: Vec<String> = Vec::new();
    progress.phase.store(1, Ordering::Relaxed);
    for root in &given {
        if !dotnet_files::directory_exists(root) {
            missing = true;
            continue;
        }
        let mut files = Vec::new();
        let mut stopped = false;
        let walked = dotnet_files::visit_files_with(root, options, |at, _| {
            if stopped || progress.cancelled() {
                stopped = true;
                return;
            }
            progress.files.fetch_add(1, Ordering::Relaxed);
            if extensions.iter().any(|e| equals_ignore_case(e, path::extension(Style::HOST, at))) {
                files.push(at.to_string());
            }
        });
        if stopped {
            return Err(Error::Cancelled);
        }
        match walked {
            Ok(()) => found.extend(image_sets::shelved(files)),
            Err(_) => missing = true,
        }
    }
    let index = beside.packages.as_deref().filter(|p| dotnet_files::directory_exists(p)).map(|packages| {
        let compressed = beside.compressed.clone().unwrap_or_default();
        for folder in [packages, compressed.as_str()].into_iter().filter(|f| dotnet_files::directory_exists(f)) {
            let mut files = Vec::new();
            let _ = dotnet_files::visit_files_with(folder, options, |at, _| {
                if extensions.iter().any(|e| equals_ignore_case(e, path::extension(Style::HOST, at))) {
                    files.push(at.to_string());
                }
            });
            found.extend(image_sets::shelved(files));
        }
        claims::index(&claims::Roots { packages, library: given.first().map(String::as_str), compressed: &compressed })
    });
    if let Some(index) = &index {
        found.retain(|at| {
            let full = path::full_path(at);
            let zip_of_folder = archive::is_archive(&full) && index.folders.iter().any(|f| package_zip::zip_beside(f) == full);
            !index.inside_package(&full) && index.claimant(&full).is_none() && !zip_of_folder
        });
    }
    let packages_root = beside.packages.as_deref().map(path::full_path);
    progress.phase.store(2, Ordering::Relaxed);
    for at in found {
        if progress.cancelled() {
            return Err(Error::Cancelled);
        }
        let zipped = packages_root.as_deref().filter(|p| archive::is_archive(&at) && claims::within(&path::full_path(&at), p)).and_then(|_| package_zip::manifest_in(&at).ok().flatten());
        let system = match zipped {
            Some((_, doc)) => doc.get("system").and_then(|v| v.as_str()).and_then(|s| system_named(s, &systems)),
            None => route(&at, &systems),
        };
        progress.games.fetch_add(1, Ordering::Relaxed);
        scan.games.push(Game { path: at, system, package: None });
    }
    for p in index.iter().flat_map(|i| i.packages.iter()).filter(|p| dotnet_files::file_exists(&p.game)) {
        let system = system_named(&p.system, &systems).or_else(|| route(&p.game, &systems));
        progress.games.fetch_add(1, Ordering::Relaxed);
        scan.games.push(Game { path: p.game.clone(), system, package: Some(p.folder.clone()) });
    }
    scan.games.sort_by(|a, b| compare_ignore_case(path::file_name_without_extension(Style::HOST, &a.path), path::file_name_without_extension(Style::HOST, &b.path)));
    scan.outcome = if missing { Outcome::NotFound } else if scan.games.is_empty() { Outcome::Empty } else { Outcome::Ok };
    if missing && given.len() == 1 {
        scan.games.clear();
    }
    Ok(scan)
}

/// The scan narrowed to one system, its outcome made again: `RomLibrary.Narrow`.
pub fn narrow<'a>(scan: &'a Scan, system: Option<&str>) -> (Outcome, Vec<&'a Game>) {
    let games: Vec<&Game> = scan.games.iter().filter(|g| system.is_none_or(|s| g.system.as_deref() == Some(s))).collect();
    let outcome = match scan.outcome {
        Outcome::Ok | Outcome::Empty if system.is_some() => {
            if games.is_empty() { Outcome::Empty } else { Outcome::Ok }
        }
        other => other,
    };
    (outcome, games)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Temp(std::path::PathBuf);

    impl Temp {
        fn new(label: &str) -> Temp {
            let dir = std::env::temp_dir().join(format!("emusen-scan-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Temp(std::fs::canonicalize(&dir).unwrap())
        }

        fn at(&self, rel: &str) -> String {
            self.0.join(rel).to_string_lossy().into_owned()
        }

        fn put(&self, rel: &str, bytes: &[u8]) {
            let at = self.0.join(rel);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, bytes).unwrap();
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_package_is_one_entry_and_nothing_it_claims_or_holds_is_listed_again() {
        let t = Temp::new("packages");
        t.put("roms/nes/Claimed.nes", b"NES\x1aclaimed");
        t.put("roms/nes/Free.nes", b"NES\x1afree");
        t.put("roms/psx/Disc.cue", b"FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
        t.put("roms/psx/Disc.bin", &[0u8; 2352]);
        t.put("compressed/psx/Disc.chd", b"not read");
        t.put("compressed/psx/Stray.chd", b"not read either");
        let manifest = |system: &str, game: &str, extra: &str| format!("{{\"format\":\"emusen-package\",\"version\":\"1.0\",\"system\":\"{system}\",\"mode\":\"linked\",\"game\":{game}{extra}}}");
        t.put("packages/NES/Claimed.nes/package.json", manifest("nes", "{\"role\":\"image\",\"where\":\"library\",\"path\":\"nes/Claimed.nes\"}", "").as_bytes());
        t.put("packages/NES/Claimed.nes/saves/Claimed.nes", b"NES\x1anot a game of its own");
        t.put(
            "packages/PlayStation/Disc.chd/package.json",
            manifest("psx", "{\"role\":\"image\",\"where\":\"compressed\",\"path\":\"psx/Disc.chd\"}", ",\"container\":{\"type\":\"chd\",\"sources\":[{\"where\":\"library\",\"path\":\"psx/Disc.cue\"}]}").as_bytes(),
        );
        t.put("packages/NES/Gone.nes/package.json", manifest("nes", "{\"role\":\"image\",\"where\":\"library\",\"path\":\"nes/Gone.nes\"}", "").as_bytes());
        t.put("packages/NES/Broken.nes/package.json", b"{ not json");
        t.put("packages/NES/Broken.nes/Inside.nes", b"NES\x1a");
        let ids: Vec<String> = ["nes", "psx"].iter().map(|s| s.to_string()).collect();
        let beside = Beside { packages: Some(t.at("packages")), compressed: Some(t.at("compressed")) };
        let found = scan_with(&[t.at("roms")], &beside, &ids, &Progress::default()).unwrap();
        let shown: Vec<(String, Option<String>, Option<String>)> = found.games.iter().map(|g| (g.path.clone(), g.system.clone(), g.package.clone())).collect();
        let expect = vec![
            (t.at("roms/nes/Claimed.nes"), Some("nes".to_string()), Some(t.at("packages/NES/Claimed.nes"))),
            (t.at("compressed/psx/Disc.chd"), Some("psx".to_string()), Some(t.at("packages/PlayStation/Disc.chd"))),
            (t.at("roms/nes/Free.nes"), Some("nes".to_string()), None),
            (t.at("compressed/psx/Stray.chd"), Some("psx".to_string()), None),
        ];
        assert_eq!(shown, expect);
        assert_eq!(found.outcome, Outcome::Ok);
        let plain = scan(&[t.at("roms")], &ids, &Progress::default()).unwrap();
        assert_eq!(plain.games.iter().map(|g| g.path.clone()).collect::<Vec<_>>(), [t.at("roms/nes/Claimed.nes"), t.at("roms/psx/Disc.cue"), t.at("roms/nes/Free.nes")]);
    }

    #[test]
    fn a_self_contained_package_is_one_entry_and_a_package_s_zip_is_listed_only_where_its_folder_is_not() {
        let t = Temp::new("self-contained");
        t.put("roms/nes/Copied.nes", b"NES\x1acopied");
        let manifest = |game: &str, extra: &str| format!("{{\"format\":\"emusen-package\",\"version\":\"1.0\",\"system\":\"nes\",\"mode\":\"self-contained\",\"game\":{game}{extra}}}");
        t.put("packages/NES/Copied.nes/Copied.nes", b"NES\x1acopied");
        t.put("packages/NES/Copied.nes/package.json", manifest("{\"role\":\"image\",\"where\":\"package\",\"path\":\"Copied.nes\"}", ",\"sources\":[{\"role\":\"image\",\"where\":\"library\",\"path\":\"nes/Copied.nes\"}]").as_bytes());
        package_zip::write(&t.at("packages/NES/Copied.nes"), &t.at("packages/NES/Copied.nes.zip"), &|| false).unwrap();
        t.put("elsewhere/Carried.nes/Carried.nes", b"NES\x1acarried");
        t.put("elsewhere/Carried.nes/package.json", manifest("{\"role\":\"image\",\"where\":\"package\",\"path\":\"Carried.nes\"}", "").as_bytes());
        package_zip::write(&t.at("elsewhere/Carried.nes"), &t.at("packages/Carried.nes.zip"), &|| false).unwrap();
        let ids: Vec<String> = ["nes", "psx"].iter().map(|s| s.to_string()).collect();
        let beside = Beside { packages: Some(t.at("packages")), compressed: Some(t.at("compressed")) };
        let found = scan_with(&[t.at("roms")], &beside, &ids, &Progress::default()).unwrap();
        let shown: Vec<(String, Option<String>, Option<String>)> = found.games.iter().map(|g| (g.path.clone(), g.system.clone(), g.package.clone())).collect();
        let expect = vec![
            (t.at("packages/Carried.nes.zip"), Some("nes".to_string()), None),
            (t.at("packages/NES/Copied.nes/Copied.nes"), Some("nes".to_string()), Some(t.at("packages/NES/Copied.nes"))),
        ];
        assert_eq!(shown, expect);
    }

}
