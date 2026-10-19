//! The scan, the C# `RomLibrary.Scan` with each file's shelf: the roots walked as .NET's `EnumerationOptions` have it,
//! the files of a named system's extensions or an archive's kept, a sheet's and a list's files taken as their game's,
//! each file routed among the named systems by `emusen-systems` over the bytes Galaxia reads, and the games ordered by
//! title ignoring case. See EmuSen_Settings_Reference.md §4.11, §4.46 and EmuSen_Backend.md §28.4.

use crate::{Error, Progress};
use emusen_dotnet::text::{compare_ignore_case, equals_ignore_case};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::image_probe::{self, ImageProbe, head_of};
use emusen_galaxia::{archive, dotnet_files, image_sets};
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

/// One game the scan found, and the system it is shelved as, none where no named system takes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub path: String,
    pub system: Option<String>,
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

/// Walks the roots, keeping what was found in a `Scan`; `progress` is updated as it goes and asked whether to stop.
pub fn scan(roots: &[String], ids: &[String], progress: &Progress) -> Result<Scan, Error> {
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
    progress.phase.store(2, Ordering::Relaxed);
    for at in found {
        if progress.cancelled() {
            return Err(Error::Cancelled);
        }
        let system = route(&at, &systems);
        progress.games.fetch_add(1, Ordering::Relaxed);
        scan.games.push(Game { path: at, system });
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
