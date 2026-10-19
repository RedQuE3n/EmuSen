//! Where a game's picture is looked for, in order, `MediaSources.Locate` ported: another game's cover the player chose,
//! what the player placed, ScreenScraper's store, an ES-DE folder, OpenEmu's failover folder. Both frontends ask this,
//! so a game shows the same cover in each. See EmuSen_Settings_Reference.md §4.60, §4.65 and EmuSen_Backend.md §30.6.

use crate::Scraper;
use crate::consoles::{self, Core};
use crate::kinds;
use emusen_galaxia::dotnet_files::file_exists;
use emusen_galaxia::dotnet_path::{self as path, Style};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    OtherGame,
    Hand,
    ScreenScraper,
    EsdeFolder,
    OpenEmu,
}

impl Source {
    pub fn word(self) -> &'static str {
        match self {
            Source::OtherGame => "other_game",
            Source::Hand => "hand",
            Source::ScreenScraper => "screenscraper",
            Source::EsdeFolder => "esde",
            Source::OpenEmu => "openemu",
        }
    }
}

/// A game as the folders name it: its file, the folder it sits in below its console's, and whether it is a folder itself.
pub struct Game<'a> {
    pub file: &'a str,
    pub folder_path: String,
    pub folder: bool,
}

const STYLE: Style = Style::HOST;

/// Where below a type folder a game's files are named: its folder then its stem, and for a foldered game its stem alone.
fn names(game: &Game) -> Vec<String> {
    if game.folder {
        return if game.folder_path.is_empty() { Vec::new() } else { vec![game.folder_path.clone()] };
    }
    let stem = path::file_name_without_extension(STYLE, game.file).to_string();
    if game.folder_path.is_empty() { vec![stem] } else { vec![format!("{}/{stem}", game.folder_path), stem] }
}

/// An ES-DE layout's file for a game and type under `root`, `EsdeMediaFolder.Find`.
pub fn find_in(root: &str, system: &str, game: &Game, media_type: &str) -> Option<String> {
    let folder = kinds::by_type(media_type)?.folder;
    for name in names(game) {
        for ext in kinds::extensions(media_type) {
            let file = path::combine_all(STYLE, &[root, system, folder, &format!("{name}{ext}")]);
            if file_exists(&file) {
                return Some(file);
            }
        }
    }
    None
}

/// Where `CoverFetcher` writes: `<root>/<console>/<libretro-safe stem>.<ext>`.
pub fn openemu_cover(root: &str, console: &str, title: &str) -> Option<String> {
    let stem = crate::openemu::safe(title);
    [".png", ".jpg", ".jpeg"].iter().map(|ext| path::combine_all(STYLE, &[root, console, &format!("{stem}{ext}")])).find(|p| file_exists(p))
}

impl Scraper {
    fn system_ids(cores: &[Core]) -> Vec<String> {
        cores.iter().flat_map(|c| c.systems.iter().cloned()).collect()
    }

    /// The table's system the runtime's shelves would put a file on, among the cores' systems.
    pub(crate) fn route(&self, rom: &str, cores: &[Core]) -> Option<&'static emusen_systems::System> {
        let ids = Scraper::system_ids(cores);
        emusen_library::scan::route(rom, &emusen_library::scan::Systems::named(&ids)).and_then(|id| emusen_systems::find(&id))
    }

    /// The console of the core a file opens on.
    pub(crate) fn console_for(&self, rom: &str, cores: &[Core]) -> Option<String> {
        let system = self.route(rom, cores)?;
        consoles::core_of(cores, &system.id).map(|c| c.console.clone())
    }

    /// The game whose cover a ROM shows, the player's choice in the library's edits; none with no library lent.
    fn cover_from(&self, rom: &str) -> Option<String> {
        let library = self.library().ok()?;
        let edits = library.get_request(&format!("{{\"what\":\"edits\",\"path\":{}}}", emusen_dotnet::json::quoted(rom))).ok()?;
        emusen_dotnet::json::parse(&edits).ok()?.get("coverfrom").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string)
    }

    /// The picture of a type for a game on the shelf ES-DE calls `esde_system`, and where it came from.
    pub fn locate(&self, esde_system: &str, game: &Game, media_type: &str) -> Option<(Source, String)> {
        let settings = self.settings();
        let cores = consoles::cores(&settings.cores);
        let mut seen = HashSet::new();
        seen.insert(game.file.to_string());
        self.locate_in(esde_system, game, media_type, &cores, &mut seen)
    }

    fn locate_in(&self, esde_system: &str, game: &Game, media_type: &str, cores: &[Core], seen: &mut HashSet<String>) -> Option<(Source, String)> {
        let settings = self.settings();
        let cover = media_type == kinds::COVER.esde_type;
        let file = !game.folder;
        if cover && file {
            if let Some(from) = self.cover_from(game.file) {
                if seen.insert(from.clone()) {
                    if let Some(system) = self.route(&from, cores) {
                        let next = Game { file: &from, folder_path: settings.folder_of(&from), folder: false };
                        if let Some((_, borrowed)) = self.locate_in(&system.esde_system, &next, media_type, cores, seen) {
                            return Some((Source::OtherGame, borrowed));
                        }
                    }
                }
            }
        }
        let title = path::file_name_without_extension(STYLE, game.file).to_string();
        let console = if file { self.console_for(game.file, cores) } else { None };
        if cover {
            if let Some(console) = &console {
                if let Some(hand) = self.art().find(Some(console), &title) {
                    return Some((Source::Hand, hand));
                }
            }
        }
        if let Some(scraped) = find_in(&self.root(), esde_system, game, media_type) {
            return Some((Source::ScreenScraper, scraped));
        }
        if let Some(esde) = settings.esde_media.as_deref().filter(|e| !emusen_dotnet::text::is_blank(Some(e))).and_then(|root| find_in(root, esde_system, game, media_type)) {
            return Some((Source::EsdeFolder, esde));
        }
        if cover && settings.openemu {
            if let Some(console) = &console {
                if let Some(found) = openemu_cover(&self.openemu_root(), console, &title) {
                    return Some((Source::OpenEmu, found));
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
        let dir = path::combine_all(STYLE, &[root, system, kind.folder]);
        if !emusen_galaxia::dotnet_files::directory_exists(&dir) {
            continue;
        }
        let names = names.get_or_insert_with(|| games.iter().flat_map(|(file, folder_path, folder)| names_of(file, folder_path, *folder)).collect());
        let extensions = kinds::extensions(kind.esde_type);
        let mut hit = false;
        let mut look = |file: &str| {
            let rel = file[dir.len()..].trim_start_matches(STYLE.separator());
            let parent = path::directory_name(STYLE, rel).unwrap_or_default();
            let name = path::combine(STYLE, &parent, path::file_name_without_extension(STYLE, rel)).replace(STYLE.separator(), "/");
            if extensions.contains(&path::extension(STYLE, file)) && names.contains(&name) {
                hit = true;
            }
        };
        if nested {
            let _ = emusen_galaxia::dotnet_files::visit_files(&dir, |f, _| look(f));
        } else if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                if !emusen_galaxia::dotnet_files::listed_as_directory(&e) {
                    look(&emusen_galaxia::dotnet_files::join(&dir, &e.file_name().to_string_lossy()));
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
            let dir = path::combine_all(STYLE, &[root, system, kind.folder]);
            if !emusen_galaxia::dotnet_files::directory_exists(&dir) {
                return "0".to_string();
            }
            let mut stamps = Vec::new();
            let mut queue = std::collections::VecDeque::from([dir]);
            while let Some(d) = queue.pop_front() {
                if let Ok(m) = std::fs::metadata(&d) {
                    stamps.push(emusen_galaxia::dotnet_files::write_ticks(&m).to_string());
                }
                if let Ok(entries) = std::fs::read_dir(&d) {
                    for e in entries.flatten() {
                        if emusen_galaxia::dotnet_files::listed_as_directory(&e) {
                            queue.push_back(emusen_galaxia::dotnet_files::join(&d, &e.file_name().to_string_lossy()));
                        }
                    }
                }
            }
            stamps.join(";")
        })
        .collect::<Vec<_>>()
        .join(",")
}

impl Scraper {
    /// `MediaSources.Present`: the types some game of a shelf has, from the store and an ES-DE folder, and a cover from
    /// any source counted too.
    pub fn present(&self, system: &str, games: &[(String, String, bool)]) -> Vec<String> {
        let settings = self.settings();
        let mut found: Vec<String> = present_in(&self.root(), system, games).into_iter().map(str::to_string).collect();
        if let Some(esde) = settings.esde_media.as_deref().filter(|e| !emusen_dotnet::text::is_blank(Some(e))) {
            for t in present_in(esde, system, games) {
                if !found.iter().any(|f| f == t) {
                    found.push(t.to_string());
                }
            }
        }
        let cover = kinds::COVER.esde_type;
        if !found.iter().any(|f| f == cover)
            && games.iter().any(|(file, folder_path, folder)| self.locate(system, &Game { file, folder_path: folder_path.clone(), folder: *folder }, cover).is_some())
        {
            found.push(cover.to_string());
        }
        found
    }

    /// `MediaSources.Stamp`: changes when a file is added to or taken from any folder a picture could come from; the
    /// player's choices are the host's to key, given as `choice`.
    pub fn stamp(&self, system: &str, choice: &str) -> String {
        let settings = self.settings();
        let openemu = self.openemu_root();
        let openemu_stamp = if settings.openemu && emusen_galaxia::dotnet_files::directory_exists(&openemu) {
            std::fs::read_dir(&openemu)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|e| emusen_galaxia::dotnet_files::listed_as_directory(e))
                        .filter_map(|e| std::fs::metadata(e.path()).ok().map(|m| emusen_galaxia::dotnet_files::write_ticks(&m).to_string()))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default()
        } else {
            String::new()
        };
        let esde = settings.esde_media.as_deref().filter(|e| !emusen_dotnet::text::is_blank(Some(e))).map(|e| stamp_in(e, system)).unwrap_or_default();
        format!("{}|{esde}|{openemu_stamp}|{choice}", stamp_in(&self.root(), system))
    }
}
