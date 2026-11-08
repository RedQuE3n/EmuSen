//! Copies emulator-written data out of the places it used to live: the C# `DataMigration`. See EmuSen_Galaxia.md §3.2.
//!
//! The rule throughout is copy, never move, and never overwrite: the migration destroys nothing, and a destination
//! file that already exists always wins. The ROM library is named here only to be left alone (§3.3).

use crate::atomic::words;
use crate::files::{directory_exists, file_exists};
use crate::tree::{Tree, legacy_root_for};
use std::path::{Path, PathBuf};
use std::collections::VecDeque;
use std::io;

/// Only what the emulator wrote; the ROM library is excluded.
pub const MIGRATED_DIRECTORIES: [&str; 4] = ["Saves", "Logs", "Firmware", "Cheats"];
/// The old ROM folders, reported and never copied.
pub const LIBRARY_DIRECTORIES: [&str; 2] = ["Games", "Roms"];


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

/// A directory's entries as (name, is a directory), a link taken as what it links to.
fn entries(directory: &Path) -> io::Result<Vec<(String, bool)>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let is_directory = crate::files::listed_as_directory(&entry);
        found.push((entry.file_name().to_string_lossy().into_owned(), is_directory));
    }
    Ok(found)
}

/// One file copied with its permissions and its modification time, as `File.Copy` leaves it.
pub(crate) fn copy_file(source: &Path, target: &Path) -> io::Result<()> {
    if let Some(directory) = target.parent().filter(|d| !d.as_os_str().is_empty()) {
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
pub fn copy_tree(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<Copied> {
    let (source, destination) = (source.as_ref(), destination.as_ref());
    let mut copied = Copied::default();
    if !directory_exists(source) {
        return Ok(copied);
    }
    // Breadth first, the order .NET's enumeration reports in; it decides only the order of the diagnostics.
    let mut pending = VecDeque::from([PathBuf::new()]);
    while let Some(relative) = pending.pop_front() {
        for (name, is_directory) in entries(&source.join(&relative))? {
            let relative = relative.join(&name);
            if is_directory {
                pending.push_back(relative);
                continue;
            }
            let target = destination.join(&relative);
            if file_exists(&target) {
                continue;
            }
            let source_file = source.join(&relative);
            match copy_file(&source_file, &target) {
                Ok(()) => copied.files += 1,
                Err(error) => copied.diagnostics.push(format!("{}: {} Left in place.", source_file.display(), words(&error))),
            }
        }
    }
    Ok(copied)
}

/// `DataMigration.Run(legacyRoot, destinationRoot)`: the four emulator-written directories, unless the two roots are one folder.
pub fn run(legacy_root: &Path, destination_root: &Path) -> io::Result<Copied> {
    if !directory_exists(legacy_root) || crate::path_key::key(legacy_root) == crate::path_key::key(destination_root) {
        return Ok(Copied::default());
    }
    let mut copied = Copied::default();
    for name in MIGRATED_DIRECTORIES {
        copied = copied.and(copy_tree(legacy_root.join(name), destination_root.join(name))?);
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
pub fn seed_from_bundle(seed_directory: Option<&Path>, home: &Path) -> io::Result<Copied> {
    match seed_directory {
        Some(seed) => copy_tree(seed, home),
        None => Ok(Copied::default()),
    }
}

fn holds_a_file(directory: &Path) -> io::Result<bool> {
    let mut pending = VecDeque::from([directory.to_path_buf()]);
    while let Some(directory) = pending.pop_front() {
        for (name, is_directory) in entries(&directory)? {
            if !is_directory {
                return Ok(true);
            }
            pending.push_back(directory.join(&name));
        }
    }
    Ok(false)
}

/// The old ROM folders that still hold files, so a caller can say where they are; and what could not be looked into.
pub fn remaining_library_directories(legacy_root: &Path) -> (Vec<PathBuf>, Vec<String>) {
    let (mut found, mut diagnostics) = (Vec::new(), Vec::new());
    for name in LIBRARY_DIRECTORIES {
        let directory = legacy_root.join(name);
        if !directory_exists(&directory) {
            continue;
        }
        match holds_a_file(&directory) {
            Ok(true) => found.push(directory),
            Ok(false) => {}
            Err(error) => diagnostics.push(format!("{}: {}", directory.display(), words(&error))),
        }
    }
    (found, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn put(temp: &TempDir, relative: &str, contents: &str) -> PathBuf {
        let file = temp.join(relative);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
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

    /// Every entry under a directory, with a file's bytes and every entry's modification time, in order.
    fn snapshot(directory: &Path) -> Vec<(PathBuf, Option<Vec<u8>>, std::time::SystemTime)> {
        let mut found = Vec::new();
        let mut pending = vec![directory.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for (name, is_directory) in entries(&directory).unwrap() {
                let entry = directory.join(&name);
                let modified = std::fs::metadata(&entry).unwrap().modified().unwrap();
                found.push((entry.clone(), (!is_directory).then(|| std::fs::read(&entry).unwrap()), modified));
                if is_directory {
                    pending.push(entry);
                }
            }
        }
        found.sort();
        found
    }

    /// `GetShortPathNameW`: the 8.3 spelling, where the volume keeps short names.
    #[cfg(windows)]
    fn short_name(long: &str) -> Option<String> {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetShortPathNameW(long: *const u16, short: *mut u16, length: u32) -> u32;
        }
        let wide: Vec<u16> = std::ffi::OsStr::new(long).encode_wide().chain([0]).collect();
        let mut short = vec![0u16; 1024];
        // SAFETY: both buffers live for the call, the first ends in a NUL and the second's length is given.
        let length = unsafe { GetShortPathNameW(wide.as_ptr(), short.as_mut_ptr(), short.len() as u32) } as usize;
        (length > 0 && length < short.len()).then(|| std::ffi::OsString::from_wide(&short[..length]).to_string_lossy().into_owned())
    }

    /// The tree's own spellings on this system, links to it made in `links`: each names the same folder, the case variants only where the file system ignores case.
    fn spellings(temp: &TempDir, home: &Path, links: &TempDir) -> Vec<PathBuf> {
        let mut spellings = vec![home.to_path_buf(), home.join(""), temp.join("x/../Caf\u{e9}")];
        if cfg!(any(windows, target_os = "macos")) {
            spellings.extend([temp.join("CAF\u{c9}"), temp.join("caf\u{e9}")]);
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(home, links.join("linked")).unwrap();
            spellings.push(links.join("linked"));
        }
        // The decomposed é, which APFS reads as the composed one.
        #[cfg(target_os = "macos")]
        spellings.push(temp.join("Cafe\u{301}"));
        #[cfg(windows)]
        {
            let text = home.to_string_lossy().into_owned();
            let lower_drive = text[..1].to_lowercase() + &text[1..];
            spellings.extend([text.replace('\\', "/"), format!(r"\\?\{text}"), lower_drive, format!("{text}."), format!("{text} "), format!("{text}. .")].map(PathBuf::from));
            spellings.extend(short_name(&text).filter(|short| *short != text).map(PathBuf::from));
            let junction = links.join("junction");
            let made = std::process::Command::new("cmd").arg("/C").arg("mklink").arg("/J").arg(&junction).arg(home).output().unwrap();
            assert!(made.status.success(), "{made:?}");
            spellings.push(junction);
            if std::os::windows::fs::symlink_dir(home, links.join("linked")).is_ok() {
                spellings.push(links.join("linked"));
            }
        }
        spellings
    }

    /// A migration onto the tree itself, however the system lets it be spelled, copies nothing and changes nothing
    /// (EmuSen_Galaxia.md §3.2): the roots compare equal, or every file finds itself already there.
    #[test]
    fn a_tree_spelled_any_way_the_system_reads_as_itself_is_left_alone() {
        let (temp, links) = (TempDir::new("migrate-spelling"), TempDir::new("migrate-links"));
        for (file, contents) in [("Saves/a.srm", "save"), ("Saves/Save States/a.state", "state"), ("Logs/x/run.txt", "log"), ("Firmware/dsp1.rom", "fw"), ("Cheats/SNES/a.cht", "cheat")] {
            put(&temp, &format!("Caf\u{e9}/{file}"), contents);
        }
        let home = temp.join("Caf\u{e9}");
        let spellings = spellings(&temp, &home, &links);
        if cfg!(any(windows, target_os = "macos")) {
            assert!(directory_exists(&temp.join("CAF\u{c9}")), "this file system ignores case");
        }
        if cfg!(target_os = "macos") {
            assert!(directory_exists(&temp.join("Cafe\u{301}")), "this file system ignores the form of a character");
        }
        let (tree, beside) = (snapshot(&home), snapshot(temp.path()));
        for spelling in &spellings {
            assert_eq!(run(&home, spelling).unwrap(), Copied::default(), "onto {}", spelling.display());
            assert_eq!(run(spelling, &home).unwrap(), Copied::default(), "from {}", spelling.display());
        }
        assert_eq!(snapshot(&home), tree, "the tree is as it was");
        assert_eq!(snapshot(temp.path()), beside, "nothing was made beside it");
        assert_eq!(run(&temp.join("absent"), &home).unwrap(), Copied::default());
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
        assert!(copied.diagnostics[0].starts_with(&temp.text("old/broken")), "{:?}", copied.diagnostics);
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
