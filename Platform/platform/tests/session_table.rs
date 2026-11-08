//! The session service's one surface, the table `emusen_crystal_session_service` hands out, as a frontend calls it once
//! the twins are gone (frontend API 2.0): the table with no game sets the process's log switch, a read-only one refuses
//! to, and the runtime's own listing of sessions stays beside it (EmuSen_RustPlatform.md §47.23).

use emusen_crystal::abi::{SERVICE_NO_GAME, SERVICE_READ_ONLY, emusen_crystal_config};
use emusen_crystal::service::READ_ONLY_WORDS;
use emusen_galaxia::tree::{Override, tree_mut};
use emusen_platform::crystal::{emusen_crystal_free, emusen_crystal_new, emusen_crystal_sessions};
use emusen_platform::session_table::emusen_crystal_session_service;
use emusen_session_service::abi::{emusen_session_v1, status};
use std::path::PathBuf;

fn words(t: &emusen_session_v1) -> String {
    let mut out = vec![0u8; 512];
    let n = unsafe { (t.last_error.unwrap())(t.ctx, out.as_mut_ptr(), out.len()) };
    String::from_utf8_lossy(&out[..n.max(0) as usize]).into_owned()
}

#[test]
fn the_table_with_no_game_sets_the_log_switch_and_a_read_only_one_refuses() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("platform-session-table-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("cores")).unwrap();
    {
        let mut t = tree_mut();
        t.set_override(Override::Data, Some(root.join("Home")));
        t.set_override(Override::Config, Some(root.join("Config")));
    }
    let cores = serde_json::to_string(&[root.join("cores")]).unwrap();
    let config = emusen_crystal_config {
        size: std::mem::size_of::<emusen_crystal_config>() as u32,
        flags: 0,
        sqlite: std::ptr::null_mut(),
        cores: cores.as_ptr(),
        cores_len: cores.len(),
        player_cores: std::ptr::null(),
        player_cores_len: 0,
    };
    let rt = unsafe { emusen_crystal_new(&config) };
    assert!(!rt.is_null());

    let mut listed = vec![0u8; 64];
    let n = unsafe { emusen_crystal_sessions(rt, listed.as_mut_ptr(), listed.len()) };
    assert_eq!(&listed[..n as usize], b"[]", "the runtime's listing, with nothing open");

    let full = unsafe { &*emusen_crystal_session_service(rt, 0, SERVICE_NO_GAME) };
    let read_only = unsafe { &*emusen_crystal_session_service(rt, 0, SERVICE_NO_GAME | SERVICE_READ_ONLY) };
    let (switch, status_of) = (full.log_switch.unwrap(), full.log_status.unwrap());
    let was = unsafe { status_of(full.ctx) };
    assert_eq!(unsafe { switch(full.ctx, 1) }, 0, "{}", words(full));
    assert_eq!(unsafe { status_of(full.ctx) }, 1);
    assert_eq!(unsafe { (read_only.log_switch.unwrap())(read_only.ctx, 0) }, status::READ_ONLY);
    assert_eq!(words(read_only), READ_ONLY_WORDS);
    assert_eq!(unsafe { (read_only.log_status.unwrap())(read_only.ctx) }, 1, "the read-only table reads the switch");
    assert_eq!(unsafe { switch(full.ctx, 0) }, 0);
    assert_eq!(unsafe { status_of(full.ctx) }, 0);
    unsafe { switch(full.ctx, was as u32) };

    assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
    let _ = std::fs::remove_dir_all(&root);
}
