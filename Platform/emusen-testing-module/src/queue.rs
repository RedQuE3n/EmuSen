//! The queue: a runner's slots per class (two CPU and one GPU on the desktop) as lock files under
//! `~/.cache/emusen/testing/queue/`, held with flock(2), so a trial's steps and an agent's heavy job wait for the same
//! slots. `queue ask` prints the command prefix that holds a slot for as long as the job runs. When 8bb-lease arrives,
//! `acquire` is the one function that changes. See EmuSen_Testing.md §3.8 and `man queue`.

use crate::repo::testing_home;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

pub fn dir() -> PathBuf {
    testing_home().join("queue")
}

pub fn slot_path(class: &str, i: usize) -> PathBuf {
    dir().join(format!("{class}.{i}.lock"))
}

/// A slot held until dropped.
pub struct Slot {
    file: File,
    pub path: PathBuf,
}

impl Drop for Slot {
    fn drop(&mut self) {
        let _ = std::fs::write(self.path.with_extension("who"), "");
        #[cfg(unix)]
        unsafe {
            use std::os::fd::AsRawFd;
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn try_lock(path: &PathBuf) -> Option<File> {
    let file = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(path).ok()?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return None;
        }
    }
    Some(file)
}

/// Waits for a free slot of `class` among `count`, giving up when `stop` says so.
pub fn acquire(class: &str, count: usize, who: &str, stop: &dyn Fn() -> bool) -> Option<Slot> {
    let _ = std::fs::create_dir_all(dir());
    loop {
        for i in 0..count.max(1) {
            let path = slot_path(class, i);
            if let Some(file) = try_lock(&path) {
                let _ = std::fs::write(path.with_extension("who"), format!("{who} since {}\n", crate::store::now()));
                return Some(Slot { file, path });
            }
        }
        if stop() {
            return None;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// One line per slot: class, slot, held or free, and who holds it as the holder wrote it.
pub fn list(classes: &[(String, usize)]) -> Vec<String> {
    let mut out = Vec::new();
    for (class, count) in classes {
        for i in 0..*count {
            let path = slot_path(class, i);
            let free = try_lock(&path).is_some();
            let who = std::fs::read_to_string(path.with_extension("who")).unwrap_or_default();
            out.push(format!("{class}\t{i}\t{}\t{}", if free { "free" } else { "held" }, if free { "-" } else { who.trim() }));
        }
    }
    out
}

/// The prefix that runs a command in a capped scope while holding a slot of `class`: `flock` waits if the slot was
/// taken between the ask and the run.
pub fn prefix(slot: &std::path::Path, gb: u32) -> String {
    format!("flock {} systemd-run --user --scope -p CPUWeight=20 -p MemoryMax={gb}G -p MemorySwapMax=2G -q", slot.display())
}

/// Waits until a slot of `class` is free and gives its path, leaving it free for the caller's `flock` to take.
pub fn ask(class: &str, count: usize, who: &str, stop: &dyn Fn() -> bool) -> Option<PathBuf> {
    let slot = acquire(class, count, who, stop)?;
    let path = slot.path.clone();
    drop(slot);
    let _ = std::fs::write(path.with_extension("who"), format!("{who} since {}\n", crate::store::now()));
    Some(path)
}
