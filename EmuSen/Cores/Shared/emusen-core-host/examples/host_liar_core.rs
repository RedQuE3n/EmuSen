//! A core with the 32 required exports and descriptors that lie as `liar_set` asks, so that each of rule 4's refusals
//! can be shown (EmuSen_CoreAPI.md §7.2). Each copy at its own path is its own library with its own mode.

use std::sync::atomic::{AtomicU32, Ordering};

static MODE: AtomicU32 = AtomicU32::new(0);

pub const GOOD: u32 = 0;
pub const MAJOR_2: u32 = 1;
pub const CLAIMS_RESET: u32 = 2;
pub const ABI_SAYS_13: u32 = 3;
pub const UNKNOWN_OBLIGATION: u32 = 4;
pub const INFO_NOT_UTF8: u32 = 5;
pub const INFO_HUGE: u32 = 6;
pub const INFO_NOT_JSON: u32 = 7;
pub const NO_LICENSE: u32 = 8;
pub const INFO_OFF_SCHEMA: u32 = 9;
pub const SETTINGS_NOT_JSON: u32 = 10;
pub const SETTINGS_OFF_SCHEMA: u32 = 11;
pub const BAD_ID: u32 = 12;

#[unsafe(no_mangle)]
pub extern "C" fn liar_set(mode: u32) {
    MODE.store(mode, Ordering::Relaxed);
}

fn mode() -> u32 {
    MODE.load(Ordering::Relaxed)
}

unsafe fn copy(text: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() && len >= text.len() {
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len()) };
    }
    text.len() as i64
}

fn info() -> Vec<u8> {
    let field = |k: &str, v: &str| format!("\"{k}\":\"{v}\",");
    let abi = if mode() == ABI_SAYS_13 { "1.13" } else { "1.15" };
    let id = if mode() == BAD_ID { "Liar" } else { "liar-core" };
    let license = if mode() == NO_LICENSE { String::new() } else { field("license", "GPL-3.0-or-later") };
    let systems = if mode() == INFO_OFF_SCHEMA { "5" } else { r#"[{"id":"liar","name":"Liar","extensions":[".liar"]}]"# };
    let requires = if mode() == UNKNOWN_OBLIGATION { r#","host_requires":["teleport"]"# } else { "" };
    let mut text = format!("{{{}{}{}{license}\"systems\":{systems},\"capabilities\":[]{requires}}}", field("abi", abi), field("id", id), field("name", "Liar") + &field("version", "1.0.0")).into_bytes();
    match mode() {
        INFO_NOT_UTF8 => text.push(0xFF),
        INFO_HUGE => text = vec![b' '; (1 << 20) + 1],
        INFO_NOT_JSON => text.truncate(text.len() - 1),
        _ => {}
    }
    text
}

#[unsafe(no_mangle)]
pub extern "C" fn emusen_core_abi_version() -> u32 {
    if mode() == MAJOR_2 { 2 << 16 } else { (1 << 16) | 15 }
}

#[unsafe(no_mangle)]
pub extern "C" fn emusen_core_capabilities() -> u64 {
    if mode() == CLAIMS_RESET { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_core_info(out: *mut u8, len: usize) -> i64 {
    unsafe { copy(&info(), out, len) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_core_settings_schema(out: *mut u8, len: usize) -> i64 {
    let text: &[u8] = match mode() {
        SETTINGS_NOT_JSON => b"[",
        SETTINGS_OFF_SCHEMA => br#"[{"key":"x"}]"#,
        _ => b"[]",
    };
    unsafe { copy(text, out, len) }
}

macro_rules! refusing {
    ($($name:ident($($a:ty),*) -> $r:ty = $v:expr;)*) => {
        $(
            #[unsafe(no_mangle)]
            pub extern "C" fn $name($(_: $a),*) -> $r { $v }
        )*
    };
}

type P = *mut u8;
refusing! {
    emusen_core_firmware_for(*const u8, usize, P, usize) -> i64 = 2;
    emusen_core_status_text(i32, P, usize) -> i64 = -256;
    emusen_core_set_crash_log(*const u8) -> i32 = 0;
    emusen_core_log_drain(P, P, usize) -> i64 = 0;
    emusen_core_create(*const u8, *mut i32) -> P = std::ptr::null_mut();
    emusen_core_free(P) -> i32 = 0;
    emusen_core_machine_info(P, P, usize) -> i64 = -1;
    emusen_core_last_error(P, P, usize) -> i64 = 0;
    emusen_core_advance(P, *mut u64) -> i32 = -1;
    emusen_core_set_options(P, u32) -> i32 = -1;
    emusen_core_frame_count(P) -> i64 = 0;
    emusen_core_events(P, P, usize, usize) -> i64 = 0;
    emusen_core_frame_info(P, P) -> i32 = -1;
    emusen_core_frame_copy(P, P, usize) -> i64 = 0;
    emusen_core_audio_rate(P) -> i32 = 0;
    emusen_core_audio_buffered(P) -> i64 = 0;
    emusen_core_audio_drain(P, P, usize, i64, *mut i32) -> i64 = 0;
    emusen_core_set_audio_limit(P, u64) -> i32 = 0;
    emusen_core_set_buttons(P, u32, u32, u32) -> i32 = 0;
    emusen_core_state_size(P, u32) -> i64 = -256;
    emusen_core_state_save(P, u32, P, usize) -> i64 = -256;
    emusen_core_state_load(P, *const u8, usize) -> i32 = -256;
    emusen_core_state_layout(P, u32, P, usize) -> i64 = -256;
    emusen_core_space_size(P, u32) -> i64 = -1;
    emusen_core_space_read(P, u32, u32, P, usize) -> i64 = -1;
    emusen_core_space_write(P, u32, u32, *const u8, usize) -> i64 = -1;
    emusen_core_battery(P, u32, P, usize, *mut u32) -> i64 = 0;
    emusen_core_battery_saved(P, u32) -> i32 = 0;
}
