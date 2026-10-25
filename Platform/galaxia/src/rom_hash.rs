//! A ROM file's identity across a rename: the MD5 of every byte, header included, and with it the CRC-32 that zip and
//! ScreenScraper compute and, when asked, the SHA-1 ScreenScraper also takes, of the whole file or a byte range, in one
//! read; a console's comparable form, by a transform its system pack owns; and a disc's tracks, each where
//! `emusen-disc` places it. See EmuSen_Galaxia.md §5.4, EmuSen_GamePackages.md §2.2, EmuSen_Backend.md §28.2 and §30.4.

use md5::{Digest, Md5};
use sha1::Sha1;
use std::io::{Read, Seek, SeekFrom};

/// The file's MD5 as 32 lower-case hexadecimal digits.
pub fn md5(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

/// CRC-32 as zip and ScreenScraper compute it, the C# `Crc32`: polynomial 0xEDB88320, reflected, initial and final
/// value all ones.
pub struct Crc32 {
    value: u32,
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

static CRC_TABLE: [u32; 256] = crc_table();

impl Default for Crc32 {
    fn default() -> Self {
        Crc32 { value: 0xFFFF_FFFF }
    }
}

impl Crc32 {
    pub fn append(&mut self, bytes: &[u8]) {
        let mut value = self.value;
        for &b in bytes {
            value = CRC_TABLE[((value ^ b as u32) & 0xFF) as usize] ^ (value >> 8);
        }
        self.value = value;
    }

    pub fn value(&self) -> u32 {
        !self.value
    }

    /// Eight lower-case hexadecimal digits, as the C#'s `Hex`.
    pub fn hex(&self) -> String {
        format!("{:08x}", self.value())
    }
}

/// A dump's hashes, taken in one read: MD5, CRC-32 and, when asked for, SHA-1 as lower-case hexadecimal, and the bytes read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hashes {
    pub md5: String,
    pub crc32: String,
    pub sha1: Option<String>,
    pub size: u64,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The hashes of a file, or of `length` bytes from `offset` where a range is given (fewer where the file ends).
pub fn hashes(path: &str, range: Option<(u64, u64)>) -> std::io::Result<Hashes> {
    hashes_with(path, range, false, &|| false)
}

/// As [`hashes`], with SHA-1 too when `sha1`; `stop` is asked before each 64 KiB read, and a stop is `ErrorKind::Interrupted`.
pub fn hashes_with(path: &str, range: Option<(u64, u64)>, sha1: bool, stop: &dyn Fn() -> bool) -> std::io::Result<Hashes> {
    let mut file = std::fs::File::open(path)?;
    let mut left = u64::MAX;
    if let Some((offset, length)) = range {
        file.seek(SeekFrom::Start(offset))?;
        left = length;
    }
    let (mut md5, mut crc) = (Md5::new(), Crc32::default());
    let mut sha = sha1.then(Sha1::new);
    let mut buffer = vec![0u8; 1 << 16];
    let mut size = 0u64;
    while left > 0 {
        if stop() {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
        }
        let want = (buffer.len() as u64).min(left) as usize;
        let read = file.read(&mut buffer[..want])?;
        if read == 0 {
            break;
        }
        md5.update(&buffer[..read]);
        crc.append(&buffer[..read]);
        if let Some(sha) = sha.as_mut() {
            sha.update(&buffer[..read]);
        }
        size += read as u64;
        left -= read as u64;
    }
    Ok(Hashes { md5: hex(&md5.finalize()), crc32: crc.hex(), sha1: sha.map(|sha| hex(&sha.finalize())), size })
}

/// All three hashes of bytes held in memory, as the C# `RomHashes.Of(byte[])` takes them of a transformed image.
pub fn hashes_of(bytes: &[u8]) -> Hashes {
    let mut crc = Crc32::default();
    crc.append(bytes);
    Hashes { md5: hex(&Md5::digest(bytes)), crc32: crc.hex(), sha1: Some(hex(&Sha1::digest(bytes))), size: bytes.len() as u64 }
}

/// The hashes of the form `transform` makes of a file's bytes, or none where it makes none or the same bytes: the
/// transform is the system pack's, and this knows no console (EmuSen_GamePackages.md §2.2).
pub fn transformed(file: &[u8], transform: impl FnOnce(&[u8]) -> Option<Vec<u8>>) -> Option<Hashes> {
    transform(file).filter(|changed| changed.as_slice() != file).map(|changed| hashes_of(&changed))
}

/// One track of a sheet as dumped: its number, its file as the sheet names it, where it begins there, and its hashes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackHashes {
    pub number: u8,
    pub file: String,
    pub offset: u64,
    pub hashes: Hashes,
}

/// Every track of the cue sheet at `sheet_path`, each hashed over the bytes `emusen-disc` places it in, its files found
/// beside the sheet as a disc's are; SHA-1 always. `stop` is as [`hashes_with`]'s.
pub fn tracks(sheet_path: &str, stop: &dyn Fn() -> bool) -> std::io::Result<Vec<TrackHashes>> {
    let refuse = |words: String| std::io::Error::new(std::io::ErrorKind::InvalidData, words);
    let text = emusen_dotnet::file::decode(&std::fs::read(sheet_path)?);
    let names = emusen_disc::set::companions(&text).map_err(|e| refuse(e.to_string()))?;
    let spans = emusen_disc::set::track_spans(&text).map_err(refuse)?;
    let paths = crate::image_sets::companions(sheet_path).map_err(refuse)?;
    let mut out = Vec::with_capacity(spans.len());
    for span in spans {
        let hashes = hashes_with(&paths[span.file], Some((span.offset, span.length.unwrap_or(u64::MAX))), true, stop)?;
        out.push(TrackHashes { number: span.number, file: names[span.file].clone(), offset: span.offset, hashes });
    }
    Ok(out)
}

/// A file's SHA-1 alone, as the C# `Scraper.Sha1Of` takes it of a kept media file.
pub fn sha1(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use crate::test_support::TempDir;

    #[test]
    fn the_hash_is_md5s_own_test_vectors() {
        let temp = TempDir::new("md5");
        let file = temp.join("rom.bin");
        std::fs::write(&file, b"").unwrap();
        assert_eq!(super::md5(&file).unwrap(), "d41d8cd98f00b204e9800998ecf8427e");
        std::fs::write(&file, b"abc").unwrap();
        assert_eq!(super::md5(&file).unwrap(), "900150983cd24fb0d6963f7d28e17f72");
        std::fs::write(&file, vec![b'a'; 1_000_000]).unwrap();
        assert_eq!(super::md5(&file).unwrap(), "7707d6ae4e027c70eea2a935c2296f21");
        assert!(super::md5(&temp.join("absent.bin")).is_err());
    }

    #[test]
    fn the_crc_and_the_range_are_the_known_values() {
        let temp = TempDir::new("hashes");
        let file = temp.join("rom.bin");
        std::fs::write(&file, b"123456789").unwrap();
        let whole = super::hashes(&file, None).unwrap();
        assert_eq!((whole.crc32.as_str(), whole.size), ("cbf43926", 9));
        assert_eq!(whole.md5, super::md5(&file).unwrap());
        let part = super::hashes(&file, Some((6, 100))).unwrap();
        assert_eq!((part.md5.as_str(), part.size), ("68053af2923e00204c3ca7c6a3150cf7", 3));
        assert_eq!(whole.sha1, None);
    }

    #[test]
    fn a_transform_is_hashed_only_where_it_changes_the_bytes() {
        let file = b"HEADERbody";
        assert_eq!(super::transformed(file, |f| Some(f[6..].to_vec())), Some(super::hashes_of(b"body")));
        assert_eq!(super::transformed(file, |f| Some(f.to_vec())), None);
        assert_eq!(super::transformed(file, |_| None), None);
    }

    #[test]
    fn each_tracks_hashes_are_its_files_or_its_range_of_one() {
        let temp = TempDir::new("tracks");
        let (one, two): (Vec<u8>, Vec<u8>) = ((0..2352 * 3).map(|i| (i % 251) as u8).collect(), (0..2352 * 2).map(|i| (i % 241) as u8).collect());
        std::fs::write(temp.join("g (Track 1).bin"), &one).unwrap();
        std::fs::write(temp.join("G (TRACK 2).BIN"), &two).unwrap();
        let sheet = temp.join("g.cue");
        std::fs::write(&sheet, "FILE \"g (Track 1).bin\" BINARY\n TRACK 01 MODE2/2352\n  INDEX 01 00:00:00\nFILE \"g (Track 2).bin\" BINARY\n TRACK 02 AUDIO\n  INDEX 00 00:00:00\n  INDEX 01 00:00:01\n").unwrap();
        let tracks = super::tracks(&sheet, &|| false).unwrap();
        assert_eq!(tracks.iter().map(|t| (t.number, t.file.as_str(), t.offset)).collect::<Vec<_>>(), [(1, "g (Track 1).bin", 0), (2, "g (Track 2).bin", 0)]);
        assert_eq!((tracks[0].hashes.clone(), tracks[1].hashes.clone()), (super::hashes_of(&one), super::hashes_of(&two)));
        let joined = [one.clone(), two.clone()].concat();
        std::fs::write(temp.join("j.bin"), &joined).unwrap();
        let single = temp.join("j.cue");
        std::fs::write(&single, "FILE j.bin BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nINDEX 00 00:00:03\nINDEX 01 00:00:04\n").unwrap();
        let split = super::tracks(&single, &|| false).unwrap();
        assert_eq!((split[0].hashes.clone(), split[1].hashes.clone(), split[1].offset), (super::hashes_of(&one), super::hashes_of(&two), 3 * 2352));
        assert_eq!(super::tracks(&temp.join("absent.cue"), &|| false).unwrap_err().kind(), std::io::ErrorKind::NotFound);
        std::fs::write(&single, "TRACK 01 AUDIO\n").unwrap();
        assert_eq!(super::tracks(&single, &|| false).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
        std::fs::write(&single, "FILE gone.bin BINARY\nTRACK 01 AUDIO\nINDEX 01 00:00:00\n").unwrap();
        assert_eq!(super::tracks(&single, &|| false).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn sha1_is_its_own_test_vectors_and_a_stop_is_interrupted() {
        let temp = TempDir::new("sha1");
        let file = temp.join("rom.bin");
        std::fs::write(&file, b"abc").unwrap();
        let all = super::hashes_with(&file, None, true, &|| false).unwrap();
        assert_eq!(all.sha1.as_deref(), Some("a9993e364706816aba3e25717850c26c9cd0d89d"));
        assert_eq!(super::sha1(&file).unwrap(), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(super::hashes_of(b"abc"), all);
        assert_eq!(super::hashes_of(b"").sha1.as_deref(), Some("da39a3ee5e6b4b0d3255bfef95601890afd80709"));
        std::fs::write(&file, vec![b'a'; 1_000_000]).unwrap();
        assert_eq!(super::sha1(&file).unwrap(), "34aa973cd4c4daa4f61eeb2bdbad27316534016f");
        let stopped = super::hashes_with(&file, None, true, &|| true).unwrap_err();
        assert_eq!(stopped.kind(), std::io::ErrorKind::Interrupted);
    }
}
