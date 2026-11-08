//! The zips a game, a package, a core's download or the cheat database comes in, read by the zip crate: the central
//! directory, the entry play opens, an entry's first bytes, an entry copied out whole and its time. Every form the
//! crate reads is taken (Zip64, data descriptors, stored, deflate and Deflate64); an encrypted entry or another method
//! is refused when it is read, in the crate's words. The crate bounds an entry by its declared length and checks its
//! CRC-32. See EmuSen_RustPlatform.md §47.15. Since game packages' step 7 the same plain zip is also written, with each
//! file's time to the tick (EmuSen_GamePackages.md §19).

use crate::image_sets;
use std::io::{Read, Seek, SeekFrom, Write};

/// `ArchiveImages.Extensions`.
pub const EXTENSIONS: &[&str] = &[".zip"];

/// `ArchiveImages.IsArchive`: by the extension, ignoring case.
pub fn is_archive(at: &str) -> bool {
    let extension = crate::dotnet_path::extension(crate::dotnet_path::Style::HOST, at);
    EXTENSIONS.iter().any(|e| e.eq_ignore_ascii_case(&extension))
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
    pub fn open(at: &str) -> Result<Self, String> {
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
pub fn entries_of(at: &str) -> Result<Vec<ZipEntry>, String> {
    Zip::open(at)?.entries()
}

/// `ArchiveImages.ImageEntry`: a cue sheet first, then a lone image of an extension `is_image` takes, by depth then by
/// name ignoring case; none where there is none.
pub fn image_entry(at: &str, is_image: impl Fn(&str) -> bool) -> Result<Option<String>, String> {
    let mut names: Vec<String> = entries_of(at)?.into_iter().map(|e| e.full_name).filter(|n| !n.ends_with('/')).collect();
    names.sort_by(|a, b| a.matches('/').count().cmp(&b.matches('/').count()).then_with(|| a.to_uppercase().cmp(&b.to_uppercase())));
    if let Some(sheet) = names.iter().find(|n| image_sets::is_sheet(n)) {
        return Ok(Some(sheet.clone()));
    }
    Ok(names.into_iter().find(|n| !image_sets::is_playlist(n) && is_image(crate::dotnet_path::extension(crate::dotnet_path::Style::Unix, n))))
}

/// `ArchiveImages.ReadEntry`: an entry's first bytes, at most `length` of them, found by its exact name and then
/// ignoring case.
pub fn read_entry(at: &str, entry: &str, length: usize) -> Result<Vec<u8>, String> {
    let mut zip = Zip::open(at)?;
    let entries = zip.entries()?;
    let e = entries.iter().find(|e| e.full_name == entry).or_else(|| entries.iter().find(|e| e.full_name.to_uppercase() == entry.to_uppercase())).ok_or_else(|| format!("the archive holds no {entry}"))?;
    zip.first_bytes(e, length)
}

/// An entry of the zip at `at` copied out whole, as [`Zip::copy`] copies it.
pub fn copy_entry(at: &str, e: &ZipEntry, out: &mut dyn Write) -> Result<(), String> {
    Zip::open(at)?.copy(e, out)
}

/// The extensions a writer stores rather than deflates: files already compressed, a CHD above all, which is never
/// compressed a second time (EmuSen_GamePackages.md §19).
pub const STORED_EXTENSIONS: &[&str] = &[".chd", ".zip", ".7z", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".mp4", ".mkv", ".webm", ".pdf"];

/// What a plain zip without Zip64 can hold: 65,535 entries and offsets and sizes under 4 GiB.
pub const MAX_ENTRIES: usize = 0xFFFF;
pub const MAX_OFFSET: u64 = 0xFFFF_FFFE;

/// A zip written entry by entry to a file, then its central directory and end record: stored or deflated entries,
/// names in UTF-8, each with its modification time in the DOS fields (local time), the extended timestamp (to the
/// second) and the NTFS field (to the tick, which a state's record is stamped with).
pub struct ZipWriter {
    file: std::fs::File,
    at: u64,
    directory: Vec<u8>,
    entries: usize,
}

const LOCAL: u32 = 0x0403_4B50;
const CENTRAL: u32 = 0x0201_4B50;
const END: u32 = 0x0605_4B50;

fn dos_time(t: std::time::SystemTime) -> (u16, u16) {
    let utc = crate::clock::ticks_of(t).div_euclid(emusen_dotnet::date::TICKS_PER_SECOND);
    let local = utc + crate::clock::local_offset(utc);
    let (y, m, d) = emusen_dotnet::date::civil(local.div_euclid(86_400));
    if y < 1980 {
        return (0, (1 << 5) | 1);
    }
    let s = local.rem_euclid(86_400);
    let time = ((s / 3600) << 11 | (s / 60 % 60) << 5 | (s % 60) / 2) as u16;
    let date = (((y - 1980).min(127) as u32) << 9 | m << 5 | d) as u16;
    (time, date)
}

fn time_fields(t: std::time::SystemTime) -> Vec<u8> {
    let ticks = crate::clock::ticks_of(t);
    let ft = (ticks as i128 + FILETIME_1970 as i128).max(0) as u64;
    let seconds = ticks.div_euclid(emusen_dotnet::date::TICKS_PER_SECOND).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let mut extra = Vec::with_capacity(45);
    extra.extend_from_slice(&0x5455u16.to_le_bytes());
    extra.extend_from_slice(&5u16.to_le_bytes());
    extra.push(1);
    extra.extend_from_slice(&seconds.to_le_bytes());
    extra.extend_from_slice(&0x000Au16.to_le_bytes());
    extra.extend_from_slice(&32u16.to_le_bytes());
    extra.extend_from_slice(&[0, 0, 0, 0]);
    extra.extend_from_slice(&1u16.to_le_bytes());
    extra.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..3 {
        extra.extend_from_slice(&ft.to_le_bytes());
    }
    extra
}

impl ZipWriter {
    pub fn create(at: &str) -> Result<ZipWriter, String> {
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(at).map_err(|e| format!("{at}: {e}"))?;
        Ok(ZipWriter { file, at: 0, directory: Vec::new(), entries: 0 })
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.file.write_all(bytes).map_err(|e| e.to_string())?;
        self.at += bytes.len() as u64;
        Ok(())
    }

    /// The file at `source` added as `name`, stored where `store` says or where deflating would not make it smaller,
    /// with `modified` as its time; refused past what a plain zip holds.
    pub fn add(&mut self, name: &str, source: &str, modified: std::time::SystemTime, store: bool, stop: &dyn Fn() -> bool) -> Result<(), String> {
        if self.entries + 1 > MAX_ENTRIES - 1 {
            return Err(format!("more than {} files, which a zip without Zip64 does not hold", MAX_ENTRIES - 1));
        }
        let length = std::fs::metadata(source).map_err(|e| format!("{source}: {e}"))?.len();
        if length > MAX_OFFSET || self.at > MAX_OFFSET {
            return Err(format!("{name} reaches past 4 GiB, which a zip without Zip64 does not hold"));
        }
        let mut crc = crate::rom_hash::Crc32::default();
        let deflated: Option<Vec<u8>> = if store {
            None
        } else {
            let bytes = std::fs::read(source).map_err(|e| format!("{source}: {e}"))?;
            crc.append(&bytes);
            let mut encoder = flate2::write::DeflateEncoder::new(Vec::with_capacity(bytes.len() / 2), flate2::Compression::new(6));
            encoder.write_all(&bytes).map_err(|e| e.to_string())?;
            let packed = encoder.finish().map_err(|e| e.to_string())?;
            if (packed.len() as u64) < length { Some(packed) } else { None }
        };
        let method: u16 = if deflated.is_some() { 8 } else { 0 };
        let (time, date) = dos_time(modified);
        let extra = time_fields(modified);
        let offset = self.at;
        let header = |crc: u32, compressed: u32, out: &mut Vec<u8>| {
            out.extend_from_slice(&LOCAL.to_le_bytes());
            out.extend_from_slice(&20u16.to_le_bytes());
            out.extend_from_slice(&0x0800u16.to_le_bytes());
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&time.to_le_bytes());
            out.extend_from_slice(&date.to_le_bytes());
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&compressed.to_le_bytes());
            out.extend_from_slice(&(length as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&(extra.len() as u16).to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&extra);
        };
        let (crc_value, compressed) = match &deflated {
            Some(packed) => {
                let mut h = Vec::new();
                header(crc.value(), packed.len() as u32, &mut h);
                self.write(&h)?;
                self.write(packed)?;
                (crc.value(), packed.len() as u64)
            }
            None => {
                let mut h = Vec::new();
                header(0, length as u32, &mut h);
                self.write(&h)?;
                let mut input = std::fs::File::open(source).map_err(|e| format!("{source}: {e}"))?;
                let mut crc = crate::rom_hash::Crc32::default();
                let mut buffer = vec![0u8; 1 << 20];
                let mut copied = 0u64;
                loop {
                    if stop() {
                        return Err("stopped".into());
                    }
                    let n = input.read(&mut buffer).map_err(|e| format!("{source}: {e}"))?;
                    if n == 0 {
                        break;
                    }
                    crc.append(&buffer[..n]);
                    self.write(&buffer[..n])?;
                    copied += n as u64;
                }
                if copied != length {
                    return Err(format!("{source} changed while it was written"));
                }
                self.file.seek(SeekFrom::Start(offset + 14)).map_err(|e| e.to_string())?;
                self.file.write_all(&crc.value().to_le_bytes()).map_err(|e| e.to_string())?;
                self.file.seek(SeekFrom::Start(self.at)).map_err(|e| e.to_string())?;
                (crc.value(), length)
            }
        };
        if self.at > MAX_OFFSET {
            return Err(format!("{name} reaches past 4 GiB, which a zip without Zip64 does not hold"));
        }
        let d = &mut self.directory;
        d.extend_from_slice(&CENTRAL.to_le_bytes());
        d.extend_from_slice(&((3u16 << 8) | 20).to_le_bytes());
        d.extend_from_slice(&20u16.to_le_bytes());
        d.extend_from_slice(&0x0800u16.to_le_bytes());
        d.extend_from_slice(&method.to_le_bytes());
        d.extend_from_slice(&time.to_le_bytes());
        d.extend_from_slice(&date.to_le_bytes());
        d.extend_from_slice(&crc_value.to_le_bytes());
        d.extend_from_slice(&(compressed as u32).to_le_bytes());
        d.extend_from_slice(&(length as u32).to_le_bytes());
        d.extend_from_slice(&(name.len() as u16).to_le_bytes());
        d.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&(0o100644u32 << 16).to_le_bytes());
        d.extend_from_slice(&(offset as u32).to_le_bytes());
        d.extend_from_slice(name.as_bytes());
        d.extend_from_slice(&extra);
        self.entries += 1;
        Ok(())
    }

    /// The central directory and the end record written, and the file flushed to the disk.
    pub fn finish(mut self) -> Result<(), String> {
        let (start, size) = (self.at, self.directory.len() as u64);
        if start > MAX_OFFSET || start + size > MAX_OFFSET {
            return Err("the zip's directory would begin past 4 GiB, which a zip without Zip64 does not hold".into());
        }
        let directory = std::mem::take(&mut self.directory);
        self.write(&directory)?;
        let mut end = Vec::with_capacity(22);
        end.extend_from_slice(&END.to_le_bytes());
        end.extend_from_slice(&[0, 0, 0, 0]);
        end.extend_from_slice(&(self.entries as u16).to_le_bytes());
        end.extend_from_slice(&(self.entries as u16).to_le_bytes());
        end.extend_from_slice(&(size as u32).to_le_bytes());
        end.extend_from_slice(&(start as u32).to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        self.write(&end)?;
        self.file.sync_all().map_err(|e| e.to_string())
    }
}
