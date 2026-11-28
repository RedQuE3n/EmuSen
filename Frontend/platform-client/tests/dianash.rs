//! End-to-end smoke test for `DianashShell` running cheat search, write, freeze, and cheat promotion.

use emusen_client_fixture::{library, World};
use emusen_platform_client::dianash::{DianashConfig, DianashShell, SubmitResult};
use emusen_platform_client::Platform;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::ffi::c_void;

type Ctx = *mut c_void;
type Out = *mut u8;
type In = *const u8;

#[repr(C)]
struct MockSessionV1 {
    size: u32,
    version: u32,
    ctx: Ctx,
    last_error: Option<unsafe extern "C" fn(Ctx, Out, usize) -> i64>,
    sessions: Option<unsafe extern "C" fn(Ctx, Out, usize) -> i64>,
    info: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    summary: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    run_control: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, i64) -> i64>,
    halt_state: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    spaces: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    space_read: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, Out, usize) -> i64>,
    space_write: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, In, usize) -> i32>,
    resolve_physical: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    processors: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    registers: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    set_register: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, u64) -> i32>,
    disassemble: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, u32, u32, In, usize, Out, usize) -> i64>,
    rest_before_freeze: [usize; 27],
    freeze_add: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u32) -> i64>,
    freeze_remove: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,
    freeze_list: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    freeze_clear: Option<unsafe extern "C" fn(Ctx, u32) -> i32>,
    rest_mid: [usize; 20],
    cheat_op: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,
    cheat_codecs: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    rest_tail: [usize; 15],
    freeze_add_wide: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u64, u32, u32) -> i64>,
}

const _: () = assert!(std::mem::size_of::<MockSessionV1>() == 680);

struct MockState {
    ram: RefCell<Vec<u8>>,
    freezes: RefCell<Vec<(u32, String, u32, u64, u32, bool)>>,
    cheats: RefCell<Vec<Value>>,
}

unsafe fn put(src: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() && len >= src.len() {
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), out, src.len()) };
    }
    src.len() as i64
}

unsafe extern "C" fn mock_info(_c: Ctx, _s: u32, o: Out, l: usize) -> i64 {
    unsafe { put(br#"{"id":1,"engine":"Mock","system":"snes","byte_order":"little","capabilities":["freezes","cheats"]}"#, o, l) }
}

unsafe extern "C" fn mock_spaces(c: Ctx, _s: u32, o: Out, l: usize) -> i64 {
    let st = unsafe { &*(c as *const MockState) };
    let doc = format!(r#"[{{"name":"WRAM","size":{},"writable":true,"side_effects":false,"byte_order":"little"}}]"#, st.ram.borrow().len());
    unsafe { put(doc.as_bytes(), o, l) }
}

unsafe extern "C" fn mock_read(c: Ctx, _s: u32, _sp: In, _sl: usize, addr: u32, o: Out, l: usize) -> i64 {
    let st = unsafe { &*(c as *const MockState) };
    let ram = st.ram.borrow();
    let start = (addr as usize).min(ram.len());
    let n = l.min(ram.len() - start);
    if !o.is_null() && n > 0 {
        unsafe { std::ptr::copy_nonoverlapping(ram[start..].as_ptr(), o, n) };
    }
    l as i64
}

unsafe extern "C" fn mock_write(c: Ctx, _s: u32, _sp: In, _sl: usize, addr: u32, d: In, l: usize) -> i32 {
    let st = unsafe { &*(c as *const MockState) };
    let mut ram = st.ram.borrow_mut();
    let bytes = unsafe { std::slice::from_raw_parts(d, l) };
    for (i, &b) in bytes.iter().enumerate() {
        if let Some(slot) = ram.get_mut(addr as usize + i) {
            *slot = b;
        }
    }
    0
}

unsafe extern "C" fn mock_freeze_add_wide(c: Ctx, _s: u32, sp: In, sl: usize, addr: u32, val: u64, w: u32, be: u32) -> i64 {
    let st = unsafe { &*(c as *const MockState) };
    let space = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(sp, sl)) }.to_string();
    let mut list = st.freezes.borrow_mut();
    let id = (list.len() + 1) as u32;
    list.push((id, space, addr, val, w, be != 0));
    i64::from(id)
}

unsafe extern "C" fn mock_freeze_list(c: Ctx, _s: u32, o: Out, l: usize) -> i64 {
    let st = unsafe { &*(c as *const MockState) };
    let arr: Vec<Value> = st
        .freezes
        .borrow()
        .iter()
        .map(|(id, sp, addr, val, w, be)| json!({"id": id, "space": sp, "address": addr, "value": val, "width": w, "big_endian": be, "blocked": 0}))
        .collect();
    unsafe { put(Value::Array(arr).to_string().as_bytes(), o, l) }
}

unsafe extern "C" fn mock_freeze_remove(c: Ctx, _s: u32, id: u32) -> i32 {
    let st = unsafe { &*(c as *const MockState) };
    st.freezes.borrow_mut().retain(|x| x.0 != id);
    0
}

unsafe extern "C" fn mock_codecs(_c: Ctx, _s: u32, o: Out, l: usize) -> i64 {
    unsafe { put(br#"{"auto":{"name":"Pro Action Replay","kind":"RamPoke","space":"CpuBus"},"explicit":null,"systems":["snes"]}"#, o, l) }
}

unsafe extern "C" fn mock_cheat_op(c: Ctx, _s: u32, op: u32, args: In, args_len: usize, o: Out, l: usize) -> i64 {
    let st = unsafe { &*(c as *const MockState) };
    let text = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(args, args_len)) };
    match op {
        1 => {
            let mut entry: Value = serde_json::from_str(text).unwrap();
            let mut list = st.cheats.borrow_mut();
            let id = (list.len() + 1) as i64;
            entry["id"] = json!(id);
            entry["enabled"] = json!(true);
            list.push(entry);
            id
        }
        0 => {
            let doc = json!({"master": true, "cheats": *st.cheats.borrow()}).to_string();
            unsafe { put(doc.as_bytes(), o, l) }
        }
        _ => -1103,
    }
}

#[test]
fn dianash_shell_drives_cheat_search_freeze_and_promote_on_mock_session() {
    let world = World::new("dianash");
    let platform = Platform::load(library()).unwrap();
    let root_str = world.root.to_string_lossy().into_owned();
    let mut state = MockState {
        ram: RefCell::new(vec![0u8; 256]),
        freezes: RefCell::new(Vec::new()),
        cheats: RefCell::new(Vec::new()),
    };
    let table = MockSessionV1 {
        size: std::mem::size_of::<MockSessionV1>() as u32,
        version: (1 << 16) | 4,
        ctx: (&mut state as *mut MockState).cast(),
        last_error: None,
        sessions: None,
        info: Some(mock_info),
        summary: None,
        run_control: None,
        halt_state: None,
        spaces: Some(mock_spaces),
        space_read: Some(mock_read),
        space_write: Some(mock_write),
        resolve_physical: None,
        processors: None,
        registers: None,
        set_register: None,
        disassemble: None,
        rest_before_freeze: [0; 27],
        freeze_add: None,
        freeze_remove: Some(mock_freeze_remove),
        freeze_list: Some(mock_freeze_list),
        freeze_clear: None,
        rest_mid: [0; 20],
        cheat_op: Some(mock_cheat_op),
        cheat_codecs: Some(mock_codecs),
        rest_tail: [0; 15],
        freeze_add_wide: Some(mock_freeze_add_wide),
    };
    let shell = DianashShell::new(platform, Some(DianashConfig { sessions: false, interactive: false, root: Some(&root_str), budget_ms: 5000 })).unwrap();
    unsafe { shell.attach_session(0, (&table as *const MockSessionV1).cast()) }.unwrap();

    let run_json = |cmd: &str| -> Value {
        let (res, code, out) = shell.run(0, cmd).unwrap();
        assert_eq!(res, SubmitResult::Done);
        assert_eq!(code, 0, "cmd `{cmd}` failed: {out}");
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("cmd `{cmd}` output `{out}`: {e}"))
    };

    let init = run_json("search unknown WRAM -w 2 --dec --json");
    assert_eq!(init["total"], 255);
    run_json("write WRAM 10 100 -w 2 --dec --json");
    let narrowed = run_json("search incby 100 --dec --json");
    assert_eq!(narrowed["total"], 1);
    let hits = run_json("search list 10 --dec --json");
    assert_eq!(hits["hits"][0]["address"], 0x10);
    assert_eq!(hits["hits"][0]["value"], "100");

    let frozen = run_json("freeze add WRAM 10 -w 2 --json");
    assert_eq!(frozen["id"], 1);
    assert_eq!(frozen["value"], 100);

    let promoted = run_json("cheat from-freeze 1 --remove --json Infinite HP");
    assert_eq!(promoted["name"], "Infinite HP");
    assert_eq!(promoted["width"], 2);
    assert!(state.freezes.borrow().is_empty());

    let cheats = run_json("cheat list --json");
    assert_eq!(cheats.as_array().unwrap().len(), 1);
    assert_eq!(cheats[0]["name"], "Infinite HP");
}
