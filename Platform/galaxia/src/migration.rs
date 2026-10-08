//! Copies emulator-written data out of the places it used to live: the C# `DataMigration`. See EmuSen_Galaxia.md §3.2.
//!
//! The rule throughout is copy, never move, and never overwrite: the migration destroys nothing, and a destination
//! file that already exists always wins. The ROM library is named here only to be left alone (§3.3).

use crate::atomic::words;
use crate::dotnet_path::{self as path, Style};
use crate::tree::{Tree, directory_exists, file_exists, legacy_root_for};
use std::collections::VecDeque;
use std::io;

/// Only what the emulator wrote; the ROM library is excluded.
pub const MIGRATED_DIRECTORIES: [&str; 4] = ["Saves", "Logs", "Firmware", "Cheats"];
/// The old ROM folders, reported and never copied.
pub const LIBRARY_DIRECTORIES: [&str; 2] = ["Games", "Roms"];

const STYLE: Style = Style::HOST;

/// How many files a copy made, and what it said about the ones it could not.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Copied {
    pub files: usize,
    pub diagnostics: Vec<String>,
}

impl Copied {
    fn and(mut self, other: Copied) -> Copied {
        self.files += other.files;
        self.diagnostics.extend(other.diagnostics);
        self
    }
}

/// A directory's entries as (name, is a directory); a link is what it names, and a dangling one is a file, as .NET's enumeration has them.
fn entries(directory: &str) -> io::Result<Vec<(String, bool)>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let is_directory = std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir());
        found.push((entry.file_name().to_string_lossy().into_owned(), is_directory));
    }
    Ok(found)
}

/// One file copied with its permissions and its modification time, as `File.Copy` leaves it.
fn copy_file(source: &str, target: &str) -> io::Result<()> {
    if let Some(directory) = path::directory_name(STYLE, target).filter(|d| !d.is_empty()) {
        std::fs::create_dir_all(directory)?;
    }
    let metadata = std::fs::metadata(source)?;
    std::fs::copy(source, target)?;
    let times = std::fs::FileTimes::new().set_modified(metadata.modified()?);
    let times = match metadata.accessed() {
        Ok(accessed) => times.set_accessed(accessed),
        Err(_) => times,
    };
    // A filesystem that will not take the times still has the file, which is what .NET settles for too.
    let _ = std::fs::File::options().write(true).open(target).and_then(|file| file.set_times(times));
    Ok(())
}

/// Every file under `source` that `destination` lacks, copied to the same place under it. An unreadable directory is an error, as it is an exception in the C#.
pub fn copy_tree(source: &str, destination: &str) -> io::Result<Copied> {
    let mut copied = Copied::default();
    if !directory_exists(source) {
        return Ok(copied);
    }
    // Breadth first, the order .NET's enumeration reports in; it decides only the order of the diagnostics.
    let mut pending = VecDeque::from([String::new()]);
    while let Some(relative) = pending.pop_front() {
        for (name, is_directory) in entries(&path::combine(STYLE, source, &relative))? {
            let relative = path::combine(STYLE, &relative, &name);
            if is_directory {
                pending.push_back(relative);
                continue;
            }
            let target = path::combine(STYLE, destination, &relative);
            if file_exists(&target) {
                continue;
            }
            let source_file = path::combine(STYLE, source, &relative);
            match copy_file(&source_file, &target) {
                Ok(()) => copied.files += 1,
                Err(error) => copied.diagnostics.push(format!("{source_file}: {} Left in place.", words(&error))),
            }
        }
    }
    Ok(copied)
}

/// `DataMigration.Run(legacyRoot, destinationRoot)`: the four emulator-written directories, unless the two roots are one.
pub fn run(legacy_root: &str, destination_root: &str) -> io::Result<Copied> {
    if !directory_exists(legacy_root) || path::equals_ignore_case(&path::full_path(legacy_root), &path::full_path(destination_root)) {
        return Ok(Copied::default());
    }
    let mut copied = Copied::default();
    for name in MIGRATED_DIRECTORIES {
        copied = copied.and(copy_tree(&path::combine(STYLE, legacy_root, name), &path::combine(STYLE, destination_root, name))?);
    }
    Ok(copied)
}

/// `DataMigration.RunConfig`: `etc/EmuSen` moved under home when the shell was rooted there.
pub fn run_config(tree: &Tree) -> io::Result<Copied> {
    copy_tree(&tree.config_previous(), &tree.config())
}

/// `DataMigration.Run()`: this program's own old tree, then its old config.
pub fn run_own(tree: &Tree) -> io::Result<Copied> {
    Ok(run(&legacy_root_for(&tree.root), &tree.home())?.and(run_config(tree)?))
}

/// `DataMigration.SeedFromBundle`: a Mac bundle's read-only skeleton, copied into home and never over the player's files.
pub fn seed_from_bundle(seed_directory: Option<&str>, home: &str) -> io::Result<Copied> {
    match seed_directory {
        Some(seed) => copy_tree(seed, home),
        None => Ok(Copied::default()),
    }
}

fn holds_a_file(directory: &str) -> io::Result<bool> {
    let mut pending = VecDeque::from([directory.to_string()]);
    while let Some(directory) = pending.pop_front() {
        for (name, is_directory) in entries(&directory)? {
            if !is_directory {
                return Ok(true);
            }
            pending.push_back(path::combine(STYLE, &directory, &name));
        }
    }
    Ok(false)
}

/// The old ROM folders that still hold files, so a caller can say where they are; and what could not be looked into.
pub fn remaining_library_directories(legacy_root: &str) -> (Vec<String>, Vec<String>) {
    let (mut found, mut diagnostics) = (Vec::new(), Vec::new());
    for name in LIBRARY_DIRECTORIES {
        let directory = path::combine(STYLE, legacy_root, name);
        if !directory_exists(&directory) {
            continue;
        }
        match holds_a_file(&directory) {
            Ok(true) => found.push(directory),
            Ok(false) => {}
            Err(error) => diagnostics.push(format!("{directory}: {}", words(&error))),
        }
    }
    (found, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn put(temp: &TempDir, relative: &str, contents: &str) -> String {
        let file = temp.join(relative);
        std::fs::create_dir_all(path::directory_name(STYLE, &file).unwrap()).unwrap();
        std::fs::write(&file, contents).unwrap();
        file
    }

    fn read(temp: &TempDir, relative: &str) -> String {
        std::fs::read_to_string(temp.join(relative)).unwrap()
    }

    #[test]
    fn emulator_written_data_is_copied_and_the_rom_library_is_not() {
        let temp = TempDir::new("migrate");
        put(&temp, "old/Saves/ALTTP.srm", "save");
        put(&temp, "old/Saves/Save States/ALTTP.state", "state");
        put(&temp, "old/Firmware/dsp1.rom", "fw");
        put(&temp, "old/Cheats/SNES/x.cht", "cheat");
        put(&temp, "old/Logs/Venus/run.txt", "log");
        let game = put(&temp, "old/Games/SNES/ALTTP.smc", "rom");
        put(&temp, "old/Roms/SMW.smc", "rom");
        let before = std::fs::metadata(&game).unwrap().modified().unwrap();

        let copied = run(&temp.join("old"), &temp.join("home")).unwrap();

        assert_eq!(copied, Copied { files: 5, diagnostics: vec![] });
        assert_eq!(read(&temp, "home/Saves/Save States/ALTTP.state"), "state");
        assert_eq!(read(&temp, "home/Cheats/SNES/x.cht"), "cheat");
        assert!(!directory_exists(&temp.join("home/Games")));
        assert!(!directory_exists(&temp.join("home/Roms")));
        assert_eq!(read(&temp, "old/Saves/ALTTP.srm"), "save");
        assert_eq!(read(&temp, "old/Games/SNES/ALTTP.smc"), "rom");
        assert_eq!(std::fs::metadata(&game).unwrap().modified().unwrap(), before);
        assert_eq!(remaining_library_directories(&temp.join("old")), (vec![temp.join("old/Games"), temp.join("old/Roms")], vec![]));
    }

    #[test]
    fn an_existing_destination_file_always_wins_and_a_second_run_copies_nothing() {
        let temp = TempDir::new("migrate");
        put(&temp, "old/Saves/a.srm", "old");
        put(&temp, "old/Saves/b.srm", "b");
        put(&temp, "home/Saves/a.srm", "newer");
        assert_eq!(run(&temp.join("old"), &temp.join("home")).unwrap().files, 1);
        assert_eq!(read(&temp, "home/Saves/a.srm"), "newer");
        assert_eq!(run(&temp.join("old"), &temp.join("home")).unwrap().files, 0);
    }

    #[test]
    fn a_tree_is_not_migrated_onto_itself_whatever_the_spelling() {
        let temp = TempDir::new("migrate");
        put(&temp, "home/Saves/a.srm", "save");
        let home = temp.join("home");
        assert_eq!(run(&home, &home).unwrap().files, 0);
        assert_eq!(run(&home, &temp.join("x/../home")).unwrap().files, 0);
        // The C# compares the two roots without regard to case on every platform, so a root spelled in another case is the same root.
        assert_eq!(run(&home, &temp.join("HOME")).unwrap().files, 0);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1, "nothing was made beside the tree");
        assert_eq!(run(&temp.join("absent"), &home).unwrap().files, 0);
    }

    #[test]
    fn a_copy_keeps_the_files_modification_time() {
        let temp = TempDir::new("migrate");
        let source = put(&temp, "old/a.txt", "a");
        let then = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_577_934_245);
        std::fs::File::options().write(true).open(&source).unwrap().set_modified(then).unwrap();
        assert_eq!(copy_tree(&temp.join("old"), &temp.join("new")).unwrap().files, 1);
        assert_eq!(std::fs::metadata(temp.join("new/a.txt")).unwrap().modified().unwrap(), then);
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_directory_is_walked_and_a_dangling_link_is_left_with_a_word() {
        let temp = TempDir::new("migrate");
        put(&temp, "elsewhere/inner/c.txt", "c");
        put(&temp, "old/a.txt", "a");
        std::os::unix::fs::symlink(temp.join("elsewhere"), temp.join("old/linked")).unwrap();
        std::os::unix::fs::symlink("nowhere", temp.join("old/broken")).unwrap();

        let copied = copy_tree(&temp.join("old"), &temp.join("new")).unwrap();

        assert_eq!(copied.files, 2);
        assert_eq!(read(&temp, "new/linked/inner/c.txt"), "c");
        assert_eq!(copied.diagnostics.len(), 1);
        assert!(copied.diagnostics[0].starts_with(&temp.join("old/broken")), "{:?}", copied.diagnostics);
        assert!(copied.diagnostics[0].ends_with(" Left in place."), "{:?}", copied.diagnostics);
    }

    #[test]
    fn an_empty_library_folder_is_not_reported() {
        let temp = TempDir::new("migrate");
        std::fs::create_dir_all(temp.join("old/Games/SNES/empty")).unwrap();
        put(&temp, "old/Roms/deep/er/game.sfc", "rom");
        assert_eq!(remaining_library_directories(&temp.join("old")), (vec![temp.join("old/Roms")], vec![]));
    }

    #[test]
    fn the_seed_is_copied_only_where_there_is_one() {
        let temp = TempDir::new("migrate");
        put(&temp, "seed/Documents/manual.md", "page");
        put(&temp, "home/Documents/kept.md", "mine");
        assert_eq!(seed_from_bundle(None, &temp.join("home")).unwrap().files, 0);
        assert_eq!(seed_from_bundle(Some(&temp.join("seed")), &temp.join("home")).unwrap().files, 1);
        assert_eq!(read(&temp, "home/Documents/manual.md"), "page");
        assert_eq!(read(&temp, "home/Documents/kept.md"), "mine");
    }
}
