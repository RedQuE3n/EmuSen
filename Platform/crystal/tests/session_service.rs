//! Tests for session service metadata (`system`, `byte_order`), bulk `space_read` parity, and wide freezes.

use emusen_core_host::{sha256, sidecar, trust::MANIFEST};
use emusen_crystal::open::Asked;
use emusen_crystal::runtime::{Config, Runtime};
use emusen_crystal::service::{args::Raw, machine, registries, Call};
use emusen_crystal::session::OpenedWith;
use emusen_galaxia::tree::{tree_mut, Override};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    profile.join("examples").join(format!("lib{name}.so"))
}

fn scratch(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("crystal-svc-{}-{name}", std::process::id()));
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
fn info_spaces_bulk_read_and_wide_freeze_work_over_session_service() {
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
    let game_str = game.to_string_lossy().into_owned();
    let id = rt.open(&Asked { path: Some(&game_str), console: None, engine: None, opened: OpenedWith::default() }).unwrap();
    let s = rt.sessions.get(id).unwrap();
    let call = Call { rt: &rt, s: &s, asked: id, read_only: false };

    let info: Value = serde_json::from_slice(&machine::info(&call).unwrap()).unwrap();
    assert_eq!(info["system"], "test");
    assert_eq!(info["byte_order"], "little");

    let spaces: Value = serde_json::from_slice(&machine::spaces(&call).unwrap()).unwrap();
    let ram = spaces.as_array().unwrap().iter().find(|x| x["name"] == "RAM").unwrap();
    assert_eq!(ram["byte_order"], "little");
    let ram_size = ram["size"].as_u64().unwrap() as usize;

    let mut bulk = vec![0u8; ram_size + 17];
    machine::space_read(&call, Raw::from_slice(b"RAM"), 13, &mut bulk).unwrap();
    for (i, &b) in bulk.iter().enumerate() {
        let mut one = [0u8; 1];
        machine::space_read(&call, Raw::from_slice(b"RAM"), 13 + i as u32, &mut one).unwrap();
        assert_eq!(b, one[0], "mismatch at offset {i}");
    }

    let fz_id = registries::freeze_add_wide(&call, Raw::from_slice(b"RAM"), 0x20, 0x1234_5678, 4, 1).unwrap();
    assert!(fz_id > 0);
    let list: Value = serde_json::from_slice(&registries::freeze_list(&call).unwrap()).unwrap();
    assert_eq!(list[0]["width"], 4);
    assert_eq!(list[0]["big_endian"], true);
    assert_eq!(list[0]["value"], 0x1234_5678);

    drop(s);
    let _ = rt.close(id);
    let _ = std::fs::remove_dir_all(&root);
}
