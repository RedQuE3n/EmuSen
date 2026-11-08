//! A big file identified without reading it whole: its size and a SHA-1 of its first and last MiB key its record, and
//! its full MD5, CRC32 and SHA-1 follow later on a thread of the lowest priority that runs only while no game is on
//! screen and gives way the moment one starts. See EmuSen_LibraryTools.md §9.12.

use crate::Scraper;
use emusen_galaxia::files::file_info;
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Files above this many bytes are fingerprinted; those of 1 GB or less are hashed in full as before.
pub const ABOVE: u64 = 1_000_000_000;

/// The bytes read at each end.
pub const EDGE: u64 = 1 << 20;

/// Whether a file of `bytes` is keyed by its fingerprint, `above` being [`ABOVE`] outside the tests.
pub fn wanted(bytes: u64, above: u64) -> bool {
    bytes > above
}

/// The fingerprint of the file at `path` of `bytes`, and how many bytes were read for it: one pass over its first and
/// last MiB, the whole file where it is smaller than both.
pub fn of(path: &str, bytes: u64) -> std::io::Result<(String, u64)> {
    let mut file = std::fs::File::open(path)?;
    let mut sha = Sha1::new();
    let mut buffer = vec![0u8; EDGE as usize];
    let mut read = 0u64;
    let mut take = |file: &mut std::fs::File, length: u64| -> std::io::Result<()> {
        let part = &mut buffer[..length as usize];
        file.read_exact(part)?;
        sha.update(&*part);
        read += length;
        Ok(())
    };
    if bytes <= 2 * EDGE {
        take(&mut file, bytes)?;
    } else {
        take(&mut file, EDGE)?;
        file.seek(SeekFrom::End(-(EDGE as i64)))?;
        take(&mut file, EDGE)?;
    }
    let digest = sha.finalize();
    Ok((digest.iter().map(|b| format!("{b:02x}")).collect(), read))
}

/// The background hashing's state: whether a game is on screen, the thread while it lives, and the handle's stop.
#[derive(Default)]
pub struct Background {
    playing: Mutex<bool>,
    changed: Condvar,
    stop: AtomicBool,
    live: AtomicBool,
    thread: Mutex<Option<JoinHandle<()>>>,
}

/// A handle's address, sent to its own thread; the handle joins it before it is freed.
#[derive(Clone, Copy)]
struct Handle(*const Scraper);
unsafe impl Send for Handle {}

impl Handle {
    fn get(&self) -> &'static Scraper {
        unsafe { &*self.0 }
    }
}

impl Scraper {
    fn playing(&self) -> bool {
        *self.background.playing.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// The host's word that a game is on screen or no longer is; the owed checksums resume when none is.
    pub fn set_playing(&self, playing: bool) {
        *self.background.playing.lock().unwrap_or_else(|p| p.into_inner()) = playing;
        self.background.changed.notify_all();
        if !playing {
            self.hash_later();
        }
    }

    /// Starts the background hashing where checksums are owed and none is running.
    pub(crate) fn hash_later(&self) {
        let b = &self.background;
        if b.stop.load(Ordering::SeqCst) || self.store().unhashed().map_or(true, |u| u.is_empty()) || b.live.swap(true, Ordering::SeqCst) {
            return;
        }
        let me = Handle(self as *const Scraper);
        let mut thread = b.thread.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(old) = thread.take() {
            let _ = old.join();
        }
        match std::thread::Builder::new().name("scrape checksums".into()).spawn(move || me.get().hash_owed()) {
            Ok(h) => *thread = Some(h),
            Err(_) => b.live.store(false, Ordering::SeqCst),
        }
    }

    /// Waits while a game is on screen; false once the handle is closing.
    fn wait_for_no_game(&self) -> bool {
        let b = &self.background;
        let mut playing = b.playing.lock().unwrap_or_else(|p| p.into_inner());
        while *playing && !b.stop.load(Ordering::SeqCst) {
            playing = b.changed.wait_timeout(playing, Duration::from_millis(200)).unwrap_or_else(|p| p.into_inner()).0;
        }
        !b.stop.load(Ordering::SeqCst)
    }

    fn hash_owed(&self) {
        lowest_priority();
        let b = &self.background;
        let mut passed: HashSet<String> = HashSet::new();
        loop {
            if !self.wait_for_no_game() {
                break;
            }
            let next = self.store().unhashed().unwrap_or_default().into_iter().find(|(p, ..)| !passed.contains(p));
            let Some((path, bytes, modified, fingerprint)) = next else {
                b.live.store(false, Ordering::SeqCst);
                let more = self.store().unhashed().is_ok_and(|u| u.iter().any(|(p, ..)| !passed.contains(p)));
                if more && !b.stop.load(Ordering::SeqCst) && !b.live.swap(true, Ordering::SeqCst) {
                    continue;
                }
                return;
            };
            if file_info(&path) != Some((bytes, modified)) {
                passed.insert(path);
                continue;
            }
            match emusen_galaxia::rom_hash::hashes_with(&path, None, true, &|| b.stop.load(Ordering::SeqCst) || self.playing()) {
                Ok(h) => {
                    let _ = self.store().fill_hashes(&path, bytes, modified, &fingerprint, &h.md5, &h.crc32, &h.sha1.unwrap_or_default());
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    passed.insert(path);
                }
            }
        }
        b.live.store(false, Ordering::SeqCst);
    }

    /// Stops the background hashing and joins it, for the handle's close.
    pub(crate) fn stop_background(&self) {
        self.background.stop.store(true, Ordering::SeqCst);
        self.background.changed.notify_all();
        if let Some(h) = self.background.thread.lock().unwrap_or_else(|p| p.into_inner()).take() {
            let _ = h.join();
        }
    }
}

/// The calling thread at the system's lowest CPU and I/O priority.
#[cfg(target_os = "linux")]
fn lowest_priority() {
    unsafe {
        let tid = libc::syscall(libc::SYS_gettid) as libc::id_t;
        libc::setpriority(libc::PRIO_PROCESS as _, tid, 19);
        libc::syscall(libc::SYS_ioprio_set, 1, tid, 3 << 13);
    }
}

#[cfg(target_vendor = "apple")]
fn lowest_priority() {
    unsafe {
        libc::setpriority(libc::PRIO_DARWIN_THREAD, 0, libc::PRIO_DARWIN_BG);
    }
}

#[cfg(windows)]
fn lowest_priority() {
    unsafe extern "system" {
        fn GetCurrentThread() -> *mut core::ffi::c_void;
        fn SetThreadPriority(thread: *mut core::ffi::c_void, priority: i32) -> i32;
    }
    const THREAD_MODE_BACKGROUND_BEGIN: i32 = 0x0001_0000;
    unsafe {
        SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN);
    }
}

#[cfg(not(any(target_os = "linux", target_vendor = "apple", windows)))]
fn lowest_priority() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_above_one_gb_is_fingerprinted_and_one_of_one_gb_is_not() {
        assert!(!wanted(1_000_000_000, ABOVE));
        assert!(wanted(1_000_000_001, ABOVE));
    }

    #[test]
    fn the_fingerprint_reads_two_mib_and_sees_both_ends_only() {
        let dir = std::env::var("TMPDIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join(format!("emusen-fingerprint-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("big.iso");
        let size = 64u64 << 20;
        let write = |at: u64, byte: u8| {
            let mut f = std::fs::OpenOptions::new().create(true).write(true).truncate(false).open(&file).unwrap();
            f.set_len(size).unwrap();
            f.seek(SeekFrom::Start(at)).unwrap();
            std::io::Write::write_all(&mut f, &[byte]).unwrap();
        };
        let path = file.to_string_lossy().into_owned();
        write(0, 1);
        let (first, read) = of(&path, size).unwrap();
        assert_eq!(read, 2 * EDGE, "two MiB read, not the 64");
        write(size / 2, 9);
        assert_eq!(of(&path, size).unwrap().0, first, "the middle is not read");
        write(size - 1, 2);
        assert_ne!(of(&path, size).unwrap().0, first, "the last MiB is");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
