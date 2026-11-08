//! The store's folders as ES-DE's guide lays them (`<system>/<type folder>/<game's folder>/<rom stem>.<ext>`), and
//! what moves files in them: a game's Clear, the one-time step into games' folders, the orphans and their cleanup into
//! `CLEANUP`, never a deletion of what the store did not write. `MediaStore`'s file half ported. See
//! EmuSen_BigPicture.md §30, §38 and EmuSen_Settings_Reference.md §4.59.

use std::path::Path;
use emusen_galaxia::path_key;
use crate::Error;
use crate::kinds;
use crate::store::{CLEANUP_FOLDER, FOLDERS_STEP, Key, Store};
use emusen_galaxia::files;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

fn io(e: std::io::Error, at: &str) -> Error {
    Error::Io(format!("{}: {}", at, e))
}

/// `MediaStore.Relative`: the system, the kind's folder, the game's folder and the file's name.
pub fn relative(system: &str, kind_folder: &str, folder: &str, file_name: &str) -> String {
    let folder = folder.replace('/', std::path::MAIN_SEPARATOR_STR);
    Path::new(system).join(kind_folder).join(&folder).join(file_name).to_string_lossy().into_owned()
}

fn parts(relative: &str) -> Vec<&str> {
    relative.split(['/', '\\']).filter(|p| !p.is_empty()).collect()
}

pub fn system_of(relative: &str) -> String {
    parts(relative).first().map(|s| s.to_string()).unwrap_or_default()
}

pub fn kind_folder_of(relative: &str) -> String {
    let p = parts(relative);
    match p.len() {
        n if n >= 3 => p[1].to_string(),
        2 => p[0].to_string(),
        _ => String::new(),
    }
}

/// The game's folder a stored file sits in, '/'-separated; "" for `<system>/<type>/<file>`.
pub fn folder_of(relative: &str) -> String {
    let p = parts(relative);
    if p.len() > 3 { p[2..p.len() - 1].join("/") } else { String::new() }
}


fn root_prefix(root: &str) -> String {
    let full = path_key::key_text(root);
    format!("{}{}", full.trim_end_matches(std::path::MAIN_SEPARATOR), std::path::MAIN_SEPARATOR)
}

/// ES-DE's Clear for one game: its answer and its pictures go, from `media.db` and this store's folders only.
pub fn forget(store: &Store, rom: &str, system: Option<&str>, folder: &str) -> Result<Vec<String>, Error> {
    let mut files: Vec<String> = store.forget_rows(rom)?.iter().map(|r| Path::new(&store.root).join(r).to_string_lossy().into_owned()).collect();
    if let Some(system) = system.filter(|s| !s.is_empty()) {
        let stem = Path::new(rom).file_stem().unwrap_or_default().to_string_lossy().to_string();
        for ty in kinds::ALL_FOLDER_ORDER.iter().map(|k| k.folder) {
            let dir = Path::new(&store.root).join(system).join(ty).join(&folder.replace('/', std::path::MAIN_SEPARATOR_STR)).to_string_lossy().into_owned();
            if !files::directory_exists(&dir) {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if files::listed_as_directory(&entry) || !name.starts_with(&format!("{stem}.")) {
                        continue;
                    }
                    if Path::new(&name).file_stem().unwrap_or_default().to_string_lossy() == stem {
                        files.push(Path::new(&dir).join(&name).to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    let root = root_prefix(&store.root);
    let mut seen = HashSet::new();
    let mut deleted = Vec::new();
    for file in files.iter().map(|f| path_key::key_text(f)) {
        if !seen.insert(file.clone()) || !file.starts_with(&root) || !files::file_exists(&file) {
            continue;
        }
        std::fs::remove_file(&file).map_err(|e| io(e, &file))?;
        deleted.push(file);
    }
    Ok(deleted)
}

/// Once: a picture stored flat for a game in a folder goes to that folder, moved when no game at the top shares it,
/// else copied; nothing is deleted and every step is logged. How many files were carried.
pub fn put_in_folders(store: &Store, folder_of_rom: &dyn Fn(&str) -> String, now: i64) -> Result<usize, Error> {
    if store.done(FOLDERS_STEP)? {
        return Ok(0);
    }
    let rows = store.all_media_rows()?;
    let mut roms: HashMap<Key, Vec<String>> = HashMap::new();
    for (p, key) in store.all_files()? {
        roms.entry(key).or_default().push(p);
    }
    struct Entry {
        rows: Vec<(Key, String, Option<String>)>,
        folders: BTreeSet<String>,
        top: bool,
    }
    let mut plan: BTreeMap<String, Entry> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for (key, ty, flat) in rows {
        if parts(&flat).len() != 3 {
            continue;
        }
        let stem = Path::new(&flat).file_stem().unwrap_or_default().to_string_lossy().to_string();
        let folders: Vec<String> = roms
            .get(&key)
            .map(|ps| ps.iter().filter(|p| Path::new(p).file_stem().unwrap_or_default().to_string_lossy() == stem).map(|p| folder_of_rom(p)).collect())
            .unwrap_or_default();
        if !plan.contains_key(&flat) {
            order.push(flat.clone());
        }
        let entry = plan.entry(flat.clone()).or_insert_with(|| Entry { rows: Vec::new(), folders: BTreeSet::new(), top: false });
        let own = folders.iter().filter(|f| !f.is_empty()).min().cloned();
        let top_here = folders.iter().any(|f| f.is_empty());
        entry.rows.push((key, ty, if top_here { None } else { own }));
        entry.folders.extend(folders.iter().filter(|f| !f.is_empty()).cloned());
        entry.top |= top_here;
    }
    let mut carried = 0;
    for flat in &order {
        let entry = &plan[flat];
        if entry.folders.is_empty() {
            continue;
        }
        let source = Path::new(&store.root).join(flat).to_string_lossy().into_owned();
        let (system, kind, name) = (system_of(flat), kind_folder_of(flat), Path::new(flat).file_name().unwrap_or_default().to_string_lossy().to_string());
        let targets: Vec<String> = entry.folders.iter().map(|f| relative(&system, &kind, f, &name)).collect();
        for (i, target) in targets.iter().enumerate() {
            let full = Path::new(&store.root).join(target).to_string_lossy().into_owned();
            let how = if files::file_exists(&full) {
                "found"
            } else if !files::file_exists(&source) {
                continue;
            } else {
                if let Some(dir) = Path::new(&full).parent().map(|d| d.to_string_lossy().into_owned()) {
                    std::fs::create_dir_all(&dir).map_err(|e| io(e, &dir))?;
                }
                let moving = i == targets.len() - 1 && !entry.top;
                if moving {
                    std::fs::rename(&source, &full).map_err(|e| io(e, &source))?;
                } else {
                    std::fs::copy(&source, &full).map_err(|e| io(e, &source))?;
                }
                carried += 1;
                if moving { "moved" } else { "copied" }
            };
            store.record_move(flat, target, how, now)?;
        }
        for (key, ty, folder) in &entry.rows {
            let Some(folder) = folder else { continue };
            let target = relative(&system, &kind, folder, &name);
            if files::file_exists(&Path::new(&store.root).join(&target).to_string_lossy().into_owned()) {
                store.repoint_media(key, ty, &target)?;
            }
        }
    }
    store.step_done(FOLDERS_STEP, now)?;
    Ok(carried)
}

/// Files of the store's type folders whose game is gone, under the systems named only, sorted.
pub fn orphans(store: &Store, names_by_system: &BTreeMap<String, HashSet<String>>) -> Vec<String> {
    let mut found = Vec::new();
    let mut folders: Vec<&str> = Vec::new();
    for f in kinds::ALL_FOLDER_ORDER.iter().map(|k| k.folder) {
        if !folders.contains(&f) {
            folders.push(f);
        }
    }
    for (system, names) in names_by_system {
        for folder in &folders {
            let dir = Path::new(&store.root).join(system).join(folder).to_string_lossy().into_owned();
            if !files::directory_exists(&dir) {
                continue;
            }
            let _ = files::visit_files(&dir, |file, _| {
                let file = &*file.to_string_lossy();
                let Some(rel) = Path::new(file).strip_prefix(&dir).ok() else { return };
                let name = rel.with_extension("").to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
                if file.ends_with(".part") || names.contains(&name) {
                    return;
                }
                found.push(file[store.root.trim_end_matches(std::path::MAIN_SEPARATOR).len()..].trim_start_matches(std::path::MAIN_SEPARATOR).to_string());
            });
        }
    }
    found.sort_by(|a, b| a.cmp(b));
    found
}

/// Each game's names in the store's terms, from the games a scan found: its stem below its folder, and each folder.
pub fn names_by_system(games: &[(String, String)], folder_of_rom: &dyn Fn(&str) -> String) -> BTreeMap<String, HashSet<String>> {
    let mut names: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    for (rom, system) in games {
        let set = names.entry(system.clone()).or_default();
        let stem = Path::new(rom).file_stem().unwrap_or_default().to_string_lossy();
        let folder = folder_of_rom(rom);
        set.insert(if folder.is_empty() { stem.to_string() } else { format!("{folder}/{stem}") });
        let mut f = folder.as_str();
        while !f.is_empty() {
            set.insert(f.to_string());
            f = f.rfind('/').map_or("", |i| &f[..i]);
        }
    }
    names
}

/// ES-DE's cleanup: each file moved, never deleted, to `CLEANUP/<local date_time>/` with its path kept, its row dropped,
/// emptied folders removed, and the moves listed in `cleanup.txt`. How many, their bytes, and the folder.
/// The folder is named by the machine's local time of the instant, the note by the time as it was given, with its offset.
pub fn clean_up(store: &Store, relatives: &[String], machine_local: i64, given_local: i64, offset_seconds: i64) -> Result<(usize, u64, String), Error> {
    let stamp = emusen_galaxia::clock::text(machine_local, emusen_galaxia::clock::STAMP_NAME);
    let backup = Path::new(&store.root).join(CLEANUP_FOLDER).join(&stamp).to_string_lossy().into_owned();
    let root = root_prefix(&store.root);
    let (mut moved, mut bytes) = (0usize, 0u64);
    let mut log = Vec::new();
    for rel in relatives {
        let source = path_key::key_text(&Path::new(&store.root).join(rel).to_string_lossy().into_owned());
        if !source.starts_with(&root) || rel.starts_with(CLEANUP_FOLDER) || !files::file_exists(&source) {
            continue;
        }
        let target = Path::new(&backup).join(rel).to_string_lossy().into_owned();
        if let Some(dir) = Path::new(&target).parent().map(|d| d.to_string_lossy().into_owned()) {
            std::fs::create_dir_all(&dir).map_err(|e| io(e, &dir))?;
        }
        let size = std::fs::metadata(&source).map(|m| m.len()).unwrap_or(0);
        if files::file_exists(&target) {
            return Err(Error::Io(format!("The file '{target}' already exists.")));
        }
        std::fs::rename(&source, &target).map_err(|e| io(e, &source))?;
        store.drop_media_at(rel)?;
        moved += 1;
        bytes += size;
        log.push(rel.clone());
        remove_empty(&Path::new(&source).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default(), &root);
    }
    if moved > 0 {
        let sign = if offset_seconds < 0 { '-' } else { '+' };
        let off = offset_seconds.abs() / 60;
        let when = format!("{} {sign}{:02}:{:02}", emusen_galaxia::clock::text(given_local, emusen_galaxia::clock::STAMP), off / 60, off % 60);
        let mut text = format!("Moved here from {} on {when}\n", store.root);
        for line in &log {
            text.push_str(line);
            text.push('\n');
        }
        let file = Path::new(&backup).join("cleanup.txt").to_string_lossy().into_owned();
        std::fs::write(&file, text).map_err(|e| io(e, &file))?;
    }
    Ok((moved, bytes, backup))
}

/// Up from a folder a file left, each folder removed while it is empty, stopping at a system's folder.
fn remove_empty(dir: &str, root: &str) {
    let mut dir = dir.to_string();
    while dir.len() > root.len() && dir.starts_with(root) && dir[root.len()..].split(std::path::MAIN_SEPARATOR).filter(|p| !p.is_empty()).count() > 1 {
        let empty = std::fs::read_dir(&dir).map(|mut e| e.next().is_none()).unwrap_or(false);
        if !empty || std::fs::remove_dir(&dir).is_err() {
            break;
        }
        dir = Path::new(&dir).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
    }
}

pub fn kinds_folder(esde_type: &str) -> Option<&'static str> {
    kinds::by_type(esde_type).map(|k| k.folder)
}
