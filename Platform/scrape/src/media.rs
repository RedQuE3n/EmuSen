//! Where a game's picture is looked for, in order, `MediaSources.Locate` ported: another game's cover the player chose,
//! what the player placed, a packaged game's package, ScreenScraper's store, an ES-DE folder, OpenEmu's failover folder. Both frontends ask this,
//! so a game shows the same cover in each. See EmuSen_Settings_Reference.md §4.60, §4.65 and EmuSen_Backend.md §30.6.

use std::path::Path;
use crate::{Error, ScraperInner};
use crate::consoles::{self, Core};
use crate::kinds;
use emusen_galaxia::files::file_exists;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    OtherGame,
    Hand,
    ScreenScraper,
    EsdeFolder,
    OpenEmu,
    Package,
}

impl Source {
    pub fn word(self) -> &'static str {
        match self {
            Source::OtherGame => "other_game",
            Source::Hand => "hand",
            Source::ScreenScraper => "screenscraper",
            Source::EsdeFolder => "esde",
            Source::OpenEmu => "openemu",
            Source::Package => "package",
        }
    }
}

/// A picture found for a game: where it came from, its file, the media type it is a file of, and for a cover borrowed
/// from another game, that game's file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub source: Source,
    pub path: String,
    pub media_type: String,
    pub from: Option<String>,
}

impl Found {
    fn of(source: Source, path: String, media_type: &str) -> Found {
        Found { source, path, media_type: media_type.to_string(), from: None }
    }
}

/// A game as the folders name it: its file, the folder it sits in below its console's, and whether it is a folder itself.
pub struct Game<'a> {
    pub file: &'a str,
    pub folder_path: String,
    pub folder: bool,
}

/// Where below a type folder a game's files are named: its folder then its stem, and for a foldered game its stem alone.
fn names(game: &Game) -> Vec<String> {
    if game.folder {
        return if game.folder_path.is_empty() { Vec::new() } else { vec![game.folder_path.clone()] };
    }
    let stem = Path::new(game.file).file_stem().unwrap_or_default().to_string_lossy().to_string();
    if game.folder_path.is_empty() { vec![stem] } else { vec![format!("{}/{stem}", game.folder_path), stem] }
}

/// An ES-DE layout's file for a game and type under `root`, `EsdeMediaFolder.Find`.
pub fn find_in(root: &str, system: &str, game: &Game, media_type: &str) -> Option<String> {
    let folder = kinds::by_type(media_type)?.folder;
    for name in names(game) {
        for ext in kinds::extensions(media_type) {
            let file = Path::new(root).join(system).join(folder).join(&format!("{name}{ext}")).to_string_lossy().into_owned();
            if file_exists(&file) {
                return Some(file);
            }
        }
    }
    None
}

/// A packaged game's file of a type in its package, `EsdeMediaFolder.FindInPackage` (EmuSen_GamePackages.md §18.3).
pub fn find_in_package(root: &str, media_type: &str) -> Option<String> {
    let folder = kinds::by_type(media_type)?.folder;
    kinds::extensions(media_type).iter().map(|ext| emusen_galaxia::package::media_path(root, folder, ext).to_string_lossy().into_owned()).find(|p| file_exists(p))
}

/// Each packaged game's media folder and its type folders' write times, `EsdeMediaFolder.PackageStamp`.
fn package_stamp(roots: &[&str]) -> String {
    roots
        .iter()
        .map(|r| {
            let dir = Path::new(r).join(emusen_galaxia::package::MEDIA_FOLDER).to_string_lossy().into_owned();
            if !emusen_galaxia::files::directory_exists(&dir) {
                return "0".to_string();
            }
            let mut subs: Vec<String> = std::fs::read_dir(&dir)
                .map(|entries| entries.flatten().filter(|e| emusen_galaxia::files::listed_as_directory(e)).map(|e| Path::new(&dir).join(e.file_name()).to_string_lossy().into_owned()).collect())
                .unwrap_or_default();
            subs.sort();
            std::iter::once(dir).chain(subs).filter_map(|d| std::fs::metadata(&d).ok().map(|m| emusen_galaxia::files::write_ticks(&m).to_string())).collect::<Vec<_>>().join(";")
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Where `CoverFetcher` writes: `<root>/<console>/<libretro-safe stem>.<ext>`.
pub fn openemu_cover(root: &str, console: &str, title: &str) -> Option<String> {
    let stem = crate::openemu::safe(title);
    [".png", ".jpg", ".jpeg"].iter().map(|ext| Path::new(root).join(console).join(&format!("{stem}{ext}")).to_string_lossy().into_owned()).find(|p| file_exists(p))
}

impl ScraperInner {
    fn system_ids(cores: &[Core]) -> Vec<String> {
        cores.iter().flat_map(|c| c.systems.iter().cloned()).collect()
    }

    /// The table's system the runtime's shelves would put a file on, among the cores' systems.
    pub(crate) fn route(&self, rom: &str, cores: &[Core]) -> Option<&'static emusen_systems::System> {
        let ids = ScraperInner::system_ids(cores);
        emusen_library::scan::route(rom, &emusen_library::scan::Systems::named(&ids)).and_then(|id| emusen_systems::find(&id))
    }

    /// The console of the core a file opens on.
    pub(crate) fn console_for(&self, rom: &str, cores: &[Core]) -> Option<String> {
        let system = self.route(rom, cores)?;
        consoles::core_of(cores, &system.id).map(|c| c.console.clone())
    }

    /// The player's cover choices for a ROM in the library's edits: the game whose cover it shows, and the type of its own
    /// picture shown as its cover. A handle with no library lent has no place a choice could be kept, so it has none; an
    /// edits read that fails is an error.
    fn cover_choices(&self, rom: &str) -> Result<(Option<String>, Option<String>), Error> {
        let Ok(library) = self.library() else { return Ok((None, None)) };
        let edits = library.get_request(&serde_json::json!({ "what": "edits", "path": rom }).to_string()).map_err(|e| Error::Io(e.words()))?;
        let edits: serde_json::Value = serde_json::from_str(&edits).map_err(|e| Error::Io(format!("The library's edits of {rom} could not be read: {e}.")))?;
        let field = |name: &str| edits.get(name).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
        Ok((field(emusen_library::records::COVER_FROM), field(emusen_library::records::COVER_TYPE)))
    }

    /// The picture of a type for a game on the shelf ES-DE calls `esde_system`, as shown, and where it came from.
    pub fn locate(&self, esde_system: &str, game: &Game, media_type: &str) -> Result<Option<(Source, String)>, Error> {
        Ok(self.found(esde_system, game, media_type, true)?.map(|f| (f.source, f.path)))
    }

    /// `locate`, with the type of the file found: for a cover, the type of the game's own picture chosen as its cover.
    /// Without `choices`, a cover is the one found for the game, as though the player had chosen none, and nothing is read.
    pub fn found(&self, esde_system: &str, game: &Game, media_type: &str, choices: bool) -> Result<Option<Found>, Error> {
        let cores = consoles::cores(&self.settings().cores);
        if !choices {
            return Ok(self.found_in(esde_system, game, media_type, &cores));
        }
        let mut seen = HashSet::new();
        seen.insert(game.file.to_string());
        self.chosen_in(esde_system, game, media_type, &cores, &mut seen)
    }

    /// Makes one of a game's own pictures, by its type, its cover, or with none puts the cover back to the one found for
    /// it; a type that is not a picture, or one the game has no file of, is refused and nothing is written. Either
    /// choice ends a cover borrowed from another game. The files are never touched (EmuSen_Backend.md §30.6).
    pub(crate) fn choose_cover(&self, esde_system: &str, game: &Game, media_type: Option<&str>, at: Option<&str>) -> Result<(), Error> {
        let library = self.library()?;
        let chosen = media_type.filter(|t| *t != kinds::COVER.esde_type);
        if let Some(t) = chosen {
            if !kinds::may_be_cover(t) {
                return Err(Error::BadRequest(format!("{t} is not a kind of picture, so it cannot be a game's cover.")));
            }
            if game.folder || self.found(esde_system, game, t, false)?.is_none() {
                let title = Path::new(game.file).file_stem().unwrap_or_default().to_string_lossy().to_string();
                return Err(Error::BadRequest(format!("{title} has no picture of the type {t} to be its cover.")));
            }
        }
        let mut changes = serde_json::Map::new();
        changes.insert(emusen_library::records::COVER_TYPE.into(), chosen.map_or(serde_json::Value::Null, |t| t.into()));
        changes.insert(emusen_library::records::COVER_FROM.into(), serde_json::Value::Null);
        let mut request = serde_json::json!({ "op": "save_edits", "path": game.file, "changes": changes });
        if let Some(at) = at {
            request["at"] = at.into();
        }
        library.change_request(&request.to_string()).map(drop).map_err(|e| Error::Io(e.words()))
    }

    /// A picture as the player's choices give it: a game's cover borrowed from another game, else its own picture chosen
    /// while it has a file of it; else, and for every other type, the picture found.
    fn chosen_in(&self, esde_system: &str, game: &Game, media_type: &str, cores: &[Core], seen: &mut HashSet<String>) -> Result<Option<Found>, Error> {
        if media_type == kinds::COVER.esde_type && !game.folder {
            let (from, chosen) = self.cover_choices(game.file)?;
            if let Some(from) = from
                && seen.insert(from.clone())
                && let Some(system) = self.route(&from, cores)
            {
                let next = Game { file: &from, folder_path: self.settings().folder_of(&from), folder: false };
                if let Some(borrowed) = self.chosen_in(&system.esde_system, &next, media_type, cores, seen)? {
                    return Ok(Some(Found { source: Source::OtherGame, from: Some(from.clone()), ..borrowed }));
                }
            }
            if let Some(own) = chosen.filter(|t| *t != media_type && kinds::may_be_cover(t)).and_then(|t| self.found_in(esde_system, game, &t, cores)) {
                return Ok(Some(own));
            }
        }
        Ok(self.found_in(esde_system, game, media_type, cores))
    }

    /// The picture of a type found for a game in the sources' order, whatever the player chose.
    pub(crate) fn found_in(&self, esde_system: &str, game: &Game, media_type: &str, cores: &[Core]) -> Option<Found> {
        let settings = self.settings();
        let cover = media_type == kinds::COVER.esde_type;
        let file = !game.folder;
        let title = Path::new(game.file).file_stem().unwrap_or_default().to_string_lossy().to_string();
        let console = if file { self.console_for(game.file, cores) } else { None };
        if cover {
            if let Some(console) = &console {
                if let Some(hand) = self.art().find(Some(console), &title) {
                    return Some(Found::of(Source::Hand, hand, media_type));
                }
            }
        }
        if file
            && let Some(package) = settings.package_of(game.file)
            && let Some(packaged) = find_in_package(package, media_type)
        {
            return Some(Found::of(Source::Package, packaged, media_type));
        }
        let stored = Game { file: game.file, folder_path: settings.store_folder(game.file, &game.folder_path, game.folder), folder: game.folder };
        let game = &stored;
        if let Some(scraped) = find_in(&self.root(), esde_system, game, media_type) {
            return Some(Found::of(Source::ScreenScraper, scraped, media_type));
        }
        if let Some(esde) = settings.esde_media.as_deref().filter(|e| !e.trim().is_empty()).and_then(|root| find_in(root, esde_system, game, media_type)) {
            return Some(Found::of(Source::EsdeFolder, esde, media_type));
        }
        if cover && settings.openemu {
            if let Some(console) = &console {
                if let Some(found) = openemu_cover(&self.openemu_root(), console, &title) {
                    return Some(Found::of(Source::OpenEmu, found, media_type));
                }
            }
        }
        None
    }
}

/// `EsdeMediaFolder.Present`: the types some game has a file of under `root`, one listing a type folder.
fn present_in(root: &str, system: &str, games: &[(String, String, bool)]) -> Vec<&'static str> {
    let mut found = Vec::new();
    let nested = games.iter().any(|g| !g.1.is_empty());
    let mut names: Option<HashSet<String>> = None;
    for kind in kinds::ALL_FOLDER_ORDER {
        let dir = Path::new(root).join(system).join(kind.folder).to_string_lossy().into_owned();
        if !emusen_galaxia::files::directory_exists(&dir) {
            continue;
        }
        let names = names.get_or_insert_with(|| games.iter().flat_map(|(file, folder_path, folder)| names_of(file, folder_path, *folder)).collect());
        let extensions = kinds::extensions(kind.esde_type);
        let mut hit = false;
        let mut look = |file: &Path| {
            let Some(rel) = file.strip_prefix(&dir).ok() else { return };
            let name = rel.with_extension("").to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            let dotted = file.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
            if extensions.contains(&dotted.as_str()) && names.contains(&name) {
                hit = true;
            }
        };
        if nested {
            let _ = emusen_galaxia::files::visit_files(&dir, |f, _| look(f));
        } else if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                if !emusen_galaxia::files::listed_as_directory(&e) {
                    look(&Path::new(&dir).join(e.file_name()));
                }
            }
        }
        if hit {
            found.push(kind.esde_type);
        }
    }
    found
}

fn names_of(file: &str, folder_path: &str, folder: bool) -> Vec<String> {
    names(&Game { file, folder_path: folder_path.to_string(), folder })
}

/// Each type folder and its own folders' write times, a text that changes when a file is added or taken.
fn stamp_in(root: &str, system: &str) -> String {
    kinds::ALL_FOLDER_ORDER
        .iter()
        .map(|kind| {
            let dir = Path::new(root).join(system).join(kind.folder).to_string_lossy().into_owned();
            if !emusen_galaxia::files::directory_exists(&dir) {
                return "0".to_string();
            }
            let mut stamps = Vec::new();
            let mut queue = std::collections::VecDeque::from([dir]);
            while let Some(d) = queue.pop_front() {
                if let Ok(m) = std::fs::metadata(&d) {
                    stamps.push(emusen_galaxia::files::write_ticks(&m).to_string());
                }
                if let Ok(entries) = std::fs::read_dir(&d) {
                    for e in entries.flatten() {
                        if emusen_galaxia::files::listed_as_directory(&e) {
                            queue.push_back(Path::new(&d).join(e.file_name()).to_string_lossy().into_owned());
                        }
                    }
                }
            }
            stamps.join(";")
        })
        .collect::<Vec<_>>()
        .join(",")
}

impl ScraperInner {
    /// `MediaSources.Present`: the types some game of a shelf has, from the store and an ES-DE folder, and a cover from
    /// any source counted too, as shown, the player's choices with it.
    pub fn present(&self, system: &str, games: &[(String, String, bool)]) -> Result<Vec<String>, Error> {
        let settings = self.settings();
        let stored: Vec<(String, String, bool)> = games.iter().map(|(f, p, d)| (f.clone(), settings.store_folder(f, p, *d), *d)).collect();
        let mut found: Vec<String> = present_in(&self.root(), system, &stored).into_iter().map(str::to_string).collect();
        if let Some(esde) = settings.esde_media.as_deref().filter(|e| !e.trim().is_empty()) {
            for t in present_in(esde, system, &stored) {
                if !found.iter().any(|f| f == t) {
                    found.push(t.to_string());
                }
            }
        }
        for (file, _, folder) in games {
            let Some(package) = settings.package_of(file).filter(|_| !*folder) else { continue };
            for kind in kinds::ALL_FOLDER_ORDER {
                if !found.iter().any(|f| f == kind.esde_type) && find_in_package(package, kind.esde_type).is_some() {
                    found.push(kind.esde_type.to_string());
                }
            }
        }
        let cover = kinds::COVER.esde_type;
        if !found.iter().any(|f| f == cover) {
            for (file, folder_path, folder) in games {
                if self.locate(system, &Game { file, folder_path: folder_path.clone(), folder: *folder }, cover)?.is_some() {
                    found.push(cover.to_string());
                    break;
                }
            }
        }
        Ok(found)
    }

    /// `MediaSources.Stamp`: changes when a file is added to or taken from any folder a picture could come from; the
    /// player's choices are the host's to key, given as `choice`.
    pub fn stamp(&self, system: &str, choice: &str) -> String {
        let settings = self.settings();
        let openemu = self.openemu_root();
        let openemu_stamp = if settings.openemu && emusen_galaxia::files::directory_exists(&openemu) {
            std::fs::read_dir(&openemu)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|e| emusen_galaxia::files::listed_as_directory(e))
                        .filter_map(|e| std::fs::metadata(e.path()).ok().map(|m| emusen_galaxia::files::write_ticks(&m).to_string()))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default()
        } else {
            String::new()
        };
        let esde = settings.esde_media.as_deref().filter(|e| !e.trim().is_empty()).map(|e| stamp_in(e, system)).unwrap_or_default();
        let mut roots: Vec<&str> = settings.packages.iter().map(|(_, f)| f.as_str()).collect();
        roots.sort();
        format!("{}|{esde}|{openemu_stamp}|{choice}|{}", stamp_in(&self.root(), system), package_stamp(&roots))
    }
}
