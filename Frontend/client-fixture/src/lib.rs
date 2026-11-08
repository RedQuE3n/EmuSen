//! The client crates' test world (EmuSen_Qt.md §1.1.3): the platform library and the crystal test core built by cargo
//! from this commit's `Platform/`, a scratch tree with Galaxia's config and data redirected into it, and the test core's
//! images. Nothing here names a runtime crate: the library is reached as the front end reaches it, by its path.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// The profile the library is built in: the land gate's for the Platform workspace, so its build is shared.
pub const PROFILE: &str = "test-fast";

/// The test core's system, as its info declares it.
pub const TEST_SYSTEM: &str = "test";

/// The test core's extension.
pub const TEST_EXTENSION: &str = ".tst";

/// The Platform workspace of this commit.
pub fn platform_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Platform")
}

struct Built {
    library: PathBuf,
    test_core: PathBuf,
}

/// The library and the test core, built once a process; `EMUSEN_PLATFORM_LIBRARY` and `EMUSEN_TEST_CORE` name prebuilt ones.
fn built() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| {
        if let (Ok(library), Ok(test_core)) = (std::env::var("EMUSEN_PLATFORM_LIBRARY"), std::env::var("EMUSEN_TEST_CORE")) {
            return Built { library: library.into(), test_core: test_core.into() };
        }
        let dir = platform_dir();
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = Command::new(cargo)
            .current_dir(&dir)
            .args([
                "build",
                "--quiet",
                "--profile",
                PROFILE,
                "-p",
                "emusen-platform",
                "--lib",
                "-p",
                "emusen-crystal",
                "--example",
                "crystal_test_core",
            ])
            .env_remove("CARGO_TARGET_DIR")
            .status()
            .expect("cargo runs");
        assert!(status.success(), "the platform library did not build: {status}");
        let out = dir.join("target").join(PROFILE);
        Built { library: out.join("libemusen_platform.so"), test_core: out.join("examples").join("libcrystal_test_core.so") }
    })
}

/// libemusen_platform.so as built from this commit.
pub fn library() -> &'static Path {
    &built().library
}

/// The crystal test core's library as built from this commit.
pub fn test_core() -> &'static Path {
    &built().test_core
}

/// A test's scratch tree, with Galaxia's config and data redirected into it while the guard is held.
pub struct World {
    pub root: PathBuf,
    _held: MutexGuard<'static, ()>,
}

impl World {
    /// A fresh tree named for the test; one world at a time in a process, since Galaxia's redirects are the process's.
    pub fn new(name: &str) -> World {
        static ONE: Mutex<()> = Mutex::new(());
        let held = ONE.lock().unwrap_or_else(|p| p.into_inner());
        let root = std::env::temp_dir().join(format!("emusen-client-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        redirect(&root);
        World { root, _held: held }
    }

    /// A directory under the root, made.
    pub fn dir(&self, name: &str) -> PathBuf {
        let d = self.root.join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// A game file for the test core holding `payload`.
    pub fn game(&self, name: &str, payload: &[u8]) -> PathBuf {
        let at = self.root.join(name);
        std::fs::write(&at, image(payload)).unwrap();
        at
    }

    /// The test core copied into `dir` as a player's core, unread and unapproved until the runtime approves its hash.
    pub fn player_core(&self, dir: &Path) -> PathBuf {
        let at = dir.join("libcrystal_test_core.so");
        std::fs::copy(test_core(), &at).unwrap();
        at
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// The test core's image: its magic, a little-endian u16 payload length, and the payload.
pub fn image(payload: &[u8]) -> Vec<u8> {
    let mut v = b"V1TC".to_vec();
    v.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    v.extend_from_slice(payload);
    v
}

/// Galaxia's config and data redirects, EMUSEN_GALAXIA_OVERRIDE_CONFIG (0) and _DATA (1), moved under `root`.
fn redirect(root: &Path) {
    type SetOverride = unsafe extern "C" fn(u32, *const u8, usize) -> i32;
    let lib = unsafe { libloading::Library::new(library()) }.expect("the platform library loads");
    let set: libloading::Symbol<SetOverride> = unsafe { lib.get(b"emusen_galaxia_set_override\0") }.expect("emusen_galaxia_set_override");
    for (which, sub) in [(0u32, "Config"), (1u32, "Home")] {
        let d = root.join(sub);
        let text = d.to_str().unwrap();
        assert_eq!(unsafe { set(which, text.as_ptr(), text.len()) }, 0, "redirect {which}");
    }
    // The library stays loaded for the process, as the clients load it again by the same path.
    std::mem::forget(lib);
}
