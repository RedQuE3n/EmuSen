//! `Mapped::advise` on a file of the build's own disk: the bytes advised reach memory without being touched, and
//! those not advised do not (EmuSen_Galaxia.md §3.9). Linux only, where the page cache can be asked and emptied.

#![cfg(target_os = "linux")]

use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

use emusen_galaxia::mapped::Mapped;

const PAGE: usize = 4096;

/// How many of the pages of `len` bytes from `offset` of the mapping are in memory.
fn resident(mapped: &Mapped, offset: usize, len: usize) -> usize {
    let mut pages = vec![0u8; len.div_ceil(PAGE)];
    // SAFETY: the range lies inside the mapping, and `pages` holds a byte for each of its pages.
    let r = unsafe { libc::mincore(mapped.bytes()[offset..].as_ptr() as *mut libc::c_void, len, pages.as_mut_ptr()) };
    assert_eq!(r, 0, "{}", std::io::Error::last_os_error());
    pages.iter().filter(|p| **p & 1 != 0).count()
}

#[test]
fn advised_bytes_reach_memory_untouched_and_the_rest_stay_on_the_medium() {
    const MIB: usize = 1 << 20;
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("advise-{}.bin", std::process::id()));
    let bytes: Vec<u8> = (0..16 * MIB as u32).map(|i| (i ^ i >> 11) as u8).collect();
    std::fs::write(&path, &bytes).unwrap();
    let file = std::fs::File::open(&path).unwrap();
    file.sync_all().unwrap();
    // SAFETY: a descriptor this test holds; the call drops the file's clean pages from the page cache.
    assert_eq!(unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED) }, 0);
    let mapped = Mapped::open(&path).unwrap();
    let (at, len) = (5 * MIB + 1234, 128 * 1024);
    if resident(&mapped, 0, 16 * MIB) != 0 {
        // A file system that keeps every page in memory has nothing to bring in; the advice is still taken.
        eprintln!("skipped: the pages of {} stay in memory, so their arrival cannot be shown", path.display());
        assert!(mapped.advise(at as u64, len as u64));
        std::fs::remove_file(&path).unwrap();
        return;
    }
    assert!(mapped.advise(at as u64, len as u64));
    let (first, pages) = (at / PAGE * PAGE, (at + len).div_ceil(PAGE) - at / PAGE);
    let started = Instant::now();
    while resident(&mapped, first, pages * PAGE) < pages {
        assert!(started.elapsed() < Duration::from_secs(10), "{} of {pages} advised pages arrived", resident(&mapped, first, pages * PAGE));
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(resident(&mapped, 12 * MIB, MIB), 0, "a megabyte nothing advised is still on the medium");
    assert_eq!(&mapped.bytes()[at..at + len], &bytes[at..at + len]);
    drop(mapped);
    std::fs::remove_file(&path).unwrap();
}
