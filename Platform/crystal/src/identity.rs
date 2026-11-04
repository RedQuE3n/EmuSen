//! A session's game identity, its MD5 and size, as a saved state records them: hashed on a thread from the moment the
//! session opens, or on the spot by a save that comes before that thread has finished, so that no state of a game is
//! written without its game's identity. A patched package's identity, set at open, is never overwritten by the file's
//! (EmuSen_GamePackages.md §5.5). See EmuSen_Backend.md §14.15.

use std::sync::{Arc, Mutex};

/// The MD5 and size of a file, the MD5 none where it cannot be read.
pub fn of_file(at: &str) -> (Option<String>, i64) {
    let bytes = std::fs::metadata(at).map_or(0, |m| m.len() as i64);
    (emusen_galaxia::rom_hash::md5(at).ok(), bytes)
}

#[derive(Clone, Default)]
pub struct Identity(Arc<Mutex<(Option<String>, i64)>>);

impl Identity {
    /// Hashing `game` begun on a thread of its own; none for a console opened without a game.
    pub fn start(game: Option<String>) -> Identity {
        Identity::start_with(game, of_file)
    }

    /// As [`Identity::start`], with the hashing given, so that a test can hold it.
    pub fn start_with(game: Option<String>, hash: impl FnOnce(&str) -> (Option<String>, i64) + Send + 'static) -> Identity {
        let identity = Identity::default();
        if let Some(at) = game {
            let theirs = identity.clone();
            let _ = std::thread::Builder::new().name("crystal-identify".to_string()).spawn(move || {
                let found = hash(&at);
                let mut known = theirs.lock();
                if known.0.is_none() {
                    *known = found;
                }
            });
        }
        identity
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, (Option<String>, i64)> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// The identity a package names, which stands over the file's.
    pub fn set(&self, md5: String, bytes: i64) {
        *self.lock() = (Some(md5), bytes);
    }

    /// The identity a state of `game` records: the one known, else the file hashed now, which the thread then keeps.
    pub fn for_state(&self, game: Option<&str>) -> (Option<String>, i64) {
        let mut known = self.lock();
        if known.0.is_none() {
            if let Some(at) = game {
                *known = of_file(at);
            }
        }
        known.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_saved_while_the_hashing_thread_is_held_records_the_game_s_md5() {
        let dir = std::env::temp_dir().join(format!("emusen-crystal-identity-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let game = dir.join("plain.tst").to_string_lossy().into_owned();
        std::fs::write(&game, b"0123456789").unwrap();
        let (release, held) = std::sync::mpsc::channel::<()>();
        let identity = Identity::start_with(Some(game.clone()), move |at| {
            let _ = held.recv();
            of_file(at)
        });
        let (md5, bytes) = identity.for_state(Some(&game));
        assert_eq!((md5.as_deref(), bytes), (Some("781e5e245d69b566979b86e28d23f2c7"), 10), "a save before the thread ends hashes the game itself");
        let _ = release.send(());
        assert_eq!(identity.for_state(None), (Some("781e5e245d69b566979b86e28d23f2c7".to_string()), 10));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_package_s_identity_stands_over_the_file_s() {
        let identity = Identity::start(None);
        identity.set("patched".into(), 4);
        assert_eq!(identity.for_state(Some("/no/such/file")), (Some("patched".to_string()), 4));
    }
}
