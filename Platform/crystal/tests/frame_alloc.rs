//! A frame through the runtime allocates nothing once its session has its buffers: `frame`, `picture_copy` and
//! `audio_drain` as the C layer calls them, with the controls held and moved between, rewind off and then on
//! (EmuSen_Backend.md §29.4). The core's own allocations are its standard library's, not this binary's, so the count
//! is the runtime's alone.

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use emusen_core_host::sidecar;
use emusen_core_host::sha256;
use emusen_core_host::trust::MANIFEST;
use emusen_crystal::abi::emusen_crystal_frame_info;
use emusen_crystal::open::Asked;
use emusen_crystal::runtime::{Config, Runtime};
use emusen_crystal::session::OpenedWith;
use emusen_galaxia::tree::{Override, tree_mut};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    profile.join("examples").join(if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") })
}

fn scratch(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("crystal-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The test core installed in `dir` with its sidecar and a manifest that vouches for it.
fn install(dir: &Path) {
    let build = scratch("build").join("libcrystal_test_core.so");
    std::fs::copy(example("crystal_test_core"), &build).unwrap();
    let written = sidecar::write(&build, false).unwrap();
    let to = dir.join("libcrystal_test_core.so");
    std::fs::copy(&build, &to).unwrap();
    std::fs::copy(&written, sidecar::path_for(&to)).unwrap();
    std::fs::write(dir.join(MANIFEST), format!("{}  libcrystal_test_core.so\n", sha256::file_hex(&to).unwrap())).unwrap();
}

#[test]
fn a_frame_through_the_runtime_allocates_nothing() {
    let root = scratch("root");
    let cores = root.join("cores");
    std::fs::create_dir_all(&cores).unwrap();
    install(&cores);
    {
        let mut t = tree_mut();
        t.set_override(Override::Data, Some(root.join("Home").to_string_lossy().into_owned()));
        t.set_override(Override::Config, Some(root.join("Config").to_string_lossy().into_owned()));
    }
    let game = root.join("game.tst");
    std::fs::write(&game, [b'V', b'1', b'T', b'C', 4, 0, 1, 2, 3, 4]).unwrap();
    let rt = Runtime::new(&Config { fetch: None, cores: Some(vec![cores.to_string_lossy().into_owned()]), development: Some(false), sqlite: None, lender: None });
    let game = game.to_string_lossy().into_owned();
    let id = rt.open(&Asked { path: Some(&game), console: None, engine: None, opened: OpenedWith::default() }).unwrap();
    let s = rt.sessions.get(id).unwrap();
    let (mut picture, mut sound) = (vec![0u8; 1 << 20], vec![0i16; 8192]);
    let mut info = emusen_crystal_frame_info { size: std::mem::size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
    let mut frame = |i: u32| {
        // Port 2's A: port 1's marks the battery changed, and a battery file written is a file's allocations.
        s.button(1, 8, i & 1 == 1).unwrap();
        s.axis(0, 0, i as f64 / 1000.0).unwrap();
        s.frame(Some(&mut info)).unwrap();
        assert_eq!(info.ran, 1);
        let n = s.copy_picture(&mut picture).unwrap();
        assert!(n <= picture.len());
        s.drain_audio(Some(&mut sound)).unwrap();
        drop(rt.sessions.get(id).unwrap());
    };
    // Past the first battery flush, at frame 300, which gives the battery's buffer its size.
    for i in 0..400 {
        frame(i);
    }
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for i in 400..1400 {
        frame(i);
    }
    let made = ALLOCATIONS.load(Ordering::Relaxed) - before;
    // Rewind on, its budget small enough that the chain reaches it while warming and lets its oldest deltas go after.
    rt.sessions.get(id).unwrap().rewind_set(true, 4, 512, 0).unwrap();
    for i in 1400..1800 {
        frame(i);
    }
    let rewinding = ALLOCATIONS.load(Ordering::Relaxed);
    for i in 1800..2800 {
        frame(i);
    }
    let made_rewinding = ALLOCATIONS.load(Ordering::Relaxed) - rewinding;
    let reel = rt.sessions.get(id).unwrap().rewind_reel_json().unwrap();
    rt.close(id).unwrap();
    assert_eq!(made, 0, "allocations in 1,000 frames");
    assert_eq!(made_rewinding, 0, "allocations in 1,000 frames with rewind on");
    assert!(reel.contains("\"enabled\":true") && !reel.contains("\"depth\":0,"), "{reel}");
}
