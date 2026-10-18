//! The player's cheat database, the C# `CheatDatabase`: an index over a tree of `.cht` files, each in a folder named
//! for its system as libretro lays them out, scanned once and searched by game (`man cheat`). Nothing EmuSen ships.

use emusen_dotnet::text::{compare_ignore_case, contains_ignore_case, ends_with_ignore_case, equals_ignore_case, is_blank, starts_with_ignore_case, trim};
use emusen_galaxia::dotnet_path::{self as path, Style};
use std::cmp::Ordering;
use std::collections::VecDeque;

/// One `.cht` file in the tree: its folder's name, its name without `.cht`, and its path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub system: String,
    pub game: String,
    pub path: String,
}

/// `Path.Join` of a folder and a name in it, as .NET's enumeration builds each path it gives.
pub(crate) fn join(folder: &str, name: &str) -> String {
    if folder.is_empty() || path::ends_in_separator(Style::HOST, folder) { format!("{folder}{name}") } else { format!("{folder}{}{name}", Style::HOST.separator()) }
}

/// Whether a listed entry is a directory as .NET's enumeration takes it: a link is taken as what it links to.
pub(crate) fn listed_as_directory(entry: &std::fs::DirEntry) -> bool {
    std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir())
}

/// Every file under a folder, handed over as `Directory.EnumerateFiles(folder, pattern, AllDirectories)` yields them: a
/// folder's files in the system's order, its folders queued and visited after, links to folders followed. An error
/// ends the walk, after what was handed over before it, as it ends .NET's lazy enumeration.
pub(crate) fn visit_files(folder: &str, mut each: impl FnMut(&str, &str)) -> std::io::Result<()> {
    let mut queue = VecDeque::from([folder.to_string()]);
    while let Some(dir) = queue.pop_front() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let at = join(&dir, &name);
            if listed_as_directory(&entry) {
                queue.push_back(at);
            } else {
                each(&at, &name);
            }
        }
    }
    Ok(())
}

/// Whether a name ends as `*.cht` asks, by case on Linux, as .NET's enumeration matches there.
fn is_cht(name: &str) -> bool {
    if cfg!(any(windows, target_os = "macos")) { ends_with_ignore_case(name, ".cht") } else { name.ends_with(".cht") }
}

/// The index, scanned once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Database {
    pub directory: String,
    pub entries: Vec<Entry>,
}

fn by_system_then_game(a: &Entry, b: &Entry) -> Ordering {
    compare_ignore_case(&a.system, &b.system).then_with(|| compare_ignore_case(&a.game, &b.game))
}

impl Database {
    /// `CheatDatabase.All` on first use: nothing where the folder is absent or the tree cannot be listed whole.
    pub fn scan(directory: &str) -> Database {
        let mut db = Database { directory: directory.to_string(), entries: Vec::new() };
        if !emusen_galaxia::tree::directory_exists(directory) {
            return db;
        }
        let mut files = Vec::new();
        if visit_files(directory, |at, name| if is_cht(name) { files.push(at.to_string()) }).is_err() {
            return db;
        }
        db.entries = files
            .into_iter()
            .map(|p| Entry {
                system: path::directory_name(Style::HOST, &p).map(|d| path::file_name(Style::HOST, &d).to_string()).unwrap_or_default(),
                game: path::file_name_without_extension(Style::HOST, &p).to_string(),
                path: p,
            })
            .collect();
        db.entries.sort_by(by_system_then_game);
        db
    }

    /// `CheatDatabase.Systems`: each system once, spelt as its first file's folder is, with its count.
    pub fn systems(&self) -> Vec<(String, usize)> {
        let mut groups: Vec<(String, usize)> = Vec::new();
        for e in &self.entries {
            match groups.iter_mut().find(|(s, _)| equals_ignore_case(s, &e.system)) {
                Some(group) => group.1 += 1,
                None => groups.push((e.system.clone(), 1)),
            }
        }
        groups.sort_by(|a, b| compare_ignore_case(&a.0, &b.0));
        groups
    }

    /// `CheatDatabase.Games`: a system's files, those whose game holds the filter where one is given, by their places.
    pub fn games(&self, system: &str, filter: Option<&str>) -> Vec<usize> {
        let needle = filter.filter(|f| !is_blank(Some(f))).map(trim);
        (0..self.entries.len()).filter(|&i| equals_ignore_case(&self.entries[i].system, system) && needle.is_none_or(|n| contains_ignore_case(&self.entries[i].game, n))).collect()
    }

    /// `CheatDatabase.Find`: games named as the text, then starting with it, then holding it, a ROM's extension dropped
    /// from both, by their places.
    pub fn find(&self, text: &str, limit: i32) -> Vec<usize> {
        let needle = normalize(text);
        if needle.is_empty() || limit <= 0 {
            return Vec::new();
        }
        let mut scored: Vec<(usize, u8)> = (0..self.entries.len()).map(|i| (i, score(&normalize(&self.entries[i].game), &needle))).filter(|(_, s)| *s > 0).collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| compare_ignore_case(&self.entries[a.0].game, &self.entries[b.0].game)));
        scored.into_iter().take(limit as usize).map(|(i, _)| i).collect()
    }
}

fn score(game: &str, needle: &str) -> u8 {
    if equals_ignore_case(game, needle) {
        3
    } else if starts_with_ignore_case(game, needle) {
        2
    } else if contains_ignore_case(game, needle) {
        1
    } else {
        0
    }
}

/// `Normalize`: trimmed, and the first of `.cht`, `.sfc`, `.smc` and `.zip` it ends with, without regard to case, dropped.
fn normalize(text: &str) -> String {
    let t = trim(text);
    for extension in [".cht", ".sfc", ".smc", ".zip"] {
        if ends_with_ignore_case(t, extension) {
            let keep = t.chars().count() - extension.chars().count();
            return t.chars().take(keep).collect();
        }
    }
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_is_indexed_by_folder_and_game_and_searched_by_rank() {
        let root = std::env::temp_dir().join(format!("emusen-cheats-db-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (dir, file) in [("Nintendo - SNES", "Super Mario World (USA).cht"), ("Nintendo - SNES", "Mario Paint.cht"), ("Sega - Mega Drive", "Sonic.cht"), ("Sega - Mega Drive", "notes.txt")] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
            std::fs::write(root.join(dir).join(file), b"cheats = 0\n").unwrap();
        }
        let db = Database::scan(&root.to_string_lossy());
        assert_eq!(db.entries.len(), 3);
        assert_eq!(db.systems(), [("Nintendo - SNES".to_string(), 2), ("Sega - Mega Drive".to_string(), 1)]);
        assert_eq!(db.games("nintendo - snes", Some(" paint ")).len(), 1);
        let found: Vec<&str> = db.find("mario paint.sfc", 20).iter().map(|&i| db.entries[i].game.as_str()).collect();
        assert_eq!(found, ["Mario Paint"]);
        let found: Vec<&str> = db.find("mario", 20).iter().map(|&i| db.entries[i].game.as_str()).collect();
        assert_eq!(found, ["Mario Paint", "Super Mario World (USA)"]);
        std::fs::remove_dir_all(&root).unwrap();
        assert!(Database::scan(&root.to_string_lossy()).entries.is_empty());
    }
}
