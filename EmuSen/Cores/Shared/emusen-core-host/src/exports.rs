//! Every `emusen_core_*` export, typed as `emusen_core.h` types it and resolved by name; the optional ones `None` where
//! the library lacks them. Moved from the conformance kit, which still calls through it (EmuSen_Backend.md §27.4).

use std::ffi::c_char;

use emusen_native::core::sys::{CreateParams, DeviceFrame, Event, FrameInfo, Machine as RawMachine, Media, RawFrame};

/// A machine as the exports take it.
pub type M = *mut RawMachine;
type C = *const RawMachine;

macro_rules! table {
    (required { $($r:ident : fn($($ra:ty),*) -> $rr:ty),* $(,)? } optional { $($o:ident : fn($($oa:ty),*) -> $or:ty),* $(,)? }) => {
        /// Every export, the optional ones `None` where the library lacks them.
        #[allow(non_snake_case)]
        pub struct Exports {
            $( pub $r: unsafe extern "C" fn($($ra),*) -> $rr, )*
            $( pub $o: Option<unsafe extern "C" fn($($oa),*) -> $or>, )*
        }

        impl Exports {
            /// The table, or the name of the first required export missing.
            pub(crate) unsafe fn resolve(lib: &libloading::Library) -> Result<Exports, String> {
                Ok(Exports {
                    $( $r: unsafe { *lib.get::<unsafe extern "C" fn($($ra),*) -> $rr>(concat!("emusen_core_", stringify!($r), "\0").as_bytes()).map_err(|_| concat!("emusen_core_", stringify!($r)).to_owned())? }, )*
                    $( $o: unsafe { lib.get::<unsafe extern "C" fn($($oa),*) -> $or>(concat!("emusen_core_", stringify!($o), "\0").as_bytes()).ok().map(|s| *s) }, )*
                })
            }
        }
    };
}

table! {
    required {
        abi_version: fn() -> u32,
        capabilities: fn() -> u64,
        info: fn(*mut u8, usize) -> i64,
        settings_schema: fn(*mut u8, usize) -> i64,
        firmware_for: fn(*const u8, usize, *mut u8, usize) -> i64,
        status_text: fn(i32, *mut u8, usize) -> i64,
        set_crash_log: fn(*const c_char) -> i32,
        log_drain: fn(M, *mut u8, usize) -> i64,
        create: fn(*const CreateParams, *mut i32) -> M,
        free: fn(M) -> i32,
        machine_info: fn(C, *mut u8, usize) -> i64,
        last_error: fn(C, *mut u8, usize) -> i64,
        advance: fn(M, *mut u64) -> i32,
        set_options: fn(M, u32) -> i32,
        frame_count: fn(C) -> i64,
        events: fn(M, *mut Event, usize, usize) -> i64,
        frame_info: fn(C, *mut FrameInfo) -> i32,
        frame_copy: fn(C, *mut u8, usize) -> i64,
        audio_rate: fn(C) -> i32,
        audio_buffered: fn(C) -> i64,
        audio_drain: fn(M, *mut i16, usize, i64, *mut i32) -> i64,
        set_audio_limit: fn(M, u64) -> i32,
        set_buttons: fn(M, u32, u32, u32) -> i32,
        state_size: fn(C, u32) -> i64,
        state_save: fn(M, u32, *mut u8, usize) -> i64,
        state_load: fn(M, *const u8, usize) -> i32,
        state_layout: fn(C, u32, *mut u8, usize) -> i64,
        space_size: fn(C, u32) -> i64,
        space_read: fn(M, u32, u32, *mut u8, usize) -> i64,
        space_write: fn(M, u32, u32, *const u8, usize) -> i64,
        battery: fn(C, u32, *mut u8, usize, *mut u32) -> i64,
        battery_saved: fn(M, u32) -> i32,
    }
    optional {
        reset: fn(M) -> i32,
        present: fn(M) -> i32,
        phases: fn(C, *mut i64, usize) -> i64,
        audio_peek: fn(C, *mut i16, usize) -> i64,
        set_mutes: fn(M, u32) -> i32,
        set_axis: fn(M, u32, u32, f64) -> i32,
        set_rom_patches: fn(M, *const u32, usize) -> i64,
        set_cheat_pokes: fn(M, *const u32, usize) -> i64,
        set_settings: fn(M, *const u8, usize) -> i32,
        setting_notes: fn(C, *mut u8, usize) -> i64,
        debug_set: fn(M, u32, i32, i32) -> i32,
        debug_set_stack: fn(M, *const u32, usize) -> i32,
        debug_set_breakpoints: fn(M, u32, *const i32, usize) -> i32,
        debug_set_ranges: fn(M, u32, *const u32, usize) -> i32,
        debug_run_frame: fn(M, u32, *mut u32, *mut u64, *mut u64) -> i32,
        debug_writes: fn(M, *mut u32, usize) -> i64,
        debug_calls: fn(M, *mut u32, usize) -> i64,
        debug_profile: fn(M, *mut i64, usize) -> i64,
        debug_coverage: fn(M, u32, *mut u8, usize, *mut i64) -> i64,
        debug_counters: fn(M, *mut i64, usize) -> i64,
        debug_pc: fn(C, u32, *mut u64) -> i32,
        debug_registers: fn(C, u32, *mut i64, usize) -> i64,
        debug_disassemble: fn(M, u32, u32, u32, u32, *mut u8, usize) -> i64,
        media_lid: fn(M, u32, u32) -> i32,
        media_insert: fn(M, u32, *const Media) -> i32,
        debug_reads: fn(M, *mut u32, usize) -> i64,
        frame_device: fn(M, *mut DeviceFrame) -> i32,
        raw_select: fn(M, u64) -> i32,
        raw_info: fn(C, u32, *mut RawFrame) -> i32,
        raw_copy: fn(C, u32, *mut u8, usize) -> i64,
        telemetry: fn(C, *mut i64, usize) -> i64,
        set_present: fn(M, u32, u32) -> i32,
        debug_translate: fn(C, u32, u32, *mut u32) -> i32,
    }
}
