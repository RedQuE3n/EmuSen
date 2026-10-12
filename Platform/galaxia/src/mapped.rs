//! A file mapped read-only and held: its bytes where the system keeps them, read from the medium as they are first
//! touched, never all at once. What a host lends a core for a machine's life (the core ABI's `BORROW`) is one of these
//! for the image and each of its tracks. See EmuSen_Galaxia.md §3.8 and EmuSen_CoreAPI.md §6.24.

use std::fs::File;
use std::io;
use std::path::Path;

/// One file's bytes, mapped. They stay readable and in place until this is dropped; what keeps them unchanged is that
/// nothing of the program writes a file it has mapped (§3.8), which the system does not enforce everywhere.
pub struct Mapped {
    /// `None` for an empty file, which has nothing to map.
    map: Option<memmap2::Mmap>,
}

impl Mapped {
    /// Maps the file at `path` whole. Opening reads none of it.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Mapped> {
        let file = File::open(path)?;
        if file.metadata()?.len() == 0 {
            return Ok(Mapped { map: None });
        }
        // SAFETY: the mapping is read-only and private to this value; the file's bytes are the caller's promise not to
        // change, as memmap2 asks: the program never writes a file it maps, and says so of other programs in §3.8.
        let map = unsafe { memmap2::Mmap::map(&file) }?;
        Ok(Mapped { map: Some(map) })
    }

    pub fn bytes(&self) -> &[u8] {
        self.map.as_deref().unwrap_or(&[])
    }
}

impl AsRef<[u8]> for Mapped {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::Mapped;
    use crate::test_support::TempDir;

    #[test]
    fn a_mapped_file_is_its_bytes_and_stays_so_while_held() {
        let temp = TempDir::new("mapped");
        let file = temp.join("track.bin");
        let bytes: Vec<u8> = (0..300_000u32).map(|i| (i * 7 + i / 251) as u8).collect();
        std::fs::write(&file, &bytes).unwrap();
        let mapped = Mapped::open(&file).unwrap();
        assert_eq!(mapped.bytes(), &bytes[..]);
        assert_eq!((mapped.as_ref().len(), mapped.bytes().as_ptr()), (bytes.len(), Mapped::as_ref(&mapped).as_ptr()), "one address for its life");
        let again = Mapped::open(&file).unwrap();
        assert_eq!(again.bytes(), mapped.bytes());
        drop(again);
        assert_eq!(mapped.bytes()[299_999], bytes[299_999]);
    }

    #[test]
    fn an_empty_file_maps_to_no_bytes_and_a_missing_one_is_the_systems_error() {
        let temp = TempDir::new("mapped-empty");
        let file = temp.join("empty.bin");
        std::fs::write(&file, b"").unwrap();
        assert!(Mapped::open(&file).unwrap().bytes().is_empty());
        assert_eq!(Mapped::open(temp.join("absent.bin")).err().map(|e| e.kind()), Some(std::io::ErrorKind::NotFound));
        assert!(Mapped::open(temp.join("")).is_err(), "a directory is no file to map");
    }

    /// Unlinking a mapped file leaves its bytes readable where the system keeps a file's blocks past its name.
    #[cfg(unix)]
    #[test]
    fn a_mapped_file_outlives_its_name() {
        let temp = TempDir::new("mapped-unlinked");
        let file = temp.join("track.bin");
        std::fs::write(&file, vec![0x5Au8; 70_000]).unwrap();
        let mapped = Mapped::open(&file).unwrap();
        std::fs::remove_file(&file).unwrap();
        assert!(mapped.bytes().iter().all(|&b| b == 0x5A));
    }
}
