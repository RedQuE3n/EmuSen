//! The zips a game, a package, a core's download or the cheat database comes in, read by the zip crate: the central
//! directory, the entry play opens, an entry's first bytes, an entry copied out whole and its time. Every form the
//! crate reads is taken (Zip64, data descriptors, stored, deflate and Deflate64); an encrypted entry or another method
//! is refused when it is read, in the crate's words. The crate bounds an entry by its declared length and checks its
//! CRC-32. See EmuSen_RustPlatform.md §47.15. Since game packages' step 7 a package's zip is also written, by the same
//! crate since §47.16, with each file's time to the tick (EmuSen_GamePackages.md §19).

use crate::image_sets;
use std::io::{Read, Seek, Write};
use std::path::Path;

/// `ArchiveImages.Extensions`.
pub const EXTENSIONS: &[&str] = &[".zip"];

/// `ArchiveImages.IsArchive`: by the extension, ignoring case.
pub fn is_archive(at: impl AsRef<Path>) -> bool {
    at.as_ref().extension().is_some_and(|extension| EXTENSIONS.iter().any(|e| e.strip_prefix('.').is_some_and(|e| extension.eq_ignore_ascii_case(e))))
}

/// One entry of a zip's central directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZipEntry {
    /// Its place in the directory, by which it is read.
    index: usize,
    pub full_name: String,
    pub length: u64,
    /// The CRC-32 the directory records for the entry's bytes.
    pub crc32: u32,
    /// Whether it is stored rather than compressed.
    pub stored: bool,
    /// Whether it is a symbolic link, by its Unix mode.
    pub link: bool,
    /// Where its name puts it under the folder it is unpacked into; none where the name would leave that folder.
    pub enclosed: Option<std::path::PathBuf>,
}

/// A zip opened for reading: its directory read once, its entries read by their place in it.
pub struct Zip<R> {
    archive: zip::ZipArchive<R>,
}

impl Zip<std::io::BufReader<std::fs::File>> {
    pub fn open(at: impl AsRef<Path>) -> Result<Self, String> {
        Zip::over(std::io::BufReader::new(std::fs::File::open(at).map_err(|e| e.to_string())?))
    }
}

impl<R: Read + Seek> Zip<R> {
    /// A zip read from any seekable source, a file or bytes in memory; one that names an entry twice is refused, since
    /// readers differ in which copy they take.
    pub fn over(source: R) -> Result<Self, String> {
        let archive = zip::ZipArchive::new(source).map_err(|e| e.to_string())?;
        if archive.has_duplicate_names() {
            return Err("the zip names an entry twice, and readers differ in which copy they take".into());
        }
        Ok(Zip { archive })
    }

    /// The entries, in the directory's order.
    pub fn entries(&self) -> Result<Vec<ZipEntry>, String> {
        (0..self.archive.len())
            .map(|index| {
                let e = self.archive.by_index_data(index).map_err(|x| x.to_string())?;
                let full_name = e.name().map_err(|x| x.to_string())?.into_owned();
                Ok(ZipEntry { index, full_name, length: e.size(), crc32: e.crc32(), stored: e.compression() == zip::CompressionMethod::Stored, link: e.is_symlink(), enclosed: e.enclosed_name() })
            })
            .collect()
    }

    /// An entry's first bytes, at most `length` of them.
    pub fn first_bytes(&mut self, e: &ZipEntry, length: usize) -> Result<Vec<u8>, String> {
        let file = self.archive.by_index(e.index).map_err(|x| format!("{}: {x}", e.full_name))?;
        let mut out = Vec::with_capacity((length as u64).min(e.length) as usize);
        file.take(length as u64).read_to_end(&mut out).map_err(|x| format!("{}: {x}", e.full_name))?;
        Ok(out)
    }

    /// An entry copied out whole into `out`; refused where its bytes are not the length and CRC-32 the zip records, so
    /// that a damaged zip is never taken for its files.
    pub fn copy(&mut self, e: &ZipEntry, out: &mut dyn Write) -> Result<(), String> {
        let mut file = self.archive.by_index(e.index).map_err(|x| format!("{}: {x}", e.full_name))?;
        let damaged = |why: String| format!("{} is damaged ({why}): the zip records {} bytes with CRC-32 {:08x}", e.full_name, e.length, e.crc32);
        let copied = std::io::copy(&mut file, out).map_err(|x| damaged(x.to_string()))?;
        if copied != e.length {
            return Err(damaged(format!("it holds {copied} bytes")));
        }
        Ok(())
    }

    /// An entry's modification time: the NTFS extra field's, to the tick, else the extended timestamp's, to the second;
    /// none where neither is there (the DOS time, local and to two seconds, is not taken).
    pub fn modified(&mut self, e: &ZipEntry) -> Option<std::time::SystemTime> {
        let file = self.archive.by_index_raw(e.index).ok()?;
        let mut seconds = None;
        for field in file.extra_data_fields() {
            match field {
                zip::ExtraField::Ntfs(ntfs) => return Some(from_filetime(ntfs.mtime())),
                zip::ExtraField::ExtendedTimestamp(t) => seconds = t.mod_time().map(|s| from_unix_seconds(s as i32 as i64)),
                _ => {}
            }
        }
        seconds
    }
}

/// FILETIME's count of ticks from 1601 to 1970.
const FILETIME_1970: u64 = 116_444_736_000_000_000;

fn from_filetime(ft: u64) -> std::time::SystemTime {
    let unix_ticks = ft as i128 - FILETIME_1970 as i128;
    let span = std::time::Duration::from_nanos((unix_ticks.unsigned_abs() * 100) as u64);
    if unix_ticks >= 0 { std::time::UNIX_EPOCH + span } else { std::time::UNIX_EPOCH - span }
}

fn from_unix_seconds(s: i64) -> std::time::SystemTime {
    let span = std::time::Duration::from_secs(s.unsigned_abs());
    if s >= 0 { std::time::UNIX_EPOCH + span } else { std::time::UNIX_EPOCH - span }
}

/// The entries of an archive held whole in memory.
pub fn entries_in(zip: &[u8]) -> Result<Vec<ZipEntry>, String> {
    Zip::over(std::io::Cursor::new(zip))?.entries()
}

/// The entries of an archive on disk, reading only its end and its directory.
pub fn entries_of(at: impl AsRef<Path>) -> Result<Vec<ZipEntry>, String> {
    Zip::open(at)?.entries()
}

/// `ArchiveImages.ImageEntry`: a cue sheet first, then a lone image of an extension `is_image` takes, by depth then by
/// name ignoring case; none where there is none.
pub fn image_entry(at: impl AsRef<Path>, is_image: impl Fn(&str) -> bool) -> Result<Option<String>, String> {
    let mut names: Vec<String> = entries_of(at)?.into_iter().map(|e| e.full_name).filter(|n| !n.ends_with('/')).collect();
    names.sort_by(|a, b| a.matches('/').count().cmp(&b.matches('/').count()).then_with(|| a.to_uppercase().cmp(&b.to_uppercase())));
    if let Some(sheet) = names.iter().find(|n| image_sets::is_sheet(n)) {
        return Ok(Some(sheet.clone()));
    }
    let dotted = |n: &str| Path::new(n).extension().map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
    Ok(names.into_iter().find(|n| !image_sets::is_playlist(n) && is_image(&dotted(n))))
}

/// `ArchiveImages.ReadEntry`: an entry's first bytes, at most `length` of them, found by its exact name and then
/// ignoring case.
pub fn read_entry(at: impl AsRef<Path>, entry: &str, length: usize) -> Result<Vec<u8>, String> {
    let mut zip = Zip::open(at)?;
    let entries = zip.entries()?;
    let e = entries.iter().find(|e| e.full_name == entry).or_else(|| entries.iter().find(|e| e.full_name.to_uppercase() == entry.to_uppercase())).ok_or_else(|| format!("the archive holds no {entry}"))?;
    zip.first_bytes(e, length)
}

/// An entry of the zip at `at` copied out whole, as [`Zip::copy`] copies it.
pub fn copy_entry(at: impl AsRef<Path>, e: &ZipEntry, out: &mut dyn Write) -> Result<(), String> {
    Zip::open(at)?.copy(e, out)
}

/// The extensions a writer stores rather than deflates: files already compressed, a CHD above all, which is never
/// compressed a second time (EmuSen_GamePackages.md §19).
pub const STORED_EXTENSIONS: &[&str] = &[".chd", ".zip", ".7z", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".mp4", ".mkv", ".webm", ".pdf"];

/// A zip written entry by entry to a file by the zip crate: stored or deflated entries, names in UTF-8, each with its
/// modification time in the DOS fields (local time), the extended timestamp (to the second) and the NTFS field (to the
/// tick, which a state's record is stamped with). Zip64 is written where an entry or the archive needs it.
pub struct ZipWriter {
    zip: zip::ZipWriter<std::fs::File>,
}

/// A time's DOS fields in local time; the first of 1980 for a time before it.
fn dos_time(t: std::time::SystemTime) -> zip::DateTime {
    let utc = crate::clock::ticks_of(t);
    let local = crate::clock::utc(utc + crate::clock::local_offset(utc.div_euclid(crate::clock::TICKS_PER_SECOND)) * crate::clock::TICKS_PER_SECOND);
    let year = u16::try_from(local.year()).ok();
    year.and_then(|y| zip::DateTime::from_date_and_time(y, local.month() as u8, local.day(), local.hour(), local.minute(), local.second()).ok()).unwrap_or_default()
}

/// The extended timestamp's body (0x5455): its flags, modification only, then Unix seconds.
fn extended_timestamp(t: std::time::SystemTime) -> [u8; 5] {
    let seconds = crate::clock::ticks_of(t).div_euclid(crate::clock::TICKS_PER_SECOND).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let mut body = [1u8; 5];
    body[1..].copy_from_slice(&seconds.to_le_bytes());
    body
}

/// The NTFS field's body (0x000A): four reserved bytes, then tag 1 of 24 bytes holding the modification, access and
/// creation FILETIMEs, all three the one time.
fn ntfs_times(t: std::time::SystemTime) -> [u8; 32] {
    let ft = (crate::clock::ticks_of(t) as i128 + FILETIME_1970 as i128).max(0) as u64;
    let mut body = [0u8; 32];
    body[4..6].copy_from_slice(&1u16.to_le_bytes());
    body[6..8].copy_from_slice(&24u16.to_le_bytes());
    for at in [8, 16, 24] {
        body[at..at + 8].copy_from_slice(&ft.to_le_bytes());
    }
    body
}

impl ZipWriter {
    pub fn create(at: impl AsRef<Path>) -> Result<ZipWriter, String> {
        let at = at.as_ref();
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(at).map_err(|e| format!("{}: {e}", at.display()))?;
        Ok(ZipWriter { zip: zip::ZipWriter::new(file) })
    }

    /// The file at `source` added as `name`, stored where `store` says and deflated otherwise, with `modified` as its
    /// time; `stop` is asked between each MiB.
    pub fn add(&mut self, name: &str, source: impl AsRef<Path>, modified: std::time::SystemTime, store: bool, stop: &dyn Fn() -> bool) -> Result<(), String> {
        let source = source.as_ref();
        let mut input = std::fs::File::open(source).map_err(|e| format!("{}: {e}", source.display()))?;
        let length = input.metadata().map_err(|e| format!("{}: {e}", source.display()))?.len();
        let (method, level) = if store { (zip::CompressionMethod::Stored, None) } else { (zip::CompressionMethod::Deflated, Some(6)) };
        let mut options = zip::write::FullFileOptions::default()
            .compression_method(method)
            .compression_level(level)
            .last_modified_time(dos_time(modified))
            .unix_permissions(0o644)
            .large_file(length > u32::MAX as u64)
            .validate_extra_fields(true);
        let field = |e: zip::result::ZipError| format!("{name}: {e}");
        options.add_extra_field(0x5455, extended_timestamp(modified), false).map_err(field)?;
        options.add_extra_field(0x000A, ntfs_times(modified), false).map_err(field)?;
        self.zip.start_file(name, options).map_err(field)?;
        let mut buffer = vec![0u8; 1 << 20];
        let mut copied = 0u64;
        loop {
            if stop() {
                return Err("stopped".into());
            }
            let n = input.read(&mut buffer).map_err(|e| format!("{}: {e}", source.display()))?;
            if n == 0 {
                break;
            }
            self.zip.write_all(&buffer[..n]).map_err(|e| format!("{name}: {e}"))?;
            copied += n as u64;
        }
        if copied != length {
            return Err(format!("{} changed while it was written", source.display()));
        }
        Ok(())
    }

    /// The central directory and the end record written, and the file flushed to the disk.
    pub fn finish(self) -> Result<(), String> {
        self.zip.finish().map_err(|e| e.to_string())?.sync_all().map_err(|e| e.to_string())
    }
}
