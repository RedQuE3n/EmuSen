//! The archives a game may be packed in, the C# `ArchiveImages`' reading: a zip's central directory, the entry play
//! opens, and an entry's first bytes inflated without unpacking. Only a plain zip of stored and deflated entries is read;
//! the central directory's reader is shared with `emusen-cheats`' installer. See EmuSen_Backend.md §28.2.

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
        let dos_time = w(12)?;
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
        list.push(ZipEntry { platform: (made_by >> 8) as u8, method, dos_time, compressed: compressed as usize, length: length as u64, external, offset: offset as usize, full_name });
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
