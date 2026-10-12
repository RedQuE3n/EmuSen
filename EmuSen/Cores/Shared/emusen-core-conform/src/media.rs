//! An image as the runtime passes it (EmuSen_CoreAPI_Extensions.md §2.4, §2.6): a cue sheet with the files its FILE
//! lines name as `EMUSEN_FILE_MEDIA + n` and the `.sbi` found beside it, and an `.m3u` as the first disc it lists, which
//! is what reaches `create`; any other file alone. The lists are read by emusen-disc's set module, as the runtime's are.

use std::path::{Path, PathBuf};

use emusen_disc::set;
use emusen_native::core::flags::FILE_MEDIA;

/// Whether `path` names a sheet by its extension.
fn is(path: &Path, extension: &str) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case(extension))
}

/// The disc an `.m3u` opens with, its first entry beside it; any other path is itself.
pub fn first_disc(path: &Path) -> Result<PathBuf, String> {
    if !is(path, "m3u") {
        return Ok(path.to_owned());
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let first = set::playlist(&text).into_iter().next().ok_or_else(|| format!("{} lists no disc", path.display()))?;
    Ok(path.parent().unwrap_or(Path::new(".")).join(first))
}

/// The files to read beside the image at `path`: a sheet's in order, then its `.sbi` where one exists.
pub fn companions(path: &Path) -> Result<Vec<PathBuf>, String> {
    if !is(path, "cue") {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut files: Vec<PathBuf> = set::companions(&text).map_err(|e| format!("{}: {e}", path.display()))?.into_iter().map(|n| dir.join(n)).collect();
    let sbi = PathBuf::from(set::sbi_beside(&path.to_string_lossy()));
    if sbi.is_file() {
        files.push(sbi);
    }
    Ok(files)
}

/// The bytes `create` receives for the image at `path`, and its companions numbered from `FILE_MEDIA`.
pub fn open(path: &Path) -> Result<(Vec<u8>, Vec<(u32, Vec<u8>)>), String> {
    let path = first_disc(path)?;
    let image = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut files = Vec::new();
    for (n, file) in companions(&path)?.into_iter().enumerate() {
        files.push((FILE_MEDIA + n as u32, std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?));
    }
    Ok((image, files))
}

/// The bytes an image and its companions take, from their sizes on disk, for the corpus form's memory estimate.
pub fn size(path: &Path) -> u64 {
    let Ok(path) = first_disc(path) else { return 0 };
    let len = |p: &Path| std::fs::metadata(p).map_or(0, |m| m.len());
    len(&path) + companions(&path).unwrap_or_default().iter().map(|p| len(p)).sum::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_brings_its_tracks_and_its_sbi_and_a_list_its_first_disc() {
        let dir = std::env::temp_dir().join(format!("emusen-kit-media-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a (Track 1).bin"), [1u8; 4]).unwrap();
        std::fs::write(dir.join("a (Track 2).bin"), [2u8; 6]).unwrap();
        std::fs::write(dir.join("a.sbi"), b"SBI\0").unwrap();
        std::fs::write(dir.join("a.cue"), "FILE \"a (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"a (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n").unwrap();
        std::fs::write(dir.join("a.m3u"), "#EXTM3U\na.cue\nb.cue\n").unwrap();
        let (image, files) = open(&dir.join("a.m3u")).unwrap();
        assert!(image.starts_with(b"FILE"));
        assert_eq!(files.iter().map(|(w, d)| (*w, d.len())).collect::<Vec<_>>(), [(FILE_MEDIA, 4), (FILE_MEDIA + 1, 6), (FILE_MEDIA + 2, 4)]);
        assert_eq!(size(&dir.join("a.cue")), std::fs::metadata(dir.join("a.cue")).unwrap().len() + 14);
        assert!(open(&dir.join("a (Track 1).bin")).unwrap().1.is_empty());
        std::fs::remove_file(dir.join("a (Track 2).bin")).unwrap();
        assert!(open(&dir.join("a.cue")).unwrap_err().contains("a (Track 2).bin"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
