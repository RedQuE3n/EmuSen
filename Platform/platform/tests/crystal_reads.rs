//! What the frontend API answers since 2.0 for what emusen_platform.h answered before: the systems table's rows whole,
//! as emusen-systems writes them, and the cheat codecs' catalogue, decoder and encoder (EmuSen_RustPlatform.md §47.24).

use emusen_crystal::abi::{emusen_crystal_config, emusen_crystal_runtime};
use emusen_galaxia::tree::{Override, tree_mut};
use emusen_platform::crystal::{emusen_crystal_cheat_codec_list, emusen_crystal_cheat_decode, emusen_crystal_cheat_encode, emusen_crystal_free, emusen_crystal_last_error, emusen_crystal_new, emusen_crystal_systems};
use emusen_session_service::abi::status;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// The tree's overrides are the process's: one test at a time sets them.
static TREE: Mutex<()> = Mutex::new(());

/// A runtime with no cores directories, so that its systems are the table's alone.
fn runtime(name: &str) -> (*mut emusen_crystal_runtime, PathBuf, MutexGuard<'static, ()>) {
    let held = TREE.lock().unwrap_or_else(|p| p.into_inner());
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("platform-crystal-reads-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    {
        let mut t = tree_mut();
        t.set_override(Override::Data, Some(root.join("Home")));
        t.set_override(Override::Config, Some(root.join("Config")));
    }
    let none = "[]";
    let config = emusen_crystal_config {
        size: std::mem::size_of::<emusen_crystal_config>() as u32,
        flags: 0,
        sqlite: std::ptr::null_mut(),
        cores: none.as_ptr(),
        cores_len: none.len(),
        player_cores: none.as_ptr(),
        player_cores_len: none.len(),
    };
    let rt = unsafe { emusen_crystal_new(&config) };
    assert!(!rt.is_null());
    (rt, root, held)
}

fn call(f: impl Fn(*mut u8, usize) -> i64) -> Result<String, (i64, String)> {
    let n = f(std::ptr::null_mut(), 0);
    if n < 0 {
        let mut words = vec![0u8; 512];
        let w = unsafe { emusen_crystal_last_error(std::ptr::null_mut(), words.as_mut_ptr(), words.len()) };
        return Err((n, String::from_utf8_lossy(&words[..w as usize]).into_owned()));
    }
    let mut out = vec![0u8; n as usize];
    assert_eq!(f(out.as_mut_ptr(), out.len()), n);
    Ok(String::from_utf8(out).unwrap())
}

#[test]
fn every_row_holds_the_table_s_keys_as_emusen_systems_writes_them_then_the_runtime_s() {
    let (rt, root, _held) = runtime("systems");
    let rows: Vec<Value> = serde_json::from_str(&call(|o, l| unsafe { emusen_crystal_systems(rt, o, l) }).unwrap()).unwrap();
    let table = emusen_systems::table().unwrap();
    assert_eq!(rows.len(), table.len(), "no cores, so the table's systems alone");
    for (row, system) in rows.iter().zip(table) {
        let Value::Object(row) = row else { panic!("a row is an object") };
        let names: Vec<&str> = row.keys().map(String::as_str).collect();
        let mut expected: Vec<&str> = emusen_systems::KEYS.to_vec();
        expected.extend(["engines", "starts_empty", "core", "controllers"]);
        assert_eq!(names, expected, "{}", system.id);
        let Value::Object(written) = serde_json::to_value(system).unwrap() else { panic!("a system is an object") };
        for (key, value) in written.iter().filter(|(k, _)| *k != "look") {
            assert_eq!(&row[key], value, "{}.{key}", system.id);
        }
        assert!(row["look"].is_object(), "{}: a look always, the table's or the one computed", system.id);
    }
    assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_codecs_are_listed_decoded_and_encoded_as_emusen_systems_answers() {
    let (rt, root, _held) = runtime("codecs");
    assert_eq!(call(|o, l| unsafe { emusen_crystal_cheat_codec_list(rt, o, l) }).unwrap(), emusen_systems::cheats::codecs_json());

    let (codec, code) = ("snes.gg", "DD62-6DAD");
    let decoded = call(|o, l| unsafe { emusen_crystal_cheat_decode(rt, codec.as_ptr(), codec.len(), code.as_ptr(), code.len(), o, l) }).unwrap();
    assert_eq!(decoded, emusen_systems::cheats::answer_json(&emusen_systems::cheats::decode(codec, code).unwrap()));
    let missing = "no.such";
    assert_eq!(call(|o, l| unsafe { emusen_crystal_cheat_decode(rt, missing.as_ptr(), missing.len(), code.as_ptr(), code.len(), o, l) }), Err((status::ABSENT as i64, "no codec is named no.such".into())));

    let encoded = call(|o, l| unsafe { emusen_crystal_cheat_encode(rt, codec.as_ptr(), codec.len(), 0x7E0DBF, 0x09, o, l) }).unwrap();
    assert_eq!(encoded, emusen_systems::snes::cheats::encode_game_genie(0x7E0DBF, 0x09));
    let raw = "nes.raw";
    assert_eq!(call(|o, l| unsafe { emusen_crystal_cheat_encode(rt, raw.as_ptr(), raw.len(), 0, 0, o, l) }), Err((status::ABSENT as i64, "the codec nes.raw has no encoder".into())));
    assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
    let _ = std::fs::remove_dir_all(&root);
}
