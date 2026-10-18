//! An advisory lock on a save medium's file, the C# `MediumLock` (EmuSen_SaveMedia.md §3.5): `<file>.lock` opened and
//! locked exclusively while held and removed when let go, holding the holder's words and the process's id, so two
//! programs never write one card. The lock is the system's `flock`, which is what .NET's `FileShare.None` takes on
//! Unix, so a lock taken by either half, or by another program of either half, refuses the other. Within the process a
//! table answers who holds a file in the holder's own words; shared handles are the program's machines, one run at a time.

use crate::dotnet_path::{self as path, Style};
use crate::tree::file_exists;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

pub const SUFFIX: &str = ".lock";

struct Taken {
    file: File,
    shared: bool,
    handles: Vec<(u64, String)>,
}

static HELD: Mutex<Option<HashMap<String, Taken>>> = Mutex::new(None);
static NEXT: AtomicU64 = AtomicU64::new(1);

/// A held lock; letting it go releases this handle, and the file's lock with the last one.
#[derive(Debug)]
pub struct MediumLock {
    pub id: u64,
    pub path: String,
    pub holder: String,
}

fn table<R>(f: impl FnOnce(&mut HashMap<String, Taken>) -> R) -> R {
    let mut held = HELD.lock().unwrap_or_else(|p| p.into_inner());
    f(held.get_or_insert_with(HashMap::new))
}

/// The lock file opened and locked, or none when another holds it; the folder is made where it is missing.
fn open(key: &str) -> Option<File> {
    if let Some(folder) = path::directory_name(Style::HOST, key).filter(|f| !f.is_empty()) {
        std::fs::create_dir_all(folder).ok()?;
    }
    let file = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(format!("{key}{SUFFIX}")).ok()?;
    file.try_lock().ok()?;
    Some(file)
}

/// `MediumLock.TryTake`: the lock on `file` for `holder`, or none where another program holds it or this one holds it unshared.
pub fn try_take(file: &str, holder: &str, shared: bool) -> Option<MediumLock> {
    let key = path::full_path(file);
    table(|held| {
        if let Some(taken) = held.get_mut(&key) {
            if !shared || !taken.shared {
                return None;
            }
            return Some(add(taken, &key, holder));
        }
        let mut lock = open(&key)?;
        let words = format!("{holder}\n{}\n", std::process::id());
        if lock.set_len(0).is_err() || lock.write_all(words.as_bytes()).is_err() || lock.flush().is_err() {
            release_file(&key, lock);
            return None;
        }
        let mut taken = Taken { file: lock, shared, handles: Vec::new() };
        let handle = add(&mut taken, &key, holder);
        held.insert(key, taken);
        Some(handle)
    })
}

fn add(taken: &mut Taken, key: &str, holder: &str) -> MediumLock {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    taken.handles.push((id, holder.to_string()));
    MediumLock { id, path: key.to_string(), holder: holder.to_string() }
}

/// The file removed, then its lock let go as it closes, as .NET's `DeleteOnClose` does.
fn release_file(key: &str, file: File) {
    let _ = std::fs::remove_file(format!("{key}{SUFFIX}"));
    drop(file);
}

/// `MediumLock.HolderOf`: the latest holder's words in this process, "another program" elsewhere, none when no one holds it.
pub fn holder_of(file: &str) -> Option<String> {
    let key = path::full_path(file);
    table(|held| {
        if let Some(taken) = held.get(&key) {
            return taken.handles.last().map(|(_, holder)| holder.clone());
        }
        let lock = format!("{key}{SUFFIX}");
        if !file_exists(&lock) {
            return None;
        }
        let probe = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&lock);
        match probe {
            Ok(f) if f.try_lock().is_ok() => None,
            _ => Some("another program".to_string()),
        }
    })
}

/// Lets go of the handle with this id; false when it was let go already or never given.
pub fn release(id: u64) -> bool {
    table(|held| {
        let Some(key) = held.iter().find(|(_, t)| t.handles.iter().any(|(h, _)| *h == id)).map(|(k, _)| k.clone()) else {
            return false;
        };
        let taken = held.get_mut(&key).expect("found above");
        taken.handles.retain(|(h, _)| *h != id);
        if taken.handles.is_empty() {
            let taken = held.remove(&key).expect("found above");
            release_file(&key, taken.file);
        }
        true
    })
}

impl Drop for MediumLock {
    fn drop(&mut self) {
        release(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn one_holder_at_a_time_unless_both_share_and_the_file_goes_with_the_last() {
        let dir = TempDir::new("medium-lock");
        let card = dir.join("Cards/a.mcr");
        let game = try_take(&card, "Game A", false).unwrap();
        assert!(file_exists(&format!("{card}{SUFFIX}")));
        assert_eq!(std::fs::read_to_string(format!("{card}{SUFFIX}")).unwrap(), format!("Game A\n{}\n", std::process::id()));
        assert!(try_take(&card, "the card manager", false).is_none());
        assert_eq!(holder_of(&card).as_deref(), Some("Game A"));
        drop(game);
        assert!(!file_exists(&format!("{card}{SUFFIX}")));
        assert_eq!(holder_of(&card), None);
        let one = try_take(&card, "Game B", true).unwrap();
        let two = try_take(&card, "Game C", true).unwrap();
        assert!(try_take(&card, "Game D", false).is_none());
        assert_eq!(holder_of(&card).as_deref(), Some("Game C"));
        drop(two);
        assert_eq!(holder_of(&card).as_deref(), Some("Game B"));
        drop(one);
        assert_eq!(holder_of(&card), None);
    }

    #[test]
    fn a_file_another_open_file_has_locked_is_held_by_another_program_and_a_stale_one_is_taken() {
        let dir = TempDir::new("medium-lock-other");
        let card = dir.join("b.mcr");
        let other = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(format!("{card}{SUFFIX}")).unwrap();
        other.try_lock().unwrap();
        assert_eq!(holder_of(&card).as_deref(), Some("another program"));
        assert!(try_take(&card, "Game", false).is_none());
        drop(other);
        assert_eq!(holder_of(&card), None);
        assert!(file_exists(&format!("{card}{SUFFIX}")));
        let taken = try_take(&card, "Game", false).unwrap();
        assert!(release(taken.id));
        assert!(!release(taken.id));
        std::mem::forget(taken);
    }
}
