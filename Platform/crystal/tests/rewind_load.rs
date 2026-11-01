//! A state load while rewind is on keeps the chain and first takes a snapshot, so Rewind undoes a wrong load
//! (EmuSen_Backend.md §14.13, decided 2026-11-01 for the Pause Card, EmuSen_BigPicture.md §48.8).
use std::path::{Path, PathBuf};

use emusen_core_host::sidecar;
use emusen_core_host::sha256;
use emusen_core_host::trust::MANIFEST;
use emusen_crystal::open::Asked;
use emusen_crystal::runtime::{Config, Runtime};
use emusen_crystal::session::OpenedWith;
use emusen_galaxia::tree::{Override, tree_mut};

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    profile.join("examples").join(if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") })
}

fn scratch(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("crystal-rewind-load-{}-{name}", std::process::id()));
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

/// The newest moment's frame and how many moments the reel holds.
fn reel(json: &str) -> (i64, usize) {
    let frames: Vec<i64> = json.split("\"frame\":").skip(1).map(|t| t.split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap()).collect();
    (*frames.last().unwrap_or(&-1), frames.len())
}

#[test]
fn a_load_with_rewind_on_keeps_the_chain_and_rewind_returns_to_the_moment_before_it_and_with_it_off_holds_none() {
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
    let rt = Runtime::new(&Config { fetch: None, player_cores: None, cores: Some(vec![cores.to_string_lossy().into_owned()]), development: Some(false), sqlite: None, lender: None });
    let game = game.to_string_lossy().into_owned();
    let id = rt.open(&Asked { path: Some(&game), console: None, engine: None, opened: OpenedWith::default() }).unwrap();
    let s = rt.sessions.get(id).unwrap();
    s.rewind_set(true, 4, 1 << 20, 0).unwrap();
    for _ in 0..20 {
        s.frame(None).unwrap();
    }
    let state = root.join("early.state").to_string_lossy().into_owned();
    s.save_state(&state, None, false).unwrap();
    for _ in 0..22 {
        s.frame(None).unwrap();
    }
    let (newest, held) = reel(&s.rewind_reel_json().unwrap());
    assert_eq!(newest, 40);

    rt.with_ctx(|ctx| s.load_state(&state, None, ctx)).unwrap();
    let (after, kept) = reel(&s.rewind_reel_json().unwrap());
    assert_eq!(after, 42, "the moment before the load is the newest");
    assert_eq!(kept, held + 1, "the chain is kept, with one snapshot more");

    // Rewind to the moment before the load puts the game back where it was; a second load at once adds nothing.
    rt.with_ctx(|ctx| s.load_state(&state, None, ctx)).unwrap();
    assert_eq!(reel(&s.rewind_reel_json().unwrap()).1, kept);
    rt.with_ctx(|ctx| s.rewind_to(42, ctx)).unwrap();
    assert_eq!(reel(&s.rewind_reel_json().unwrap()).0, 42);

    // With rewind off a load holds no chain, as before.
    s.rewind_set(false, 0, 0, 0).unwrap();
    s.frame(None).unwrap();
    rt.with_ctx(|ctx| s.load_state(&state, None, ctx)).unwrap();
    assert_eq!(reel(&s.rewind_reel_json().unwrap()).1, 0);
}
