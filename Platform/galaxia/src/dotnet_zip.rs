//! A zip archive read as .NET's `ZipArchive` reads one in read mode: the end record found within the last 64 KiB and
//! 22 bytes, a Zip64 end where a field of the plain one is at its maximum, every central directory entry with its Zip64
//! extra field, and the words of the `InvalidDataException` .NET throws for an archive it refuses whole. An entry's bytes
//! are read with the checks `ZipArchiveEntry.Open` makes, stored, deflated and Deflate64 alike. Measured against .NET
//! 10.0.112 by WiseMan's `CheatArchiveFormsTests`. See EmuSen_Backend.md §40.1.

use std::io::Read;

/// .NET's words for each refusal, as `System.IO.Compression`'s resources give them.
pub mod words {
    pub const END_NOT_FOUND: &str = "End of Central Directory record could not be found.";
    pub const DIRECTORY_CORRUPT: &str = "Central Directory corrupt.";
    pub const SPANNED: &str = "Split or spanned archives are not supported.";
    pub const COUNT_WRONG: &str = "Number of entries expected in End Of Central Directory does not correspond to number of entries in Central Directory.";
    pub const OFFSET_TOO_BIG: &str = "Offset to Central Directory cannot be held in an Int64.";
    pub const ZIP64_END_MISPLACED: &str = "Zip 64 End of Central Directory Record not where indicated.";
    pub const LOCAL_HEADER_CORRUPT: &str = "A local file header is corrupt.";
    pub const UNSUPPORTED_METHOD: &str = "The archive entry was compressed using an unsupported compression method.";
    pub const DATA_CORRUPT: &str = "The archive entry data is corrupt.";
}

/// One central directory entry, its Zip64 fields resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub platform: u8,
    pub flags: u16,
    pub method: u16,
    pub dos_time: u32,
    pub compressed: u64,
    pub length: u64,
    pub external: u32,
    pub offset: u64,
    pub disk: u32,
    pub full_name: String,
}

/// An archive .NET would open: the disk its end names and its entries in the directory's order.
pub struct Archive<'a> {
    bytes: &'a [u8],
    pub disk: u32,
    pub entries: Vec<Entry>,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at.checked_add(2)?).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at.checked_add(4)?).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    b.get(at..at.checked_add(8)?).map(|s| u64::from_le_bytes(s.try_into().unwrap()))
}

const END: u32 = 0x0605_4B50;
const ZIP64_END: u32 = 0x0606_4B50;
const ZIP64_LOCATOR: u32 = 0x0706_4B50;
const CENTRAL: u32 = 0x0201_4B50;
const LOCAL: u32 = 0x0403_4B50;

/// `ZipArchive`'s constructor and its `Entries`: the archive, or the words .NET throws.
pub fn open(zip: &[u8]) -> Result<Archive<'_>, &'static str> {
    let len = zip.len();
    if len < 18 {
        return Err(words::DIRECTORY_CORRUPT);
    }
    let from = len - 18;
    let lowest = from.saturating_sub(0xFFFF + 4);
    let end = (lowest..from.saturating_sub(3)).rev().find(|&at| u32_at(zip, at) == Some(END)).ok_or(words::END_NOT_FOUND)?;
    let field16 = |o: usize| u16_at(zip, end + o).ok_or(words::DIRECTORY_CORRUPT);
    let (this_disk, central_disk, on_disk, total) = (field16(4)?, field16(6)?, field16(8)?, field16(10)?);
    let start = u32_at(zip, end + 16).ok_or(words::DIRECTORY_CORRUPT)?;
    let comment = field16(20)? as usize;
    if end + 22 + comment > len {
        return Err(words::DIRECTORY_CORRUPT);
    }
    if this_disk != central_disk || on_disk != total {
        return Err(words::SPANNED);
    }
    let (mut disk, mut expected, mut central) = (this_disk as u32, total as u64, start as u64);
    if this_disk == 0xFFFF || start == u32::MAX || total == 0xFFFF {
        if end >= 20 && u32_at(zip, end - 20) == Some(ZIP64_LOCATOR) {
            let record = u64_at(zip, end - 20 + 8).ok_or(words::DIRECTORY_CORRUPT)?;
            if record > i64::MAX as u64 {
                return Err(words::ZIP64_END_MISPLACED);
            }
            let r = record as usize;
            if record > len as u64 || u32_at(zip, r) != Some(ZIP64_END) || r + 56 > len {
                return Err(words::ZIP64_END_MISPLACED);
            }
            disk = u32_at(zip, r + 16).ok_or(words::DIRECTORY_CORRUPT)?;
            let (here, all) = (u64_at(zip, r + 24).unwrap_or(0), u64_at(zip, r + 32).unwrap_or(0));
            let offset = u64_at(zip, r + 48).unwrap_or(0);
            if all > i64::MAX as u64 || offset > i64::MAX as u64 {
                return Err(words::OFFSET_TOO_BIG);
            }
            if all != here {
                return Err(words::SPANNED);
            }
            expected = all;
            central = offset;
        }
    }
    if central > len as u64 {
        return Err(words::OFFSET_TOO_BIG);
    }
    let mut entries = Vec::new();
    let mut at = central as usize;
    while let Some(e) = central_entry(zip, &mut at) {
        entries.push(e);
    }
    if entries.len() as u64 != expected {
        return Err(words::COUNT_WRONG);
    }
    Ok(Archive { bytes: zip, disk, entries })
}

/// `ZipCentralDirectoryFileHeader.TryReadBlock`: an entry and the place after it, or none where the directory ends.
fn central_entry(zip: &[u8], at: &mut usize) -> Option<Entry> {
    let h = *at;
    if u32_at(zip, h)? != CENTRAL || h + 46 > zip.len() {
        return None;
    }
    let (made_by, flags, method, dos_time) = (u16_at(zip, h + 4)?, u16_at(zip, h + 8)?, u16_at(zip, h + 10)?, u32_at(zip, h + 12)?);
    let (mut compressed, mut length) = (u32_at(zip, h + 20)? as u64, u32_at(zip, h + 24)? as u64);
    let (n, m, k) = (u16_at(zip, h + 28)? as usize, u16_at(zip, h + 30)? as usize, u16_at(zip, h + 32)? as usize);
    let mut disk = u16_at(zip, h + 34)? as u32;
    let external = u32_at(zip, h + 38)?;
    let mut offset = u32_at(zip, h + 42)? as u64;
    let name = zip.get(h + 46..h + 46 + n)?;
    let extra = zip.get(h + 46 + n..h + 46 + n + m)?;
    zip.get(h + 46 + n + m..h + 46 + n + m + k)?;
    let (want_length, want_compressed, want_offset, want_disk) = (length == u32::MAX as u64, compressed == u32::MAX as u64, offset == u32::MAX as u64, disk == 0xFFFF);
    let mut x = 0;
    while x + 4 <= extra.len() {
        let (tag, size) = (u16_at(extra, x)?, u16_at(extra, x + 2)? as usize);
        let body = &extra[x + 4..(x + 4 + size).min(extra.len())];
        if tag == 1 {
            let mut p = 0;
            let mut take = |want: bool, wide: bool| -> Option<u64> {
                if !want {
                    return None;
                }
                let v = if wide { u64_at(body, p) } else { u32_at(body, p).map(u64::from) };
                p += if wide { 8 } else { 4 };
                v
            };
            if let Some(v) = take(want_length, true) {
                length = v;
            }
            if let Some(v) = take(want_compressed, true) {
                compressed = v;
            }
            if let Some(v) = take(want_offset, true) {
                offset = v;
            }
            if let Some(v) = take(want_disk, false) {
                disk = v as u32;
            }
            break;
        }
        x += 4 + size;
    }
    *at = h + 46 + n + m + k;
    Some(Entry { platform: (made_by >> 8) as u8, flags, method, dos_time, compressed, length, external, offset, disk, full_name: String::from_utf8_lossy(name).into_owned() })
}

impl Archive<'_> {
    /// `ZipArchiveEntry.Open` read to its end: the entry's bytes, or .NET's words for why it cannot be opened or read.
    pub fn read(&self, e: &Entry) -> Result<Vec<u8>, &'static str> {
        let len = self.bytes.len() as u64;
        if e.disk != self.disk {
            return Err(words::SPANNED);
        }
        if e.offset > len {
            return Err(words::LOCAL_HEADER_CORRUPT);
        }
        let h = e.offset as usize;
        if u32_at(self.bytes, h) != Some(LOCAL) || h + 30 > self.bytes.len() {
            return Err(words::LOCAL_HEADER_CORRUPT);
        }
        let data = h as u64 + 30 + u16_at(self.bytes, h + 26).unwrap_or(0) as u64 + u16_at(self.bytes, h + 28).unwrap_or(0) as u64;
        if data > len || data.checked_add(e.compressed).is_none_or(|stop| stop > len) {
            return Err(words::LOCAL_HEADER_CORRUPT);
        }
        let raw = &self.bytes[data as usize..(data + e.compressed) as usize];
        let mut out = Vec::with_capacity(e.length.min(1 << 24) as usize);
        match e.method {
            0 => out.extend_from_slice(raw),
            8 => {
                flate2::read::DeflateDecoder::new(raw).read_to_end(&mut out).map_err(|_| words::DATA_CORRUPT)?;
            }
            9 => {
                deflate64::Deflate64Decoder::with_buffer(raw).read_to_end(&mut out).map_err(|_| words::DATA_CORRUPT)?;
            }
            _ => return Err(words::UNSUPPORTED_METHOD),
        }
        Ok(out)
    }
}
