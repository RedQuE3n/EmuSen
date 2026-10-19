//! A ROM file's identity across a rename: the MD5 of every byte, header included, and with it the CRC-32 that zip and
//! ScreenScraper compute, of the whole file or a byte range, in one read. See EmuSen_Galaxia.md §5.4,
//! EmuSen_GamePackages.md §2.2 and EmuSen_Backend.md §28.2.

use md5::{Digest, Md5};
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

/// A dump's hashes, taken in one read: MD5 and CRC-32 as lower-case hexadecimal, and the bytes read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hashes {
    pub md5: String,
    pub crc32: String,
    pub size: u64,
}

/// The hashes of a file, or of `length` bytes from `offset` where a range is given (fewer where the file ends).
pub fn hashes(path: &str, range: Option<(u64, u64)>) -> std::io::Result<Hashes> {
    let mut file = std::fs::File::open(path)?;
    let mut left = u64::MAX;
    if let Some((offset, length)) = range {
        file.seek(SeekFrom::Start(offset))?;
        left = length;
    }
    let (mut md5, mut crc) = (Md5::new(), Crc32::default());
    let mut buffer = vec![0u8; 1 << 16];
    let mut size = 0u64;
    while left > 0 {
        let want = (buffer.len() as u64).min(left) as usize;
        let read = file.read(&mut buffer[..want])?;
        if read == 0 {
            break;
        }
        md5.update(&buffer[..read]);
        crc.append(&buffer[..read]);
        size += read as u64;
        left -= read as u64;
    }
    Ok(Hashes { md5: md5.finalize().iter().map(|byte| format!("{byte:02x}")).collect(), crc32: crc.hex(), size })
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
    }
}
