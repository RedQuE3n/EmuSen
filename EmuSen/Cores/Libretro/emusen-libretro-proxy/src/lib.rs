//! The proxy of one installed libretro core: the entry's libretro host run by the broker in a confined process, the
//! libretro core handed to it as a sealed copy whose SHA-256 core info gives the host to compare with its approval
//! (EmuSen_LibretroHost.md §1.3, §3).

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use emusen_core_broker::client::Spawn;
use emusen_core_broker::proxy::Target;
use emusen_core_broker::sandbox::Policy;
use emusen_native::core::caps;

/// The libretro host's capabilities, which this proxy's exports match; the host refuses the pair if they differ.
pub const CAPABILITIES: u64 = caps::RESET
    | caps::PHASES
    | caps::AXES
    | caps::SETTINGS
    | caps::SETTING_NOTES
    | caps::PRESENCE
    | caps::SNAPSHOT
    | caps::FRAME_SERIAL
    | caps::BATTERY_DIRTY
    | caps::IMAGE_PATH
    | caps::MEDIA;

/// The cores whose memory was measured to hold still between frames, for which the frame-end window is exact. PCEE2's
/// does not: its emulation runs on threads of its own between `retro_run`s, the first MiB of its RAM moving between two
/// reads in 62 of 1,000 frames of Jak and Daxter (34 with MTVU off), measured 2026-10-31 (EmuSen_LibretroHost.md §10c).
const STILL_BETWEEN_FRAMES: &[&str] = &["testcore"];

/// The entry this proxy was copied into, its libretro core sealed, and that copy's hash.
struct Entry {
    folder: PathBuf,
    stem: String,
    file: String,
    sealed: OwnedFd,
    sha256: String,
}

static ENTRY: OnceLock<Result<Entry, String>> = OnceLock::new();

fn own_folder() -> Result<PathBuf, String> {
    let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
    if unsafe { libc::dladdr(own_folder as *const libc::c_void, &mut info) } == 0 || info.dli_fname.is_null() {
        return Err("the proxy cannot tell which file it was loaded from".into());
    }
    let path = Path::new(unsafe { std::ffi::CStr::from_ptr(info.dli_fname) }.to_str().map_err(|e| e.to_string())?).to_path_buf();
    std::fs::canonicalize(&path).map_err(|e| format!("{}: {e}", path.display()))?.parent().map(Path::to_path_buf).ok_or_else(|| "the proxy has no folder".into())
}

/// The core copied into memory and sealed against every change, so that what is hashed is what runs.
fn seal(path: &Path) -> Result<(OwnedFd, String), String> {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let fd = unsafe { libc::memfd_create(c"libretro-core".as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
    if fd < 0 {
        return Err(format!("a sealed copy of the core: {}", std::io::Error::last_os_error()));
    }
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    let mut f = std::fs::File::from(owned.try_clone().map_err(|e| e.to_string())?);
    f.write_all(&bytes).map_err(|e| e.to_string())?;
    let seals = libc::F_SEAL_WRITE | libc::F_SEAL_SHRINK | libc::F_SEAL_GROW | libc::F_SEAL_SEAL;
    if unsafe { libc::fcntl(owned.as_raw_fd(), libc::F_ADD_SEALS, seals) } != 0 {
        return Err(format!("sealing the core's copy: {}", std::io::Error::last_os_error()));
    }
    let hash = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    Ok((owned, hash))
}

fn entry() -> Result<&'static Entry, String> {
    ENTRY
        .get_or_init(|| {
            let folder = own_folder()?;
            let suffix = format!("_libretro.{}", emusen_core_entry::library_suffix());
            let mut found = std::fs::read_dir(&folder).map_err(|e| e.to_string())?.flatten().filter_map(|e| e.file_name().to_str().map(str::to_owned)).filter(|n| n.ends_with(&suffix));
            let file = found.next().ok_or_else(|| format!("{} holds no libretro core", folder.display()))?;
            if found.next().is_some() {
                return Err(format!("{} holds more than one libretro core", folder.display()));
            }
            let stem = emusen_core_entry::stem_of_core(&file).unwrap_or_default().to_owned();
            let (sealed, sha256) = seal(&folder.join(&file))?;
            Ok(Entry { folder, stem, file, sealed, sha256 })
        })
        .as_ref()
        .map_err(String::clone)
}

/// The runner: named by `EMUSEN_CORE_RUNNER`, else beside the program that loaded the proxy, or in its `cores`.
fn runner() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("EMUSEN_CORE_RUNNER") {
        return Ok(PathBuf::from(p));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("the program has no folder")?;
    [dir.join("emusen-core-runner"), dir.join("cores").join("emusen-core-runner")].into_iter().find(|p| p.is_file()).ok_or_else(|| format!("no emusen-core-runner beside {}", exe.display()))
}

struct Libretro;

impl Target for Libretro {
    const CAPABILITIES: u64 = CAPABILITIES;

    fn spawn(image_path: Option<&str>) -> Result<Spawn, String> {
        let e = entry()?;
        let runner = runner()?;
        let data = e.folder.parent().unwrap_or(&e.folder).join(".data").join(&e.stem);
        std::fs::create_dir_all(data.join("cache").join("mesa")).map_err(|err| format!("{}: {err}", data.display()))?;
        let mut read: Vec<PathBuf> = ["/etc", "/proc/self", "/proc/cpuinfo", "/proc/meminfo", "/proc/stat", "/sys"].iter().map(PathBuf::from).filter(|p| p.exists()).collect();
        if let Some(dir) = image_path.and_then(|p| Path::new(p).parent()) {
            read.push(dir.to_path_buf());
        }
        let mut read_exec: Vec<PathBuf> = ["/usr", "/lib", "/lib64"].iter().map(PathBuf::from).filter(|p| p.exists()).collect();
        read_exec.push(e.folder.clone());
        read_exec.extend(runner.parent().map(Path::to_path_buf));
        let devices = ["/dev/dri", "/dev/null", "/dev/zero", "/dev/urandom", "/dev/random"].iter().map(PathBuf::from).filter(|p| p.exists()).collect();
        Ok(Spawn {
            runner,
            target: e.folder.join(format!("{}.{}", emusen_core_entry::ADAPTER, emusen_core_entry::library_suffix())),
            policy: Policy { read_exec, read, read_write: vec![data.clone(), PathBuf::from("/dev/shm")], devices, private_shm: true },
            env: vec![("EMUSEN_LIBRETRO_CORE_FD".into(), "4".into()), ("MESA_SHADER_CACHE_DIR".into(), data.join("cache").join("mesa").to_string_lossy().into_owned())],
            fds: vec![e.sealed.as_raw_fd()],
            require_abi: std::env::var("EMUSEN_BROKER_REQUIRE_ABI").ok().and_then(|v| v.parse().ok()).unwrap_or(1),
        })
    }

    fn window() -> bool {
        entry().is_ok_and(|e| STILL_BETWEEN_FRAMES.contains(&e.stem.as_str()))
    }

    fn info_extra() -> String {
        entry().map_or_else(|_| String::new(), |e| format!(r#""libretro":{{"file":{:?},"sha256":"{}"}}"#, e.file, e.sha256))
    }
}

emusen_core_broker::broker_exports!(Libretro; reset, phases, axes, settings, setting_notes, media, presence);
