//! The cores that draw on the device, built from this commit's `EmuSen/Cores` as the C# build builds them (each crate
//! into `EmuSen/obj/<name>`, release, so both share one cache): emusen-native's device test core, which hands its pictures
//! through a lend, and a libretro entry (the proxy, the libretro host, the libretro test core), which draws in a ring
//! (EmuSen_Qt.md §1.8.7).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The libretro test core's game: its magic and a name, the extension its core information gives.
pub const RING_GAME: &[u8] = b"TCG1 a test game";

/// The device test core's payload byte that paints the libretro test core's picture on the lent device.
pub const PAINTED: u8 = 0xD1;

/// The painted picture's size.
pub const PAINTED_SIZE: (u32, u32) = (320, 240);

/// The painted picture's pixel at (x, y) for the frame whose serial is `serial`, with no button held: the libretro test
/// core's picture, its lives falling every 60 frames from 3 (`emusen-native`'s common test core, `paint`).
pub fn painted_pixel(serial: u64, x: u32, y: u32) -> [u8; 4] {
    let lives = 3 - ((serial / 60) % 4) as u32;
    [(u64::from(x) + serial) as u8, (y * 2 + lives * 40) as u8, (x >> 3) as u8, 0xFF]
}

fn cores_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../EmuSen/Cores")
}

/// `cargo build --release <args>` in a crate under `EmuSen/Cores`, into `EmuSen/obj/<target>`; that directory's `release`.
fn build(krate: &str, target: &str, args: &[&str]) -> PathBuf {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../EmuSen/obj").join(target);
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(cores_dir().join(krate))
        .args(["build", "--quiet", "--release"])
        .args(args)
        .arg("--target-dir")
        .arg(&out)
        .env_remove("CARGO_TARGET_DIR")
        .status()
        .expect("cargo runs");
    assert!(status.success(), "{krate} did not build: {status}");
    out.join("release")
}

/// The device test core's library, built once a process; `EMUSEN_DEVICE_CORE` names a prebuilt one.
pub fn device_core() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        std::env::var_os("EMUSEN_DEVICE_CORE").map(PathBuf::from).unwrap_or_else(|| {
            build("Libretro/emusen-libretro-host", "emusen_libretro_host", &["--example", "device_test_core"])
                .join("examples")
                .join("libdevice_test_core.so")
        })
    })
}

/// The parts of an installed libretro entry, and the runner its proxy starts.
pub struct Hosted {
    pub proxy: PathBuf,
    pub host: PathBuf,
    pub core: PathBuf,
    pub runner: PathBuf,
}

/// The libretro entry's parts, built once a process.
pub fn hosted() -> &'static Hosted {
    static BUILT: OnceLock<Hosted> = OnceLock::new();
    BUILT.get_or_init(|| Hosted {
        proxy: build("Libretro/emusen-libretro-proxy", "emusen_libretro_proxy", &["--lib"]).join("libemusen_libretro_proxy.so"),
        host: build("Libretro/emusen-libretro-host", "emusen_libretro_host", &["--lib"]).join("libemusen_libretro_host.so"),
        core: build("Libretro/emusen-libretro-testcore", "emusen_libretro_testcore", &["--lib"]).join("libemusen_libretro_testcore.so"),
        runner: build("Shared/emusen-core-broker", "emusen-core-runner", &["--bin", "emusen-core-runner"]).join("emusen-core-runner"),
    })
}

/// An installation folder whose manifest vouches for the proxy and the libretro host, as a published one does: the
/// runtime's `cores` folder for a hosted core (EmuSen_Cores.md §7).
pub fn installation(dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let parts = hosted();
    let line = |p: &Path| format!("{}  {}\n", sha256(p), p.file_name().unwrap().to_string_lossy());
    std::fs::write(dir.join("cores.manifest"), line(&parts.proxy) + &line(&parts.host)).unwrap();
    dir.to_path_buf()
}

/// The libretro test core's entry under the player's `cores`, laid out as the Cores window lays one (`libretro/<stem>`:
/// the proxy, the libretro host, the core and its information file); the core's library, whose hash the runtime approves.
pub fn libretro_entry(cores: &Path) -> PathBuf {
    let parts = hosted();
    let folder = cores.join("libretro").join("testcore");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(&parts.proxy, folder.join("emusen-core-broker.so")).unwrap();
    std::fs::copy(&parts.host, folder.join("emusen-libretro-host.so")).unwrap();
    let core = folder.join("testcore_libretro.so");
    std::fs::copy(&parts.core, &core).unwrap();
    let info = "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nsystemid = \"\"\n";
    std::fs::write(folder.join("testcore_libretro.info"), info).unwrap();
    core
}

/// A file's SHA-256 in hex, by `sha256sum`, as the manifest and the approvals name it.
pub fn sha256(path: &Path) -> String {
    let out = Command::new("sha256sum").arg(path).output().expect("sha256sum");
    String::from_utf8_lossy(&out.stdout).split_whitespace().next().expect("a hash").to_string()
}
