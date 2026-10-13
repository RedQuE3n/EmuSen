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
        let meta = file.metadata()?;
        if meta.is_dir() {
            return Err(io::Error::from(io::ErrorKind::IsADirectory));
        }
        if meta.len() == 0 {
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

    /// Advises the system that the `len` bytes from `offset` will be read soon, so that it may read them from the
    /// medium before they are touched; the range is clipped to the file. One call to the system, which returns before
    /// the bytes arrive and may read fewer than were asked, so a caller asks in pieces (§3.9). Whether anything was
    /// advised: not for a range outside the file or of no length, nor where the system refuses. What a read of the
    /// bytes returns is the same whichever it was; only when they reach memory differs.
    pub fn advise(&self, offset: u64, len: u64) -> bool {
        let Some(map) = &self.map else { return false };
        let size = map.len() as u64;
        if offset >= size || len == 0 {
            return false;
        }
        will_need(map, offset as usize, len.min(size - offset) as usize)
    }
}

/// `madvise` with `MADV_WILLNEED` on the range, which memmap2 widens to whole pages.
#[cfg(unix)]
fn will_need(map: &memmap2::Mmap, offset: usize, len: usize) -> bool {
    map.advise_range(memmap2::Advice::WillNeed, offset, len).is_ok()
}

/// `PrefetchVirtualMemory` on the range, Windows' word for the same advice.
#[cfg(windows)]
fn will_need(map: &memmap2::Mmap, offset: usize, len: usize) -> bool {
    /// `WIN32_MEMORY_RANGE_ENTRY`.
    #[repr(C)]
    struct Range {
        address: *const std::ffi::c_void,
        bytes: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn PrefetchVirtualMemory(process: *mut std::ffi::c_void, count: usize, ranges: *const Range, flags: u32) -> i32;
    }
    let range = Range { address: map[offset..offset + len].as_ptr().cast(), bytes: len };
    // SAFETY: the one range lies inside the mapping, which is held for the call; the call writes no memory of the
    // program's and only asks the system to bring the pages in.
    unsafe { PrefetchVirtualMemory(GetCurrentProcess(), 1, &range, 0) != 0 }
}

#[cfg(not(any(unix, windows)))]
fn will_need(_map: &memmap2::Mmap, _offset: usize, _len: usize) -> bool {
    false
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
        assert_eq!(Mapped::open(temp.path()).err().map(|e| e.kind()), Some(std::io::ErrorKind::IsADirectory), "a directory is no file to map");
    }

    #[test]
    fn advice_is_taken_inside_the_file_and_changes_no_byte() {
        let temp = TempDir::new("mapped-advise");
        let (file, empty) = (temp.join("track.bin"), temp.join("empty.bin"));
        let bytes: Vec<u8> = (0..700_000u32).map(|i| (i * 13 + i / 97) as u8).collect();
        std::fs::write(&file, &bytes).unwrap();
        std::fs::write(&empty, b"").unwrap();
        let mapped = Mapped::open(&file).unwrap();
        let size = bytes.len() as u64;
        if cfg!(any(unix, windows)) {
            assert!(mapped.advise(0, 4096) && mapped.advise(300_001, 131_072) && mapped.advise(size - 1, 1), "a range inside the file");
            assert!(mapped.advise(size - 10, 1 << 30), "a range running past the end is clipped to it");
        }
        assert!(!mapped.advise(size, 4096) && !mapped.advise(u64::MAX, 1) && !mapped.advise(100, 0), "nothing to advise");
        assert!(!Mapped::open(&empty).unwrap().advise(0, 4096), "an empty file has no bytes to advise");
        assert_eq!(mapped.bytes(), &bytes[..]);
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
