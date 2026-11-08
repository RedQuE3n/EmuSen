//! The library's disc games as the tool sees them: found under the folders and files named, routed to their system
//! by the library's own scan, and each prepared for chdman in a scratch folder of links, since chdman 0.289 refused a
//! sheet naming a library file by its absolute path (EmuSen_GamePackages.md §4.2). The library is only read.

use emusen_disc::cue::{FileKind, Mode, Sheet};
use emusen_disc::Kind;
use emusen_galaxia::files::{self, Options};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::Tree;
use emusen_galaxia::{archive, disc_set, image_sets};

/// The systems whose discs the tool converts: the compact-disc consoles with a core, and the PlayStation 2 (§4.2).
pub const SYSTEMS: [&str; 3] = ["psx", "mcd", "ps2"];
/// Of those, the systems whose drive reads a DVD, and whose core opens an image of 2,048-byte sectors as one (§4.2a).
pub const DVD_SYSTEMS: [&str; 1] = ["ps2"];

/// What a library file holding a disc is.
const DISC_EXTENSIONS: [&str; 5] = [".cue", ".bin", ".iso", ".zip", ".m3u"];

/// One disc game of the library: its path relative to the library with `/` between parts, its full path, and its system.
#[derive(Clone, Debug)]
pub struct Item {
    pub rel: String,
    pub path: String,
    pub system: &'static emusen_systems::System,
}

/// A path the request named, relative to the library: `/` or `\` between parts, no `..`, empty for the whole library.
pub fn relative(given: &str) -> Result<String, String> {
    let parts: Vec<&str> = given.split(['/', '\\']).filter(|p| !p.is_empty() && *p != ".").collect();
    if parts.contains(&"..") {
        return Err(format!("{given}: a path in the library has no .. part"));
    }
    Ok(parts.join("/"))
}

fn full(library: &str, rel: &str) -> String {
    rel.split('/').filter(|p| !p.is_empty()).fold(library.to_string(), |at, part| path::combine(Style::HOST, &at, part))
}

/// A full path under the library back to its relative form.
pub fn rel_of(library: &str, at: &str) -> String {
    let root = path::trim_ending_separator(Style::HOST, library);
    at.strip_prefix(root).unwrap_or(at).trim_start_matches(['/', '\\']).replace('\\', "/")
}

/// The disc games under each named path, in order of their relative path ignoring case, each once; the words for a
/// path that names nothing in the library.
pub fn items(library: &str, given: &[String]) -> (Vec<Item>, Vec<String>) {
    let mut files: Vec<String> = Vec::new();
    let mut refused = Vec::new();
    let named: Vec<String> = if given.is_empty() { vec![String::new()] } else { given.to_vec() };
    for g in &named {
        let rel = match relative(g) {
            Ok(rel) => rel,
            Err(words) => {
                refused.push(words);
                continue;
            }
        };
        let at = full(library, &rel);
        if files::directory_exists(&at) {
            let _ = files::visit_files_with(&at, Options { skip_hidden: true, ignore_inaccessible: true }, |f, _| files.push(f.to_string()));
        } else if files::file_exists(&at) {
            files.push(at);
        } else {
            refused.push(format!("{g}: no such file or folder in the library"));
        }
    }
    files.sort();
    files.dedup();
    let systems = crate::packages::every_system();
    let mut found: Vec<Item> = image_sets::shelved(files).into_iter().filter_map(|at| item_of(library, at, &systems)).collect();
    found.sort_by(|a, b| a.rel.to_uppercase().cmp(&b.rel.to_uppercase()).then_with(|| a.rel.cmp(&b.rel)));
    (found, refused)
}

/// A library file as the disc the tool converts, or none: a sheet, track, ISO, zip or playlist of a system it converts.
fn item_of(library: &str, at: String, systems: &emusen_library::scan::Systems) -> Option<Item> {
    if !DISC_EXTENSIONS.iter().any(|e| at.to_uppercase().ends_with(&e.to_uppercase())) {
        return None;
    }
    let rel = rel_of(library, &at);
    let id = emusen_library::scan::route(&at, systems).or_else(|| by_folder(&rel))?;
    if !SYSTEMS.contains(&id.as_str()) {
        return None;
    }
    let system = emusen_systems::find(&id)?;
    Some(Item { rel, path: at, system })
}

/// Whether `chd convert` takes a library file, as `items` would find it (EmuSen_Packages.md §3a); a playlist is not
/// converted as one, so it is not offered.
pub fn convertible(library: &str, at: &str, systems: &emusen_library::scan::Systems) -> bool {
    !at.to_uppercase().ends_with(".M3U") && item_of(library, at.to_string(), systems).is_some()
}

/// Where the scan's routing names no system (no recogniser claimed the disc, and the folder holding it is no ES-DE
/// system's), the nearest folder above it that is one of the converted systems', as a library sorted by console has it.
fn by_folder(rel: &str) -> Option<String> {
    let folders: Vec<&str> = rel.split('/').collect();
    folders[..folders.len().saturating_sub(1)].iter().rev().find_map(|f| {
        SYSTEMS.iter().filter_map(|id| emusen_systems::find(id)).find(|s| s.esde_system.eq_ignore_ascii_case(f)).map(|s| s.id.clone())
    })
}

/// The form a dump was in, as the manifest's `dump.form` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    CueBin,
    LoneTrack,
    Iso,
}

impl Form {
    pub fn word(self) -> &'static str {
        match self {
            Form::CueBin => "cue-bin",
            Form::LoneTrack => "lone-track",
            Form::Iso => "iso",
        }
    }
}

/// A disc made ready for chdman: the sheet it reads, the files that sheet names, and what the library held.
pub struct Prepared {
    pub form: Form,
    /// What chdman writes it as: a compact disc from the sheet, or a DVD from its one file of 2,048-byte sectors (§4.2a).
    pub medium: Kind,
    /// The sheet chdman reads: the library's own for a sheet, else one line made for the lone image.
    pub sheet: String,
    /// The library's sheet, its file name and bytes, where the dump has one.
    pub original: Option<(String, Vec<u8>)>,
    /// Each file the sheet names, as it names it, and its full path.
    pub names: Vec<String>,
    pub files: Vec<String>,
    /// The sheet's place in the scratch folder, beside a link named as the sheet names each file.
    pub scratch_sheet: String,
    /// The library files read for this disc, whose size and time must not change during the conversion.
    pub sources: Vec<String>,
}

/// What a lone image of 2,048-byte sectors on `system` is written as (§4.2a): a DVD where the system's core opens such an
/// image as one, so that the CHD is the medium the game saw; a compact disc elsewhere.
fn lone_medium(system: &str) -> Kind {
    if DVD_SYSTEMS.contains(&system) { Kind::Dvd } else { Kind::Cd }
}

/// The sheet `TRACK` line for a lone image: a raw data track by its sync and mode byte, else an ISO 9660 image's
/// 2,048-byte sectors; anything else is refused, since a lone audio track is no game.
fn lone_mode(at: &str, size: u64) -> Result<(&'static str, Form), String> {
    let head = emusen_galaxia::image_probe::head_of(at, 16 * 2048 + 8).map_err(|e| format!("{at}: {e}"))?;
    const SYNC: [u8; 12] = [0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0];
    if head.len() >= 16 && head[..12] == SYNC {
        if !size.is_multiple_of(2352) {
            return Err(format!("{} is not a whole number of 2,352-byte sectors", path::file_name(Style::HOST, at)));
        }
        return Ok((if head[15] == 1 { "MODE1/2352" } else { "MODE2/2352" }, Form::LoneTrack));
    }
    if head.get(16 * 2048 + 1..16 * 2048 + 6) == Some(b"CD001") {
        if !size.is_multiple_of(2048) {
            return Err(format!("{} is not a whole number of 2,048-byte sectors", path::file_name(Style::HOST, at)));
        }
        return Ok(("MODE1/2048", Form::Iso));
    }
    Err(format!("{} is neither a raw data track nor an ISO 9660 image", path::file_name(Style::HOST, at)))
}

fn link(target: &str, at: &std::path::Path) -> Result<(), String> {
    if let Some(dir) = at.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(target, at);
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_file(target, at);
    made.map_err(|e| format!("a link in the scratch folder could not be made: {e}"))
}

/// The image a library file stands for: itself, or for a zip the entry play opens, unpacked into `Unpacked` once. A
/// zip of several tracks and no sheet is refused before anything is unpacked.
fn image_of(tree: &Tree, at: &str) -> Result<String, String> {
    if !archive::is_archive(at) {
        return Ok(at.to_string());
    }
    let names: Vec<String> = archive::entries_of(at)?.into_iter().map(|e| e.full_name).filter(|n| !n.ends_with('/')).collect();
    let tracks = names.iter().filter(|n| n.to_uppercase().ends_with(".BIN")).count();
    if !names.iter().any(|n| image_sets::is_sheet(n)) && tracks > 1 {
        return Err(format!("{} holds {tracks} track files and no sheet, so its tracks' order and gaps are unknown", path::file_name(Style::HOST, at)));
    }
    disc_set::unpack(tree, at, &|e| [".bin", ".iso"].iter().any(|x| x.eq_ignore_ascii_case(e)))
}

/// A library disc made ready in `scratch`, which this makes and the caller removes.
pub fn prepare(tree: &Tree, item: &Item, scratch: &str) -> Result<Prepared, String> {
    if image_sets::is_playlist(&item.path) {
        return Err("an .m3u is not converted by this step: name each of its discs, and the list is left as it is".into());
    }
    let image = image_of(tree, &item.path)?;
    let (form, sheet, original, names, files, medium) = if image_sets::is_sheet(&image) {
        let bytes = std::fs::read(&image).map_err(|e| format!("{image}: {e}"))?;
        let text = emusen_galaxia::text_file::decode(&bytes);
        let parsed = Sheet::parse(&text).map_err(|e| e.to_string())?;
        if parsed.kinds.contains(&FileKind::Wave) {
            return Err("the sheet names a WAVE file, whose header a CHD does not keep: not converted by this step".into());
        }
        let files = image_sets::companions(&image)?;
        if let Some(gone) = files.iter().find(|f| !files::file_exists(f)) {
            return Err(format!("the sheet names {gone}, which is not there"));
        }
        // A sheet is a compact disc's, as every core opens one (§4.2a).
        (Form::CueBin, text, Some((path::file_name(Style::HOST, &image).to_string(), bytes)), parsed.files, files, Kind::Cd)
    } else {
        let size = std::fs::metadata(&image).map_err(|e| format!("{image}: {e}"))?.len();
        let (mode, form) = lone_mode(&image, size)?;
        let medium = if form == Form::Iso { lone_medium(&item.system.id) } else { Kind::Cd };
        let name = path::file_name(Style::HOST, &image).to_string();
        (form, format!("FILE \"{name}\" BINARY\n  TRACK 01 {mode}\n    INDEX 01 00:00:00\n"), None, vec![name], vec![image.clone()], medium)
    };
    std::fs::create_dir_all(scratch).map_err(|e| format!("the scratch folder {scratch} could not be made: {e}"))?;
    for (name, file) in names.iter().zip(&files) {
        let parts: Vec<&str> = name.split(['/', '\\']).filter(|p| !p.is_empty()).collect();
        if parts.is_empty() || parts.contains(&"..") || path::is_rooted(Style::HOST, name) {
            return Err(format!("the sheet names {name}, which is not a file beside it"));
        }
        let at = parts.iter().fold(std::path::PathBuf::from(scratch), |p, part| p.join(part));
        link(&path::full_path(file), &at)?;
    }
    let scratch_sheet = path::combine(Style::HOST, scratch, "disc.cue");
    std::fs::write(&scratch_sheet, &sheet).map_err(|e| e.to_string())?;
    let sources = match (archive::is_archive(&item.path), &original) {
        (true, _) => vec![item.path.clone()],
        (false, Some(_)) => std::iter::once(image.clone()).chain(files.iter().cloned()).collect(),
        (false, None) => vec![image.clone()],
    };
    Ok(Prepared { form, medium, sheet, original, names, files, scratch_sheet, sources })
}

/// The file form of a raw sector a track of `mode` stored: what its file held for it.
pub fn file_form(mode: Mode, raw: &[u8; 2352]) -> &[u8] {
    match mode {
        Mode::Mode1User => &raw[16..16 + 2048],
        Mode::Mode2Xa => &raw[16..],
        _ => &raw[..],
    }
}
