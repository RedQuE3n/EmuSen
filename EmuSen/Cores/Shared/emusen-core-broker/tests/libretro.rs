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

/// An entry laid out as Cores lays it out under `root`: the proxy as the file hosts load, the libretro host and the
/// test core beside it; the proxy's path and the test core's hash.
fn entry(root: &std::path::Path) -> (PathBuf, String) {
    let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let entry = root.join("libretro").join("testcore");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(&entry).unwrap();
    let s = suffix();
    std::fs::copy(deps.join(format!("libemusen_libretro_proxy.{s}")), entry.join(format!("emusen-core-broker.{s}"))).unwrap();
    std::fs::copy(deps.join(format!("libemusen_libretro_host.{s}")), entry.join(format!("emusen-libretro-host.{s}"))).unwrap();
    std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{s}")), entry.join(format!("testcore_libretro.{s}"))).unwrap();
    std::fs::write(entry.join("testcore_libretro.info"), "corename = \"TestCore\"\nlicense = \"GPLv3\"\n").unwrap();
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", env!("CARGO_BIN_EXE_emusen-core-runner")) };
    let hash = emusen_core_host::sha256::sha256_hex(&std::fs::read(entry.join(format!("testcore_libretro.{s}"))).unwrap());
    (entry.join(format!("emusen-core-broker.{s}")), hash)
}

/// The brokered entry every test shares, loaded with its core's hash approved.
fn library() -> (&'static Library, PathBuf) {
    let (lib, root) = LIB.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("broker-{}", std::process::id()));
        let (proxy, hash) = entry(&root);
        (Library::load_expecting(&proxy, &hash).unwrap_or_else(|e| panic!("{e:?}")), root)
    });
    (*lib, root.clone())
}

/// The runners this process has started that are still alive.
fn runners() -> usize {
    let me = std::process::id().to_string();
    std::fs::read_dir("/proc").unwrap().flatten().filter_map(|e| std::fs::read_to_string(e.path().join("stat")).ok()).filter(|stat| {
        let (Some(open), Some(close)) = (stat.find('('), stat.rfind(')')) else { return false };
        stat[open + 1..close].starts_with("emusen-core-run") && stat[close + 2..].split(' ').nth(1) == Some(me.as_str())
    }).count()
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
fn a_core_changed_between_the_hosts_hash_and_the_load_never_runs() {
    let _t = turn();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("broker-expect-{}", std::process::id()));
    let (proxy, approved) = entry(&root);
    let core = proxy.with_file_name(format!("testcore_libretro.{}", suffix()));
    let mut changed = std::fs::read(&core).unwrap();
    changed.extend_from_slice(b"a rebuilt core");
    std::fs::write(&core, &changed).unwrap();
    let actual = emusen_core_host::sha256::sha256_hex(&changed);
    let before = runners();
    let why = match Library::load_expecting(&proxy, &approved) {
        Err(emusen_core_host::library::LoadError::NotExpected(why)) => why,
        other => panic!("loaded: {:?}", other.map(|l| l.file_name())),
    };
    assert!(why.contains(&approved) && why.contains(&actual), "{why}");
    // Every later call answers as a broken library, and none starts a runner.
    let path = std::ffi::CString::new(proxy.to_str().unwrap()).unwrap();
    let handle = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW) };
    assert!(!handle.is_null());
    let info: unsafe extern "C" fn(*mut u8, usize) -> i64 = unsafe { std::mem::transmute(libc::dlsym(handle, c"emusen_core_info".as_ptr())) };
    let mut text = vec![0u8; 4096];
    let n = unsafe { info(text.as_mut_ptr(), text.len()) } as usize;
    let text = String::from_utf8_lossy(&text[..n.min(4096)]).into_owned();
    assert!(text.contains("broken-proxy") && text.contains(&approved) && text.contains(&actual), "{text}");
    let create: unsafe extern "C" fn(*const emusen_native::core::sys::CreateParams, *mut i32) -> *mut u8 = unsafe { std::mem::transmute(libc::dlsym(handle, c"emusen_core_create".as_ptr())) };
    let mut params: emusen_native::core::sys::CreateParams = unsafe { std::mem::zeroed() };
    params.size = size_of::<emusen_native::core::sys::CreateParams>() as u32;
    params.host_abi_version = emusen_native::core::sys::ABI_VERSION;
    (params.image, params.image_len, params.pixel_formats) = (GAME.as_ptr(), GAME.len(), 1);
    let mut status = 0;
    assert!(unsafe { create(&params, &mut status) }.is_null());
    assert_eq!(status, emusen_native::core::sys::status::NOT_EXPECTED);
    assert_eq!(runners(), before, "a runner started for a core that is not the approved build");
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
fn a_core_reads_firmware_through_a_link_in_its_system_folder_and_nothing_beside_it() {
    let _t = turn();
    let (_, root) = library();
    let elsewhere = root.join("elsewhere");
    let bios = root.join("libretro").join(emusen_core_entry::DATA).join("testcore").join("system").join("bios");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::create_dir_all(&bios).unwrap();
    std::fs::write(elsewhere.join("firmware.bin"), b"dummy firmware").unwrap();
    std::fs::write(elsewhere.join("other.bin"), b"not granted").unwrap();
    let link = bios.join("firmware.bin");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(elsewhere.join("firmware.bin"), &link).unwrap();
    let mut m = make("tc_fault=read_system").unwrap();
    for _ in 0..3 {
        m.advance().unwrap();
    }
    let read = ram(&mut m, FAULT);
    drop(m);
    std::fs::remove_file(&link).unwrap();
    assert_eq!(read, 1, "1: the link read and its neighbour refused; 2: the link refused; 3: the neighbour read");
}

#[test]
fn a_core_that_cannot_start_says_so_across_the_process_boundary() {
    let _t = turn();
    let refused = make("tc_fail=load").err().expect("load refused");
    assert_eq!(refused.status, -13);
    assert!(refused.words.contains(emusen_libretro_testcore::FAIL_AT_LOAD), "{}", refused.words);
    let mut m = make("tc_fail=first_frame").unwrap();
    assert_eq!(m.advance().map_err(|e| e.0), Err(-18));
    assert!(m.last_error().contains(emusen_libretro_testcore::FAIL_AT_FIRST_FRAME), "{}", m.last_error());
    drop(m);
    make("").unwrap().advance().unwrap();
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

#[test]
fn reads_between_frames_come_from_the_window_and_stay_exact() {
    let _t = turn();
    let mut m = make("").unwrap();
    let counter = |m: &mut Machine| {
        let mut b = [0u8; 4];
        m.space_read(0, FRAME_COUNTER as u32, &mut b).unwrap();
        u32::from_le_bytes(b)
    };
    let mut saved = Vec::new();
    for n in 1..=200u32 {
        m.advance().unwrap();
        assert_eq!(counter(&mut m), n, "the counter after frame {n}");
        m.space_write(0, LIVES as u32, &[7]).unwrap();
        assert_eq!(ram(&mut m, LIVES), 7, "a write is read back at once");
        if n == 100 {
            saved = vec![0u8; m.state_size(0).unwrap()];
            m.state_save(0, &mut saved).unwrap();
        }
    }
    m.state_load(&saved).unwrap();
    assert_eq!(counter(&mut m), 100, "a load ends the window");
    let start = Instant::now();
    for _ in 0..1000 {
        counter(&mut m);
    }
    let missed = start.elapsed();
    m.advance().unwrap();
    let start = Instant::now();
    for _ in 0..1000 {
        counter(&mut m);
    }
    let held = start.elapsed();
    eprintln!("window: 1,000 reads {missed:?} before it held them, {held:?} after");
    assert!(held * 5 < missed, "{held:?} against {missed:?}");
}

#[test]
fn memory_between_frames_holds_still_while_the_core_rests() {
    let _t = turn();
    let mut m = make("").unwrap();
    let size = m.space_size(0).unwrap().min(1 << 20) as usize;
    let (mut a, mut b) = (vec![0u8; size], vec![0u8; size]);
    for _ in 0..1000 {
        m.advance().unwrap();
        m.space_read(0, 0, &mut a).unwrap();
        std::thread::sleep(Duration::from_micros(200));
        m.space_read(0, 0, &mut b).unwrap();
        assert!(a == b, "memory moved between two reads with no frame between them");
    }
}

/// The test core's pictures through the broker: software, or Vulkan into Serenity's ring, its handles crossing to the
/// runner by SCM_RIGHTS (EmuSen_LibretroHost.md §10f). Skipped where no card can make a ring.
#[test]
fn a_ring_lent_across_the_process_boundary_holds_the_software_picture() {
    use emusen_native::core::sys::{RingImage, RingLend};
    let _t = turn();
    let gpu = match emusen_serenity::vulkan::Device::create(std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), false) {
        Ok(d) if d.rings() => d,
        _ => return eprintln!("skipped: no Vulkan device here can make a ring"),
    };
    let (lib, _) = library();
    let rows = |m: &Machine| {
        let f = m.frame_info().unwrap();
        let mut all = vec![0u8; m.frame_bytes().unwrap()];
        m.frame_copy(&mut all).unwrap();
        (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4].to_vec()).collect::<Vec<u8>>()
    };
    let mut soft = Machine::create(lib, &Create { settings: "", pixel_formats: 1, ..Create::new(GAME) }).unwrap();
    let mut m = Machine::create(lib, &Create { settings: "tc_hw=vulkan", pixel_formats: 1, ..Create::new(GAME) }).unwrap();
    let ring = gpu.create_ring(640, 480, 3).unwrap();
    let images: Vec<RingImage> = ring.images.iter().map(|i| RingImage { size: size_of::<RingImage>() as u32, memory_type: i.memory_type, handle: i.handle as u64, allocation_size: i.allocation_size, width: i.width, height: i.height, format: i.format, usage: i.usage }).collect();
    let lend = RingLend { size: size_of::<RingLend>() as u32, count: 3, images: images.as_ptr(), image_size: size_of::<RingImage>(), ready: ring.ready_fd as u64, done: ring.done_fd as u64, device_uuid: ring.device_uuid, driver_uuid: ring.driver_uuid };
    m.ring_lend(Some(&lend)).unwrap();
    for n in 0..5u64 {
        soft.advance().unwrap();
        m.advance().unwrap();
        let f = m.frame_ring().unwrap().expect("the brokered picture is in the ring");
        assert_eq!(f.value, n + 1);
        assert!(ring.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap() == rows(&soft), "frame {n}");
    }
    m.ring_lend(None).unwrap();
}

/// §10f's measurement through the broker: the emulation thread's cost per frame and the delay from set_buttons to the
/// picture on the host's side, for software, the read-back and the ring. Run with --ignored --nocapture.
#[test]
#[ignore]
fn measure_the_ring_through_the_broker() {
    use emusen_native::core::sys::{RingImage, RingLend};
    let _t = turn();
    let Ok(gpu) = emusen_serenity::vulkan::Device::create(std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), false) else { return };
    let (lib, _) = library();
    let stats = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v.iter().sum::<f64>() / v.len() as f64, v[v.len() / 2], v[v.len() * 99 / 100])
    };
    for (name, settings) in [("software", ""), ("read-back", "tc_hw=vulkan"), ("ring", "tc_hw=vulkan")] {
        let mut m = Machine::create(lib, &Create { settings, pixel_formats: 1, ..Create::new(GAME) }).unwrap();
        let ring = gpu.create_ring(640, 480, 3).unwrap();
        let images: Vec<RingImage> = ring.images.iter().map(|i| RingImage { size: size_of::<RingImage>() as u32, memory_type: i.memory_type, handle: i.handle as u64, allocation_size: i.allocation_size, width: i.width, height: i.height, format: i.format, usage: i.usage }).collect();
        let lend = RingLend { size: size_of::<RingLend>() as u32, count: 3, images: images.as_ptr(), image_size: size_of::<RingImage>(), ready: ring.ready_fd as u64, done: ring.done_fd as u64, device_uuid: ring.device_uuid, driver_uuid: ring.driver_uuid };
        if name == "ring" {
            m.ring_lend(Some(&lend)).unwrap();
        }
        let mut copy = vec![0u8; 640 * 480 * 4];
        let (mut cost, mut delay) = (Vec::new(), Vec::new());
        for n in 0..600 {
            let press = n % 10 == 5;
            let t0 = Instant::now();
            if press || n % 10 == 6 {
                m.set_buttons(0, press as u32, 1).unwrap();
            }
            let t1 = Instant::now();
            m.advance().unwrap();
            let shown = if name == "ring" { m.frame_ring().unwrap() } else { None };
            if name != "ring" {
                m.frame_copy(&mut copy).unwrap();
            }
            let spent = t1.elapsed();
            if let Some(f) = shown {
                ring.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap();
            }
            if n >= 60 {
                cost.push(spent.as_secs_f64() * 1e3);
                if press {
                    delay.push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
        let (c, d) = (stats(&mut cost), stats(&mut delay));
        println!("brokered {name:9}: emulation thread mean {:.3} median {:.3} p99 {:.3} ms; set_buttons to picture mean {:.3} median {:.3} p99 {:.3} ms", c.0, c.1, c.2, d.0, d.1, d.2);
    }
}
