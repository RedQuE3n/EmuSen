//! Installing a downloaded cheat archive: every `.cht` entry of a zip written under the target folder where its name
//! puts it, with the entry's own time where the zip records one; anything else, or a name that would leave the folder,
//! counted as skipped. The archive is read by Galaxia's `archive`, the platform's one zip reader, and one it refuses is
//! refused in the zip crate's words. The download is not here (EmuSen_Backend.md SR9). See EmuSen_Backend.md §15.3 and
//! EmuSen_RustPlatform.md §47.15.

use emusen_galaxia::archive::{Zip, ZipEntry};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Where the libretro cheat database is downloaded from: `CheatDatabaseInstaller.LibretroCheatsUrl`, what RetroArch's
/// own "Update Cheats" pulls. The platform header gives it as `EMUSEN_CHEATS_DATABASE_URL`.
pub const DATABASE_URL: &str = "https://buildbot.libretro.com/assets/frontend/cheats.zip";

/// What an install did: entries written, and entries passed over or that failed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Installed {
    pub installed: usize,
    pub skipped: usize,
}

/// Why nothing was installed: the archive is no zip the reader takes, the target was not made, or a folder beside it
/// has the name an install uses but no install made it (its words name the folder).
#[derive(Debug)]
pub enum Refusal {
    Unreadable(String),
    Io(std::io::Error),
    NotOurs(String),
}

/// One `.cht` entry written whole at `destination`, its folders made, its time the zip's where it records one.
fn extract(zip: &mut Zip<Cursor<&[u8]>>, e: &ZipEntry, destination: &Path) -> Result<(), String> {
    let mut bytes = Vec::new();
    zip.copy(e, &mut bytes)?;
    if let Some(folder) = destination.parent() {
        std::fs::create_dir_all(folder).map_err(|x| x.to_string())?;
    }
    std::fs::write(destination, &bytes).map_err(|x| x.to_string())?;
    if let Some(t) = zip.modified(e) {
        let _ = std::fs::File::options().write(true).open(destination).and_then(|f| f.set_modified(t));
    }
    Ok(())
}

/// The archive's `.cht` entries installed under `target`, which is made first.
pub fn install(bytes: &[u8], target: &str) -> Result<Installed, Refusal> {
    std::fs::create_dir_all(target).map_err(Refusal::Io)?;
    let mut zip = Zip::over(Cursor::new(bytes)).map_err(Refusal::Unreadable)?;
    let mut done = Installed::default();
    for e in zip.entries().map_err(Refusal::Unreadable)? {
        if e.full_name.ends_with('/') {
            continue;
        }
        match &e.enclosed {
            Some(inside) if e.full_name.to_ascii_lowercase().ends_with(".cht") && extract(&mut zip, &e, &Path::new(target).join(inside)).is_ok() => done.installed += 1,
            _ => done.skipped += 1,
        }
    }
    Ok(done)
}

/// How long the base's own download of the database may take: `CheatDatabaseInstaller.FetchTimeout`, five minutes.
pub const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// The file an install writes into each folder beside the database it makes, and requires there before it removes or
/// puts back such a folder; a constant of the design (EmuSen_RustPlatform.md §56).
pub const MARK: &str = ".emusen-cheats-install";

/// A folder beside `target`, `<target>.part` for an install being made or `<target>.old` for the database it replaces
/// while the two are swapped, named as Galaxia names a file kept beside another.
fn beside(target: &Path, suffix: &str) -> PathBuf {
    emusen_galaxia::atomic::beside(target, suffix)
}

/// Whether an install made `folder`: its mark is there.
fn ours(folder: &Path) -> bool {
    folder.join(MARK).is_file()
}

/// The refusal for a folder an install did not make, which is left exactly as it is.
fn not_ours(folder: &Path) -> Refusal {
    Refusal::NotOurs(format!(
        "{} has the name an install of the cheat database uses, but no install made it, so it is left as it is and nothing was installed; move or rename it to install",
        folder.display()
    ))
}

/// A folder this install made, removed with all it holds; one that is not there is no error.
fn clear(folder: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(folder) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// What an earlier install left beside the database, put right before this one starts: a `.part` it made is removed; an
/// `.old` it made goes back to being the database where a swap stopped between its renames, and is removed otherwise.
/// A folder of either name with no mark is refused, untouched.
fn recover(target: &Path, part: &Path, old: &Path) -> Result<(), Refusal> {
    for folder in [part, old] {
        if folder.symlink_metadata().is_ok() && !ours(folder) {
            return Err(not_ours(folder));
        }
    }
    clear(part).map_err(Refusal::Io)?;
    if !old.exists() {
        return Ok(());
    }
    if target.symlink_metadata().is_ok() {
        return clear(old).map_err(Refusal::Io);
    }
    std::fs::rename(old, target).and_then(|_| std::fs::remove_file(target.join(MARK))).map_err(Refusal::Io)
}

/// Each file under `from` that `to` lacks, linked (or copied where a link cannot be made) to its place under `to`, so
/// the new folder keeps every file of the database the archive does not replace.
fn keep(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let place = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            keep(&entry.path(), &place)?;
        } else if !place.exists() {
            std::fs::hard_link(entry.path(), &place).or_else(|_| std::fs::copy(entry.path(), &place).map(drop))?;
        }
    }
    Ok(())
}

/// `part` made the database: its mark taken out, the old database renamed to `old` and marked as the install's, `part`
/// renamed to `target`, `old` removed. Where the second rename fails the old one is renamed back and unmarked.
fn swap(target: &Path, part: &Path, old: &Path) -> std::io::Result<()> {
    std::fs::remove_file(part.join(MARK))?;
    if !target.exists() {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        return std::fs::rename(part, target);
    }
    std::fs::rename(target, old)?;
    let moved = std::fs::write(old.join(MARK), b"").and_then(|_| std::fs::rename(part, target));
    if let Err(e) = moved {
        let _ = std::fs::rename(old, target);
        let _ = std::fs::remove_file(target.join(MARK));
        return Err(e);
    }
    let _ = clear(old);
    Ok(())
}

/// [`install`] made whole before the database is touched: the archive written into `<target>.part`, every file of the
/// database it does not replace kept beside it, and the two folders then swapped by rename. Each folder beside the
/// database the install makes carries [`MARK`], and one without it is refused and left as it is. Anything that fails
/// before the swap leaves `target` as it was; see EmuSen_RustPlatform.md §56.
pub fn install_staged(bytes: &[u8], target: &str) -> Result<Installed, Refusal> {
    let target = Path::new(target);
    if target.file_name().is_none() {
        return Err(Refusal::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the folder has no name")));
    }
    let (part, old) = (beside(target, ".part"), beside(target, ".old"));
    recover(target, &part, &old)?;
    if let Some(parent) = part.parent() {
        std::fs::create_dir_all(parent).map_err(Refusal::Io)?;
    }
    std::fs::create_dir(&part).and_then(|_| std::fs::write(part.join(MARK), b"")).map_err(Refusal::Io)?;
    let made = install(bytes, &part.to_string_lossy()).and_then(|done| {
        if target.exists() {
            keep(target, &part).map_err(Refusal::Io)?;
        }
        swap(target, &part, &old).map_err(Refusal::Io)?;
        Ok(done)
    });
    let _ = clear(&part);
    made
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_galaxia::archive::ZipWriter;

    fn scratch(name: &str) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!("emusen-install-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    fn archive(at: &Path, files: &[(&str, &str)]) -> Vec<u8> {
        let zip = at.join("made.zip");
        let mut w = ZipWriter::create(zip.to_str().unwrap()).unwrap();
        for (at_n, (name, text)) in files.iter().enumerate() {
            let source = at.join(format!("source{at_n}"));
            std::fs::write(&source, text).unwrap();
            w.add(name, source.to_str().unwrap(), std::time::SystemTime::now(), false, &|| false).unwrap();
        }
        w.finish().unwrap();
        std::fs::read(&zip).unwrap()
    }

    #[test]
    fn a_staged_install_moves_every_file_into_the_database_and_leaves_no_part() {
        let at = scratch("staged");
        let target = at.join("db");
        std::fs::create_dir_all(target.join("Sega")).unwrap();
        std::fs::write(target.join("Sega/Kept.cht"), "kept").unwrap();
        std::fs::write(target.join("Sega/Old.cht"), "old").unwrap();
        let zip = archive(&at, &[("Nintendo - SNES/Game.cht", "new game"), ("Sega/Old.cht", "new old"), ("readme.txt", "x")]);
        let done = install_staged(&zip, target.to_str().unwrap()).unwrap();
        assert_eq!(done, Installed { installed: 2, skipped: 1 });
        assert_eq!(std::fs::read_to_string(target.join("Nintendo - SNES/Game.cht")).unwrap(), "new game");
        assert_eq!(std::fs::read_to_string(target.join("Sega/Old.cht")).unwrap(), "new old", "a file already there is replaced");
        assert_eq!(std::fs::read_to_string(target.join("Sega/Kept.cht")).unwrap(), "kept", "a file the archive lacks is kept");
        assert!(!at.join("db.part").exists() && !at.join("db.old").exists(), "the staging and old folders are gone");
        std::fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn a_refused_install_leaves_the_database_as_it_was() {
        let at = scratch("refused-staged");
        let target = at.join("db");
        std::fs::create_dir_all(target.join("Sega")).unwrap();
        std::fs::write(target.join("Sega/Old.cht"), "old").unwrap();
        let mut zip = archive(&at, &[("Sega/Old.cht", "new old"), ("Sega/New.cht", "new")]);
        zip.truncate(zip.len() / 2);
        assert!(matches!(install_staged(&zip, target.to_str().unwrap()), Err(Refusal::Unreadable(_))));
        assert_eq!(std::fs::read_to_string(target.join("Sega/Old.cht")).unwrap(), "old");
        assert!(!target.join("Sega/New.cht").exists() && !at.join("db.part").exists());
        std::fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn what_a_stopped_install_left_is_cleared_and_never_reaches_the_database() {
        let at = scratch("stopped");
        let target = at.join("db");
        std::fs::create_dir_all(target.join("Sega")).unwrap();
        std::fs::write(target.join("Sega/Old.cht"), "old").unwrap();
        std::fs::create_dir_all(at.join("db.part/Stale")).unwrap();
        std::fs::write(at.join("db.part").join(MARK), "").unwrap();
        std::fs::write(at.join("db.part/Stale/Half.cht"), "half").unwrap();
        let zip = archive(&at, &[("Sega/New.cht", "new")]);
        assert_eq!(install_staged(&zip, target.to_str().unwrap()).unwrap(), Installed { installed: 1, skipped: 0 });
        assert!(!target.join("Stale").exists(), "the stopped install's file is not moved in");
        assert_eq!(std::fs::read_to_string(target.join("Sega/Old.cht")).unwrap(), "old");
        assert!(target.join("Sega/New.cht").exists() && !at.join("db.part").exists());
        std::fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn an_install_that_fails_partway_leaves_the_database_exactly_as_it_was() {
        use std::os::unix::fs::PermissionsExt;
        let at = scratch("partway");
        let target = at.join("db");
        std::fs::create_dir_all(target.join("Sega")).unwrap();
        std::fs::write(target.join("Sega/Old.cht"), "old").unwrap();
        std::fs::create_dir_all(target.join("Locked")).unwrap();
        std::fs::write(target.join("Locked/Kept.cht"), "kept").unwrap();
        std::fs::set_permissions(target.join("Locked"), std::fs::Permissions::from_mode(0o000)).unwrap();
        let zip = archive(&at, &[("Sega/Old.cht", "new old"), ("Sega/New.cht", "new")]);
        let failed = install_staged(&zip, target.to_str().unwrap());
        std::fs::set_permissions(target.join("Locked"), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(failed, Err(Refusal::Io(_))), "a folder of the database that cannot be read fails the install: {failed:?}");
        assert_eq!(std::fs::read_to_string(target.join("Sega/Old.cht")).unwrap(), "old", "the file the archive would replace is untouched");
        assert!(!target.join("Sega/New.cht").exists(), "no file of the archive reached the database");
        assert_eq!(std::fs::read_to_string(target.join("Locked/Kept.cht")).unwrap(), "kept");
        assert!(!at.join("db.part").exists() && !at.join("db.old").exists());
        std::fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn a_swap_stopped_between_its_renames_is_put_back_before_the_next_install() {
        let at = scratch("swap-stopped");
        std::fs::create_dir_all(at.join("db.old/Sega")).unwrap();
        std::fs::write(at.join("db.old/Sega/Old.cht"), "old").unwrap();
        std::fs::write(at.join("db.old").join(MARK), "").unwrap();
        let target = at.join("db");
        let zip = archive(&at, &[("Sega/New.cht", "new")]);
        assert_eq!(install_staged(&zip, target.to_str().unwrap()).unwrap(), Installed { installed: 1, skipped: 0 });
        assert_eq!(std::fs::read_to_string(target.join("Sega/Old.cht")).unwrap(), "old", "the database the stopped swap left in db.old is kept");
        assert!(target.join("Sega/New.cht").exists() && !at.join("db.old").exists() && !target.join(MARK).exists());
        std::fs::remove_dir_all(&at).unwrap();
    }

    // A `.part` and an `.old` the player made, each with a file: never removed, renamed or written into, with or
    // without a database beside them, and the refusal names the folder.
    #[test]
    fn folders_of_the_installs_names_that_no_install_made_are_refused_and_left_as_they_are() {
        for (database, folder) in [(true, "db.part"), (true, "db.old"), (false, "db.part"), (false, "db.old")] {
            let at = scratch("not-ours");
            let target = at.join("db");
            if database {
                std::fs::create_dir_all(target.join("Sega")).unwrap();
                std::fs::write(target.join("Sega/Old.cht"), "old").unwrap();
            }
            std::fs::create_dir_all(at.join(folder)).unwrap();
            std::fs::write(at.join(folder).join("Backup.cht"), "the player's").unwrap();
            let zip = archive(&at, &[("Sega/New.cht", "new")]);
            let refused = install_staged(&zip, target.to_str().unwrap());
            let Err(Refusal::NotOurs(words)) = refused else { panic!("{folder} with a database {database}: {refused:?}") };
            assert!(words.contains(&at.join(folder).display().to_string()), "the words name the folder: {words}");
            let held: Vec<_> = std::fs::read_dir(at.join(folder)).unwrap().map(|e| e.unwrap().file_name()).collect();
            assert_eq!(held, ["Backup.cht"], "{folder} holds what it held");
            assert_eq!(std::fs::read_to_string(at.join(folder).join("Backup.cht")).unwrap(), "the player's");
            assert_eq!(target.exists(), database, "the database is neither made nor removed");
            if database {
                assert_eq!(std::fs::read_to_string(target.join("Sega/Old.cht")).unwrap(), "old");
                assert!(!target.join("Sega/New.cht").exists());
            }
            std::fs::remove_dir_all(&at).unwrap();
        }
    }

    #[test]
    fn what_is_not_a_zip_is_refused_in_the_readers_words() {
        let target = scratch("refused");
        let refused = install(&[0u8; 100], target.to_str().unwrap());
        let _ = std::fs::remove_dir_all(&target);
        assert!(matches!(refused, Err(Refusal::Unreadable(ref words)) if !words.is_empty()), "{refused:?}");
    }

    #[test]
    fn only_cht_entries_inside_the_folder_are_written_with_their_times() {
        let at = scratch("cht");
        let source = at.join("source.cht");
        std::fs::write(&source, "cheats = 0\n").unwrap();
        let when = std::time::UNIX_EPOCH + std::time::Duration::from_nanos(1_700_000_000_123_456_700);
        let zip = at.join("cheats.zip");
        let mut w = ZipWriter::create(zip.to_str().unwrap()).unwrap();
        for name in ["Nintendo - SNES/Game.cht", "Nintendo - SNES/readme.txt", "../outside.cht", "Sega/Other.CHT"] {
            w.add(name, source.to_str().unwrap(), when, false, &|| false).unwrap();
        }
        w.finish().unwrap();
        let target = at.join("db");
        let done = install(&std::fs::read(&zip).unwrap(), target.to_str().unwrap()).unwrap();
        assert_eq!(done, Installed { installed: 2, skipped: 2 });
        let written = target.join("Nintendo - SNES/Game.cht");
        assert_eq!(std::fs::read_to_string(&written).unwrap(), "cheats = 0\n");
        assert_eq!(std::fs::metadata(&written).unwrap().modified().unwrap(), when);
        assert!(target.join("Sega/Other.CHT").exists());
        assert!(!at.join("outside.cht").exists());
        std::fs::remove_dir_all(&at).unwrap();
    }
}
