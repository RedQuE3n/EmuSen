//! Installing a downloaded cheat archive: every `.cht` entry of a zip written under the target folder where its name
//! puts it, with the entry's own time where the zip records one; anything else, or a name that would leave the folder,
//! counted as skipped. The archive is read by Galaxia's `archive`, the platform's one zip reader, and one it refuses is
//! refused in the zip crate's words. The download is not here (EmuSen_Backend.md SR9). See EmuSen_Backend.md §15.3 and
//! EmuSen_RustPlatform.md §47.15.

use emusen_galaxia::archive::{Zip, ZipEntry};
use std::io::Cursor;
use std::path::Path;

/// What an install did: entries written, and entries passed over or that failed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Installed {
    pub installed: usize,
    pub skipped: usize,
}

/// Why nothing was installed: the archive is no zip the reader takes, or the target was not made.
#[derive(Debug)]
pub enum Refusal {
    Unreadable(String),
    Io(std::io::Error),
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
