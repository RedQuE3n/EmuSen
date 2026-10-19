//! What a recogniser is shown of a library file's image, the C# `MediaProbe`: a file's own first bytes, a sheet's first
//! track's, an `.m3u`'s first disc's, an archive's image entry's. The library's scan and the runtime's route both read
//! a file through it, so the two route one way (EmuSen_Backend.md §28.7, §29.2).

use crate::dotnet_path::{self as path, Style};
use crate::{archive, dotnet_files, image_sets};
use std::io::Read;

/// A file's first bytes, at most `length`, as `MappedFile.Head` reads them.
pub fn head_of(file: &str, length: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    std::fs::File::open(file).map_err(|e| e.to_string())?.take(length as u64).read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Where an image's first bytes come from.
pub enum ImageProbe {
    Empty,
    File(String),
    Sheet(String),
    Entry(String, String),
    SheetEntry(String, String),
}

impl ImageProbe {
    /// The probe of a file; `takes` says which extensions an archive's entry may have.
    pub fn of(at: &str, takes: &dyn Fn(&str) -> bool) -> Result<ImageProbe, String> {
        if archive::is_archive(at) {
            return Ok(match archive::image_entry(at, takes)? {
                None => ImageProbe::Empty,
                Some(entry) if image_sets::is_sheet(&entry) => ImageProbe::SheetEntry(at.to_string(), entry),
                Some(entry) => ImageProbe::Entry(at.to_string(), entry),
            });
        }
        if image_sets::is_playlist(at) {
            return match image_sets::discs(at)?.into_iter().next() {
                Some(disc) if dotnet_files::file_exists(&disc) => ImageProbe::of(&disc, takes),
                _ => Ok(ImageProbe::Empty),
            };
        }
        if image_sets::is_sheet(at) {
            return Ok(ImageProbe::Sheet(at.to_string()));
        }
        Ok(ImageProbe::File(at.to_string()))
    }

    /// The image's first bytes, at most `length`.
    pub fn read(&self, length: usize) -> Result<Vec<u8>, String> {
        match self {
            ImageProbe::Empty => Ok(Vec::new()),
            ImageProbe::File(file) => head_of(file, length),
            ImageProbe::Sheet(sheet) => match image_sets::companions(sheet)?.into_iter().next() {
                Some(track) if dotnet_files::file_exists(&track) => head_of(&track, length),
                _ => Ok(Vec::new()),
            },
            ImageProbe::Entry(zip, entry) => archive::read_entry(zip, entry, length),
            ImageProbe::SheetEntry(zip, entry) => {
                let sheet = String::from_utf8_lossy(&archive::read_entry(zip, entry, 1 << 20)?).into_owned();
                let Some(first) = image_sets::companion_names(&sheet)?.into_iter().next().map(|f| f.replace('\\', "/")) else { return Ok(Vec::new()) };
                let dir = path::directory_name(Style::Unix, entry).unwrap_or_default();
                archive::read_entry(zip, &if dir.is_empty() { first } else { format!("{dir}/{first}") }, length)
            }
        }
    }
}

/// The extension that decides a file's system: an archive's image entry's, else its own, in lower case.
pub fn image_extension(at: &str, takes: &dyn Fn(&str) -> bool) -> String {
    if !archive::is_archive(at) {
        return emusen_dotnet::text::to_lower(path::extension(Style::HOST, at));
    }
    match archive::image_entry(at, takes) {
        Ok(Some(entry)) => emusen_dotnet::text::to_lower(path::extension(Style::Unix, &entry)),
        _ => String::new(),
    }
}
