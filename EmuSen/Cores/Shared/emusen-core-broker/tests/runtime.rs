// An installed libretro entry through the Rust runtime: listed as waiting, approved by its core's hash (which writes the
// proxy's sidecar), opened on a game, run, its state taken and loaded; and each refusal before anything of it starts
// (EmuSen_LibretroHost.md §10k).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use emusen_core_host::entry::Entry;
use emusen_core_host::trust::{MANIFEST, Standing};
use emusen_crystal::abi::emusen_crystal_frame_info;
use emusen_crystal::open::Asked;
use emusen_crystal::runtime::{Config, Runtime};
use emusen_crystal::session::OpenedWith;
use emusen_galaxia::tree::{Override, tree_mut};

static TURN: Mutex<()> = Mutex::new(());
const SUFFIX: &str = if cfg!(target_os = "macos") { "dylib" } else { "so" };

fn turn() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|e| e.into_inner())
}

fn deps() -> PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}

fn hash(path: &Path) -> String {
    emusen_core_host::sha256::file_hex(path).unwrap()
}

/// An installation directory whose manifest vouches for this build's proxy and libretro host, a player's cores
/// directory holding the test core's entry as Cores lays one out, and the data and config folders pointed under `root`.
struct Layout {
    root: PathBuf,
    installation: PathBuf,
    cores: PathBuf,
    entry: Entry,
}

fn layout(name: &str) -> Layout {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("runtime-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (installation, cores) = (root.join("lib"), root.join("Cores"));
    std::fs::create_dir_all(&installation).unwrap();
    let (proxy, host) = (deps().join(format!("libemusen_libretro_proxy.{SUFFIX}")), deps().join(format!("libemusen_libretro_host.{SUFFIX}")));
    let manifest = format!("{}  libemusen_libretro_proxy.{SUFFIX}\n{}  libemusen_libretro_host.{SUFFIX}\n", hash(&proxy), hash(&host));
    std::fs::write(installation.join(MANIFEST), manifest).unwrap();
    let entry = Entry::new(&cores, "testcore").unwrap();
    std::fs::create_dir_all(&entry.folder).unwrap();
    std::fs::copy(&proxy, entry.broker_path()).unwrap();
    std::fs::copy(&host, entry.adapter_path()).unwrap();
    std::fs::copy(deps().join(format!("libemusen_libretro_testcore.{SUFFIX}")), entry.core_path()).unwrap();
    std::fs::write(entry.info_path(), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nsystemid = \"\"\n").unwrap();
    {
        let mut t = tree_mut();
        t.set_override(Override::Data, Some(root.join("Home").to_string_lossy().into_owned()));
        t.set_override(Override::Config, Some(root.join("Config").to_string_lossy().into_owned()));
    }
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", env!("CARGO_BIN_EXE_emusen-core-runner")) };
    Layout { root, installation, cores, entry }
}

fn runtime(l: &Layout) -> Runtime {
    let dir = |p: &Path| vec![p.to_string_lossy().into_owned()];
    Runtime::new(&Config { cores: Some(dir(&l.installation)), player_cores: Some(dir(&l.cores)), development: Some(false), sqlite: None, lender: None, fetch: None })
}

fn frame(s: &emusen_crystal::session::Session) -> emusen_crystal_frame_info {
    let mut info = emusen_crystal_frame_info { size: size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
    s.frame(Some(&mut info)).unwrap();
    info
}

fn picture(s: &emusen_crystal::session::Session) -> Vec<u8> {
    let mut out = vec![0u8; 4 << 20];
    let n = s.copy_picture(&mut out).unwrap();
    out.truncate(n);
    out
}

#[test]
fn an_approved_entry_opens_on_the_rust_runtime_runs_and_takes_and_loads_a_state() {
    let _t = turn();
    let l = layout("open");
    let core = hash(&l.entry.core_path());
    let rt = runtime(&l);
    let d = rt.discovery();
    assert!(d.found().is_empty(), "an entry with no sidecar is no engine yet");
    assert_eq!(d.waiting().iter().map(|w| w.sha256.as_str()).collect::<Vec<_>>(), [core.as_str()]);
    rt.approve(&core).unwrap();
    let d = rt.discovery();
    let found = d.found().iter().find(|c| c.entry.is_some()).expect("the approved entry is listed");
    assert_eq!(found.sidecar.info.libretro.as_ref().map(|l| (l.file.as_str(), l.sha256.as_str())), Some((format!("testcore_libretro.{SUFFIX}").as_str(), core.as_str())));
    assert_eq!(found.standing(&rt.trust()), Standing::Approved);
    assert!(d.waiting().is_empty());
    let engine = found.engine_name().to_string();
    drop(d);

    let game = l.root.join("game.tcg");
    std::fs::write(&game, b"TCG1 a test game").unwrap();
    let game = game.to_string_lossy().into_owned();
    let id = rt.open(&Asked { path: Some(&game), console: None, engine: None, opened: OpenedWith { flags: emusen_crystal::abi::OPEN_NO_BATTERY, ..OpenedWith::default() } }).unwrap();
    let s = rt.sessions.get(id).unwrap();
    assert!(s.info_json().contains(&format!("\"engine\":\"{engine}\"")), "{}", s.info_json());
    s.button(0, 3, true).unwrap();
    let mut f = frame(&s);
    for _ in 0..19 {
        f = frame(&s);
    }
    assert_eq!((f.ran, f.width, f.height), (1, 320, 240));
    let state = l.root.join("game.state").to_string_lossy().into_owned();
    s.save_state(&state, None, false).unwrap();
    let after_save = {
        frame(&s);
        picture(&s)
    };
    for _ in 0..10 {
        frame(&s);
    }
    assert_ne!(picture(&s), after_save, "the picture moves on");
    rt.with_ctx(|ctx| s.load_state(&state, None, ctx)).unwrap();
    frame(&s);
    assert_eq!(picture(&s), after_save, "the state loaded brings the picture back");
    drop(s);
    rt.close(id).unwrap();
}

#[test]
fn an_entry_starts_nothing_while_unapproved_rebuilt_or_its_copies_stale() {
    let _t = turn();
    let l = layout("refusals");
    let core = hash(&l.entry.core_path());
    let rt = runtime(&l);
    rt.approve(&core).unwrap();
    rt.revoke(&core).unwrap();
    let d = rt.discovery();
    let c = d.found().iter().find(|c| c.entry.is_some()).unwrap();
    assert_eq!(c.standing(&rt.trust()), Standing::NeedsApproval);
    assert!(c.open(&rt.trust()).err().unwrap().contains("has not been approved"));
    assert!(c.awaits_approval());

    // Another build of the core, and a libretro host copy that is not this installation's: each refused before loading.
    let rebuilt = layout("rebuilt");
    let rt2 = runtime(&rebuilt);
    let approved = hash(&rebuilt.entry.core_path());
    rt2.approve(&approved).unwrap();
    let mut bytes = std::fs::read(rebuilt.entry.core_path()).unwrap();
    bytes.extend_from_slice(b"a rebuilt core");
    std::fs::write(rebuilt.entry.core_path(), &bytes).unwrap();
    rt2.rescan();
    let d2 = rt2.discovery();
    let c2 = d2.found().iter().find(|c| c.entry.is_some()).unwrap();
    assert_eq!(c2.standing(&rt2.trust()), Standing::NeedsApproval);
    assert!(c2.open(&rt2.trust()).err().unwrap().contains("is not the build that was approved"));
    assert_eq!(d2.waiting().len(), 1, "the new build waits for its approval");

    let stale = layout("stale");
    let rt3 = runtime(&stale);
    rt3.approve(&hash(&stale.entry.core_path())).unwrap();
    std::fs::write(stale.entry.adapter_path(), b"not the installation's").unwrap();
    rt3.rescan();
    let d3 = rt3.discovery();
    let c3 = d3.found().iter().find(|c| c.entry.is_some()).unwrap();
    assert_eq!(c3.standing(&rt3.trust()), Standing::NeedsRefresh);
    let why = c3.open(&rt3.trust()).err().unwrap().to_string();
    assert!(why.contains("needs refresh") && !c3.awaits_approval(), "{why}");
}
