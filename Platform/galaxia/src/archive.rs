//! The archives a game may be packed in, the C# `ArchiveImages`' reading: a zip's central directory, the entry play
//! opens, and an entry's first bytes inflated without unpacking. Only a plain zip of stored and deflated entries is read;
//! the central directory's reader is shared with `emusen-cheats`' installer. See EmuSen_Backend.md §28.2. Since game
//! packages' step 7 the same plain zip is also written, with each file's time to the tick, and an entry is copied out
//! whole and checked against its CRC-32 (EmuSen_GamePackages.md §19). Written from PKWARE's APPNOTE.TXT, never from an
//! archiver's source.

use crate::image_sets;
use emusen_dotnet::text::{compare_ignore_case, equals_ignore_case};
use std::io::{Read, Seek, SeekFrom};

/// `ArchiveImages.Extensions`.
pub const EXTENSIONS: &[&str] = &[".zip"];

/// `ArchiveImages.IsArchive`: by the extension, ignoring case.
pub fn is_archive(at: &str) -> bool {
    let extension = crate::dotnet_path::extension(crate::dotnet_path::Style::HOST, at);
    EXTENSIONS.iter().any(|e| equals_ignore_case(e, extension))
}

/// One entry of a zip's central directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZipEntry {
    pub platform: u8,
    pub method: u16,
    pub dos_time: u32,
    pub compressed: usize,
    pub length: u64,
    pub external: u32,
    pub offset: usize,
    pub full_name: String,
    /// The CRC-32 the directory records for the entry's bytes.
    pub crc32: u32,
    /// The entry's modification time: the NTFS extra field's, to the tick, else the extended timestamp's, to the second;
    /// none where neither is there (the DOS time, local and to two seconds, is not taken).
    pub modified: Option<std::time::SystemTime>,
}

/// Where a zip's central directory is, from its end record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct End {
    pub start: u64,
    pub size: u64,
    pub total: usize,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// The end record in the last bytes of an archive, `tail`, which begin `tail_offset` bytes in; a form not read here
/// is refused with why.
pub fn end_of(tail: &[u8]) -> Result<End, String> {
    let lowest = tail.len().saturating_sub(22 + 0xFFFF);
    let end = (lowest..=tail.len().saturating_sub(22)).rev().find(|&at| u32_at(tail, at) == Some(0x0605_4B50));
    let Some(end) = end.filter(|_| tail.len() >= 22) else { return Err("no end of central directory".into()) };
    if end >= 20 && u32_at(tail, end - 20) == Some(0x0706_4B50) {
        return Err("a Zip64 archive".into());
    }
    let (disk, cd_disk) = (u16_at(tail, end + 4).unwrap_or(1), u16_at(tail, end + 6).unwrap_or(1));
    let (on_disk, total) = (u16_at(tail, end + 8).unwrap_or(0), u16_at(tail, end + 10).unwrap_or(0));
    let (size, start) = (u32_at(tail, end + 12).unwrap_or(0), u32_at(tail, end + 16).unwrap_or(0));
    if disk != 0 || cd_disk != 0 || on_disk != total || total == 0xFFFF || size == u32::MAX || start == u32::MAX {
        return Err("a spanned or Zip64 archive".into());
    }
    Ok(End { start: start as u64, size: size as u64, total: total as usize })
}

/// The entries of a central directory that begins `at` bytes into `bytes`, `total` of them by the end record.
pub fn directory(bytes: &[u8], mut at: usize, total: usize) -> Result<Vec<ZipEntry>, String> {
    let mut list = Vec::new();
    while u32_at(bytes, at) == Some(0x0201_4B50) {
        let h = |o: usize| u16_at(bytes, at + o).ok_or_else(|| "a short header".to_string());
        let w = |o: usize| u32_at(bytes, at + o).ok_or_else(|| "a short header".to_string());
        let (made_by, flags, method) = (h(4)?, h(8)?, h(10)?);
        let (dos_time, crc32) = (w(12)?, w(16)?);
        let (compressed, length, n, m, k) = (w(20)?, w(24)?, h(28)? as usize, h(30)? as usize, h(32)? as usize);
        let (external, offset) = (w(38)?, w(42)?);
        if compressed == u32::MAX || length == u32::MAX || offset == u32::MAX || h(34)? != 0 {
            return Err("a Zip64 or spanned entry".into());
        }
        if flags & 1 != 0 || (method != 0 && method != 8) {
            return Err("an encrypted entry or a method other than stored and deflate".into());
        }
        let Some(name) = bytes.get(at + 46..at + 46 + n) else { return Err("a short name".into()) };
        let full_name = String::from_utf8_lossy(name).into_owned();
        if full_name.contains('\0') {
            return Err("a name holding a NUL".into());
        }
        let extra = bytes.get(at + 46 + n..at + 46 + n + m).unwrap_or(&[]);
        list.push(ZipEntry { platform: (made_by >> 8) as u8, method, dos_time, compressed: compressed as usize, length: length as u64, external, offset: offset as usize, full_name, crc32, modified: modified_of(extra) });
        at += 46 + n + m + k;
    }
    if list.len() != total {
        return Err("an entry count the directory does not hold".into());
    }
    Ok(list)
}

/// The entries of an archive held whole in memory.
pub fn entries_in(zip: &[u8]) -> Result<Vec<ZipEntry>, String> {
    let end = end_of(zip)?;
    directory(zip, end.start as usize, end.total)
}

/// The entries of an archive on disk, reading only its end and its directory.
pub fn entries_of(at: &str) -> Result<Vec<ZipEntry>, String> {
    let mut file = std::fs::File::open(at).map_err(|e| e.to_string())?;
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    let tail_length = length.min(22 + 0xFFFF + 20);
    file.seek(SeekFrom::Start(length - tail_length)).map_err(|e| e.to_string())?;
    let mut tail = vec![0u8; tail_length as usize];
    file.read_exact(&mut tail).map_err(|e| e.to_string())?;
    let end = end_of(&tail)?;
    if end.start.checked_add(end.size).is_none_or(|stop| stop > length) {
        return Err("a central directory past the end".into());
    }
    file.seek(SeekFrom::Start(end.start)).map_err(|e| e.to_string())?;
    let mut bytes = vec![0u8; end.size as usize];
    file.read_exact(&mut bytes).map_err(|e| e.to_string())?;
    directory(&bytes, 0, end.total)
}

/// `ArchiveImages.ImageEntry`: a cue sheet first, then a lone image of an extension `is_image` takes, by depth then by
/// name ignoring case; none where there is none.
pub fn image_entry(at: &str, is_image: impl Fn(&str) -> bool) -> Result<Option<String>, String> {
    let mut names: Vec<String> = entries_of(at)?.into_iter().map(|e| e.full_name).filter(|n| !n.ends_with('/')).collect();
    names.sort_by(|a, b| a.matches('/').count().cmp(&b.matches('/').count()).then_with(|| compare_ignore_case(a, b)));
    if let Some(sheet) = names.iter().find(|n| image_sets::is_sheet(n)) {
        return Ok(Some(sheet.clone()));
    }
    Ok(names.into_iter().find(|n| !image_sets::is_playlist(n) && is_image(crate::dotnet_path::extension(crate::dotnet_path::Style::Unix, n))))
}

/// `ArchiveImages.ReadEntry`: an entry's first bytes, at most `length` of them, found by its exact name and then
/// ignoring case.
pub fn read_entry(at: &str, entry: &str, length: usize) -> Result<Vec<u8>, String> {
    let entries = entries_of(at)?;
    let e = entries.iter().find(|e| e.full_name == entry).or_else(|| entries.iter().find(|e| equals_ignore_case(&e.full_name, entry))).ok_or_else(|| format!("the archive holds no {entry}"))?;
    let mut file = std::fs::File::open(at).map_err(|x| x.to_string())?;
    let mut local = [0u8; 30];
    file.seek(SeekFrom::Start(e.offset as u64)).map_err(|x| x.to_string())?;
    file.read_exact(&mut local).map_err(|x| x.to_string())?;
    if u32_at(&local, 0) != Some(0x0403_4B50) {
        return Err("a damaged local header".into());
    }
    let data = e.offset as u64 + 30 + u16_at(&local, 26).unwrap_or(0) as u64 + u16_at(&local, 28).unwrap_or(0) as u64;
    file.seek(SeekFrom::Start(data)).map_err(|x| x.to_string())?;
    let want = (length as u64).min(e.length) as usize;
    let raw = file.take(e.compressed as u64);
    let mut out = Vec::with_capacity(want);
    let read = if e.method == 0 { raw.take(want as u64).read_to_end(&mut out) } else { flate2::read::DeflateDecoder::new(raw).take(want as u64).read_to_end(&mut out) };
    read.map_err(|x| x.to_string())?;
    Ok(out)
}

/// FILETIME's count of ticks from 1601 to 1970.
const FILETIME_1970: u64 = 116_444_736_000_000_000;

/// A central directory entry's time from its extra fields: the NTFS field (0x000A, tag 1, three FILETIMEs of which the
/// first is the modification), else the extended timestamp (0x5455, its flags' bit 0 then four bytes of Unix seconds).
fn modified_of(extra: &[u8]) -> Option<std::time::SystemTime> {
    let (mut at, mut seconds) = (0usize, None);
    while at + 4 <= extra.len() {
        let (id, size) = (u16_at(extra, at)?, u16_at(extra, at + 2)? as usize);
        let field = extra.get(at + 4..at + 4 + size)?;
        if id == 0x000A && size >= 32 && u16_at(field, 4) == Some(1) && u16_at(field, 6).is_some_and(|s| s >= 24) {
            let ft = u64::from_le_bytes(field[8..16].try_into().ok()?);
            let unix_ticks = ft as i128 - FILETIME_1970 as i128;
            let at = if unix_ticks >= 0 {
                std::time::UNIX_EPOCH + std::time::Duration::from_nanos(unix_ticks as u64 * 100)
            } else {
                std::time::UNIX_EPOCH - std::time::Duration::from_nanos((-unix_ticks) as u64 * 100)
            };
            return Some(at);
        }
        if id == 0x5455 && size >= 5 && field[0] & 1 != 0 {
            let s = i32::from_le_bytes(field[1..5].try_into().ok()?) as i64;
            seconds = Some(if s >= 0 { std::time::UNIX_EPOCH + std::time::Duration::from_secs(s as u64) } else { std::time::UNIX_EPOCH - std::time::Duration::from_secs((-s) as u64) });
        }
        at += 4 + size;
    }
    seconds
}

/// An entry copied out whole into `out`, inflated where deflated, and checked: its length and CRC-32 against the
/// directory's, refused with both where they differ, so that a damaged zip is never taken for its files.
pub fn copy_entry(at: &str, e: &ZipEntry, out: &mut dyn std::io::Write) -> Result<(), String> {
    let mut file = std::fs::File::open(at).map_err(|x| x.to_string())?;
    let mut local = [0u8; 30];
    file.seek(SeekFrom::Start(e.offset as u64)).map_err(|x| x.to_string())?;
    file.read_exact(&mut local).map_err(|x| x.to_string())?;
    if u32_at(&local, 0) != Some(0x0403_4B50) {
        return Err(format!("{} has a damaged local header", e.full_name));
    }
    let data = e.offset as u64 + 30 + u16_at(&local, 26).unwrap_or(0) as u64 + u16_at(&local, 28).unwrap_or(0) as u64;
    file.seek(SeekFrom::Start(data)).map_err(|x| x.to_string())?;
    let raw = file.take(e.compressed as u64);
    let mut reader: Box<dyn Read> = if e.method == 0 { Box::new(raw) } else { Box::new(flate2::read::DeflateDecoder::new(raw)) };
    let mut crc = crate::rom_hash::Crc32::default();
    let mut buffer = vec![0u8; 1 << 16];
    let mut total = 0u64;
    loop {
        let n = reader.read(&mut buffer).map_err(|x| format!("{}: {x}", e.full_name))?;
        if n == 0 {
            break;
        }
        crc.append(&buffer[..n]);
        total += n as u64;
        if total > e.length {
            break;
        }
        out.write_all(&buffer[..n]).map_err(|x| format!("{}: {x}", e.full_name))?;
    }
    if total != e.length || crc.value() != e.crc32 {
        return Err(format!("{} is damaged: its {total} bytes have CRC-32 {:08x}, and the zip records {} bytes with CRC-32 {:08x}", e.full_name, crc.value(), e.length, e.crc32));
    }
    Ok(())
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
        use std::io::Write;
        self.file.write_all(bytes).map_err(|e| e.to_string())?;
        self.at += bytes.len() as u64;
        Ok(())
    }

    /// The file at `source` added as `name`, stored where `store` says or where deflating would not make it smaller,
    /// with `modified` as its time; refused past what a plain zip holds.
    pub fn add(&mut self, name: &str, source: &str, modified: std::time::SystemTime, store: bool, stop: &dyn Fn() -> bool) -> Result<(), String> {
        use std::io::Write;
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
            out.extend_from_slice(&0x0403_4B50u32.to_le_bytes());
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
        d.extend_from_slice(&0x0201_4B50u32.to_le_bytes());
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
        end.extend_from_slice(&0x0605_4B50u32.to_le_bytes());
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
