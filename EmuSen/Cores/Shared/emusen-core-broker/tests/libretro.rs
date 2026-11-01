// The libretro test core run through the proxy and the runner: machines side by side, faults contained, the sandbox's
// refusals, and the hash of the sealed copy (EmuSen_LibretroHost.md §3.5).

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};
use emusen_libretro_testcore::layout::*;

static TURN: Mutex<()> = Mutex::new(());
static LIB: OnceLock<(&'static Library, PathBuf)> = OnceLock::new();
const GAME: &[u8] = b"TCG1 a test game";

fn suffix() -> &'static str {
    emusen_core_entry_suffix()
}

fn emusen_core_entry_suffix() -> &'static str {
    if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

/// A brokered entry: the proxy as the file hosts load, the libretro host and the test core beside it.
fn library() -> (&'static Library, PathBuf) {
    let (lib, root) = LIB.get_or_init(|| {
        let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("broker-{}", std::process::id()));
        let entry = root.join("libretro").join("testcore");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&entry).unwrap();
        let s = suffix();
        std::fs::copy(deps.join(format!("libemusen_libretro_proxy.{s}")), entry.join(format!("emusen-core-broker.{s}"))).unwrap();
        std::fs::copy(deps.join(format!("libemusen_libretro_host.{s}")), entry.join(format!("emusen-libretro-host.{s}"))).unwrap();
        std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{s}")), entry.join(format!("testcore_libretro.{s}"))).unwrap();
        std::fs::write(entry.join("testcore_libretro.info"), "corename = \"TestCore\"\nlicense = \"GPLv3\"\n").unwrap();
        unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", env!("CARGO_BIN_EXE_emusen-core-runner")) };
        (Library::load(&entry.join(format!("emusen-core-broker.{s}"))).unwrap_or_else(|e| panic!("{e:?}")), root)
    });
    (*lib, root.clone())
}

fn turn() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|e| e.into_inner())
}

fn make(settings: &str) -> Result<Machine, emusen_core_host::machine::Refusal> {
    let (lib, _) = library();
    Machine::create(lib, &Create { settings, pixel_formats: 0b011, ..Create::new(GAME) })
}

fn ram(m: &mut Machine, at: usize) -> u8 {
    let mut b = [0u8; 1];
    m.space_read(0, at as u32, &mut b).unwrap();
    b[0]
}

#[test]
fn core_info_gives_the_hash_of_the_copy_that_runs() {
    let _t = turn();
    let (lib, root) = library();
    let info = lib.info_text().unwrap();
    let file = std::fs::read(root.join("libretro/testcore").join(format!("testcore_libretro.{}", suffix()))).unwrap();
    let hash: String = emusen_core_host::sha256::sha256_hex(&file);
    assert!(info.contains(&format!(r#""libretro":{{"file":"testcore_libretro.{}","sha256":"{hash}"}}"#, suffix())), "{info}");
    assert!(lib.settings_text().unwrap().contains(r#""key":"broker.deadline""#));
}

#[test]
fn two_machines_run_side_by_side_in_processes_of_their_own() {
    let _t = turn();
    let mut a = make("").unwrap();
    let mut b = make("tc_rate=44100").unwrap();
    for _ in 0..10 {
        a.advance().unwrap();
        b.advance().unwrap();
    }
    assert_eq!((a.audio_rate(), b.audio_rate()), (48_000, 44_100));
    let mut log = String::new();
    a.log_drain(&mut log).unwrap();
    assert!(log.contains("the core runs in process") && log.contains("Landlock"), "{log}");
}

#[test]
fn a_fault_the_core_handles_itself_runs_on() {
    let _t = turn();
    let mut m = make("tc_fault=handled_segv").unwrap();
    for _ in 0..4 {
        m.advance().unwrap();
    }
    assert_eq!(ram(&mut m, FAULT), 1);
}

#[test]
fn a_crash_or_an_abort_ends_the_core_and_not_the_host() {
    let _t = turn();
    for (fault, signal) in [("segv", 11), ("abort", 6)] {
        let mut m = make(&format!("tc_fault={fault}")).unwrap();
        m.advance().unwrap();
        assert_eq!(m.advance().map_err(|e| e.0), Err(-10), "{fault}");
        assert!(m.last_error().contains(&format!("signal {signal}")), "{fault}: {}", m.last_error());
        assert_eq!(m.frame_count(), -10);
        assert_eq!(m.library().words(-10), "the core's process ended");
        drop(m);
        make("").unwrap().advance().unwrap();
    }
}

#[test]
fn a_frame_that_never_ends_is_stopped_at_its_deadline() {
    let _t = turn();
    let mut m = make("tc_fault=hang\nbroker.deadline=1").unwrap();
    m.advance().unwrap();
    let start = Instant::now();
    assert_eq!(m.advance().map_err(|e| e.0), Err(-11));
    assert!(start.elapsed() < Duration::from_secs(5), "{:?}", start.elapsed());
    assert!(m.last_error().contains("stopped answering within 1 s"), "{}", m.last_error());
}

#[test]
fn the_kernel_refuses_a_write_outside_the_cores_folders_and_the_network() {
    let _t = turn();
    for fault in ["write_outside", "socket"] {
        let mut m = make(&format!("tc_fault={fault}")).unwrap();
        for _ in 0..3 {
            m.advance().unwrap();
        }
        assert_eq!(ram(&mut m, FAULT), 2, "{fault} refused by Landlock");
    }
}

#[test]
fn a_kernel_that_cannot_confine_the_core_fails_create_in_words() {
    let _t = turn();
    let _ = library();
    unsafe { std::env::set_var("EMUSEN_BROKER_REQUIRE_ABI", "99") };
    let refused = make("").err();
    unsafe { std::env::remove_var("EMUSEN_BROKER_REQUIRE_ABI") };
    let r = refused.expect("create refused");
    assert_eq!(r.status, -10);
    assert!(r.words.contains("Landlock ABI") && r.words.contains("never runs unconfined"), "{}", r.words);
}

#[test]
fn a_frame_costs_one_round_trip_per_call_measured() {
    let _t = turn();
    let mut m = make("").unwrap();
    m.advance().unwrap();
    let n = 2000;
    let start = Instant::now();
    for _ in 0..n {
        m.frame_count();
    }
    let per_call = start.elapsed() / n;
    let mut px = vec![0u8; m.frame_bytes().unwrap()];
    let start = Instant::now();
    for _ in 0..200 {
        m.advance().unwrap();
        m.frame_copy(&mut px).unwrap();
    }
    let per_frame = start.elapsed() / 200;
    eprintln!("broker: {per_call:?} a round trip, {per_frame:?} a frame with its {}-byte picture", px.len());
    assert!(per_call < Duration::from_millis(2), "{per_call:?}");
}
