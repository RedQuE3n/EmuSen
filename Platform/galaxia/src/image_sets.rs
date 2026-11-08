//! The files a disc image brings beside it, the C# `ImageSets`: a sheet's tracks, an `.m3u`'s discs and an `.sbi`,
//! each found as the folder holds it, and a folder's files as a library shelves them. What each list means is
//! `emusen-disc`'s set module. See EmuSen_Galaxia.md §3.8 and EmuSen_Backend.md §28.2.

use crate::files::{directory_exists, file_exists};
use crate::path_key;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn extension_is(at: &Path, extension: &str) -> bool {
    at.extension().is_some_and(|e| e.eq_ignore_ascii_case(extension))
}

pub fn is_sheet(at: impl AsRef<Path>) -> bool {
    extension_is(at.as_ref(), "cue")
}

pub fn is_playlist(at: impl AsRef<Path>) -> bool {
    extension_is(at.as_ref(), "m3u")
}

/// A name as the folder holds it: each part matched exactly, else by the one entry matching it ignoring case, either
/// slash separating parts; failing that, as written. Reading a folder that will not be read fails.
pub fn beside(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let written = dir.join(name);
    if Path::new(name).has_root() {
        return Ok(written);
    }
    let mut at = dir.to_path_buf();
    for part in name.split(['/', '\\']).filter(|p| !p.is_empty()) {
        if part == "." || part == ".." {
            at.push(part);
            continue;
        }
        if !directory_exists(&at) {
            return Ok(written);
        }
        let mut found = Vec::new();
        for entry in std::fs::read_dir(&at).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let entry = entry.file_name().to_string_lossy().into_owned();
            if entry.to_uppercase() == part.to_uppercase() {
                found.push(entry);
            }
        }
        let held = if found.iter().any(|f| f == part) { part.to_string() } else if found.len() == 1 { found.remove(0) } else { return Ok(written) };
        at.push(held);
    }
    Ok(at)
}

fn folder_of(at: &Path) -> PathBuf {
    path_key::key(at).parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// The names a sheet's FILE lines give, in its order.
pub fn companion_names(sheet: &str) -> Result<Vec<String>, String> {
    emusen_disc::set::companions(sheet).map_err(|e| format!("{e:?}"))
}

/// The files a sheet's FILE lines name, as paths beside it.
pub fn companions(sheet_path: impl AsRef<Path>) -> Result<Vec<PathBuf>, String> {
    let sheet_path = sheet_path.as_ref();
    let dir = folder_of(sheet_path);
    let text = crate::text_file::read(sheet_path)?;
    companion_names(&text)?.iter().map(|n| beside(&dir, n)).collect()
}

/// The discs an `.m3u` lists, as paths beside it.
pub fn discs(list_path: impl AsRef<Path>) -> Result<Vec<PathBuf>, String> {
    let list_path = list_path.as_ref();
    let dir = folder_of(list_path);
    let text = crate::text_file::read(list_path)?;
    emusen_disc::set::playlist(&text).iter().map(|n| beside(&dir, n)).collect()
}

/// The files a core receives beside the image: a sheet's, then an `.sbi` found beside it.
pub fn files_for(image: impl AsRef<Path>) -> Result<Vec<PathBuf>, String> {
    let image = image.as_ref();
    let mut paths = if is_sheet(image) { companions(image)? } else { Vec::new() };
    if !paths.is_empty() || extension_is(image, "bin") || extension_is(image, "iso") {
        let sbi = PathBuf::from(emusen_disc::set::sbi_beside(&image.to_string_lossy()));
        if file_exists(&sbi) {
            paths.push(sbi);
        }
    }
    Ok(paths)
}

/// Every file an image stands for on disk: itself, a sheet's tracks and an `.sbi`; for an `.m3u`, every disc's.
pub fn all_files(at: impl AsRef<Path>) -> Result<Vec<PathBuf>, String> {
    let at = at.as_ref();
    let mut all = vec![at.to_path_buf()];
    if is_playlist(at) {
        for disc in discs(at)? {
            all.extend(all_files(&disc)?);
        }
    } else {
        all.extend(files_for(at)?);
    }
    Ok(all)
}

/// A folder's files as a library shelves them: a file that a sheet or a list among them names is that game's.
pub fn shelved(files: Vec<String>) -> Vec<String> {
    let mut named = HashSet::new();
    for f in files.iter().filter(|f| is_sheet(f) || is_playlist(f)) {
        if let Ok(all) = all_files(f) {
            named.extend(all.iter().skip(1).map(path_key::key));
        }
    }
    if named.is_empty() {
        return files;
    }
    files.into_iter().filter(|f| !named.contains(&path_key::key(f))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn a_sheets_tracks_and_a_lists_discs_are_not_games_of_their_own() {
        let temp = TempDir::new("sets");
        std::fs::write(temp.join("Game.cue"), "FILE \"game (track 1).BIN\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n").unwrap();
        std::fs::write(temp.join("Game (Track 1).bin"), b"").unwrap();
        std::fs::write(temp.join("Two.m3u"), "Two (Disc 1).iso\nTwo (Disc 2).iso\n").unwrap();
        std::fs::write(temp.join("Two (Disc 1).iso"), b"").unwrap();
        std::fs::write(temp.join("Two (Disc 2).iso"), b"").unwrap();
        std::fs::write(temp.join("Two (Disc 2).sbi"), b"").unwrap();
        std::fs::write(temp.join("Lone.iso"), b"").unwrap();
        let files: Vec<String> = ["Game.cue", "Game (Track 1).bin", "Two.m3u", "Two (Disc 1).iso", "Two (Disc 2).iso", "Lone.iso"].iter().map(|f| temp.join(f).to_string_lossy().into_owned()).collect();
        let kept: Vec<String> = shelved(files).iter().map(|f| Path::new(f).file_name().unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(kept, ["Game.cue", "Two.m3u", "Lone.iso"]);
        assert_eq!(all_files(&temp.join("Two.m3u")).unwrap().len(), 4);
    }
}
