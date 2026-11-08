//! A core's motors followed from its `OUTPUT` events through the session to a simulated pad, as a frontend hands each
//! port's levels to the pad of the player it hears; held at zero where the table or the player's switch says no
//! (EmuSen_Input.md §9).
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use emusen_core_host::sha256;
use emusen_core_host::sidecar;
use emusen_core_host::trust::MANIFEST;
use emusen_crystal::open::Asked;
use emusen_crystal::runtime::{Config, Runtime};
use emusen_crystal::session::OpenedWith;
use emusen_endymion::devices::{SimulatedPad, SimulatedPads, lock};
use emusen_endymion::pads::{PadEvent, Pads};
use emusen_galaxia::tree::{Override, tree, tree_mut};

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    profile.join("examples").join(if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") })
}

fn scratch(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("crystal-rumble-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

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
fn the_test_cores_motor_reaches_the_players_pad_only_where_the_table_and_the_switch_allow() {
    let root = scratch("root");
    let cores = root.join("cores");
    std::fs::create_dir_all(&cores).unwrap();
    install(&cores);
    {
        let mut t = tree_mut();
        t.set_override(Override::Data, Some(root.join("Home")));
        t.set_override(Override::Config, Some(root.join("Config")));
    }
    let game = root.join("game.tst");
    std::fs::write(&game, [b'V', b'1', b'T', b'C', 4, 0, 1, 2, 3, 4]).unwrap();
    let rt = Runtime::new(&Config { fetch: None, player_cores: None, cores: Some(vec![cores.to_string_lossy().into_owned()]), development: Some(false), sqlite: None, lender: None });
    let game = game.to_string_lossy().into_owned();
    let id = rt.open(&Asked { path: Some(&game), console: None, engine: None, opened: OpenedWith::default() }).unwrap();
    let s = rt.sessions.get(id).unwrap();

    // Player 1's pad, as Endymion opens it.
    let set = Arc::new(Mutex::new(SimulatedPads::new()));
    let pad = Arc::new(Mutex::new(SimulatedPad::new(Some("/dev/input/rumble".into()))));
    emusen_endymion::devices::lock_set(&set).connect(pad.clone());
    let mut pads = Pads::new(Box::new(set.clone()));
    let key = pads.start().into_iter().find_map(|e| if let PadEvent::Opened { key, .. } = e { Some(key) } else { None }).expect("the pad opens");
    let frame = |pads: &mut Pads| {
        s.frame(None).unwrap();
        let (strong, weak) = s.rumble.level(0);
        pads.rumble(key, strong, weak);
        lock(&pad).rumble
    };

    // The test core's system has no row in the table, so nothing reaches the pad however its motor runs.
    assert!(!s.rumble.is_open(), "the table names no test system");
    for _ in 0..3 {
        assert_eq!(frame(&mut pads), (0, 0));
    }

    // Opened as a console whose games drive a motor: each frame's level, `frames × 997` on port 0's strong motor.
    s.rumble.set_gate(true, true);
    let first = frame(&mut pads);
    assert!(first.0 != 0 && first.1 == 0, "{first:?}");
    for i in 1..3u16 {
        assert_eq!(frame(&mut pads), (first.0.wrapping_add(997 * i), 0));
    }
    assert_eq!(s.rumble.level(1), (0, 0), "port 1 declares no motor");

    // The player's switch turned off while the game runs: the next frame's level reaches the pad as nothing; on again, the motor's own.
    s.rumble.set_switch(false);
    assert_eq!(frame(&mut pads), (0, 0), "switched off mid-game");
    s.rumble.set_switch(true);
    let back = frame(&mut pads);
    assert!(back.0 != 0, "{back:?}");

    // The switch is read from appsettings.json, on by default; off, no console's motor passes.
    assert!(emusen_crystal::rumble::switch(&tree()));
    std::fs::create_dir_all(root.join("Config")).unwrap();
    std::fs::write(root.join("Config/appsettings.json"), br#"{"Rumble":false}"#).unwrap();
    let off = emusen_crystal::rumble::switch(&tree());
    assert!(!off);
    assert!(!emusen_crystal::rumble::allowed("n64", off) && emusen_crystal::rumble::allowed("n64", true));
}
