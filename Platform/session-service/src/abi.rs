//! `emusen_session.h` as Rust lays it out: the table `emusen_session_v1`, its statuses and its open enumerations. The
//! header in `Platform/crystal/include/` is normative; `tests/header.rs` holds every offset and number here to it. See
//! EmuSen_DianaSh.md §9 and §13.6.

#![allow(non_camel_case_types)]

use std::ffi::c_void;

pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 4;
pub const VERSION: u32 = (MAJOR << 16) | MINOR;

/// A processor argument naming none: the session's own registries, as a command that names no processor reaches
/// them (since 1.1).
pub const PROCESSOR_SESSION: u32 = u32::MAX;

/// The name a module asks the shell for.
pub const SERVICE: &str = "emusen.session";

/// The service's statuses: the project's two shared codes, then the band −1100 to −1108.
pub mod status {
    pub const NULL: i32 = -1;
    pub const BAD_STRING: i32 = -5;
    pub const NO_SESSION: i32 = -1100;
    pub const NO_SUCH_SPACE: i32 = -1101;
    pub const READ_ONLY: i32 = -1102;
    pub const NOT_SUPPORTED: i32 = -1103;
    pub const BAD_ARGUMENT: i32 = -1104;
    pub const NO_SUCH_ID: i32 = -1105;
    pub const PARSE: i32 = -1106;
    pub const ABSENT: i32 = -1107;
    pub const FAULT: i32 = -1108;
}

pub mod run {
    pub const PAUSE: u32 = 0;
    pub const RESUME: u32 = 1;
    pub const STEP: u32 = 2;
    pub const STEP_OVER: u32 = 3;
    pub const STEP_OUT: u32 = 4;
    pub const TO_ADDRESS: u32 = 5;
    pub const TO_INTERRUPT: u32 = 6;
    pub const TO_SCANLINE: u32 = 7;
    pub const TO_FRAME: u32 = 8;
}

/// A call-stack frame's kind, and the interrupts `run_control` runs to.
pub mod frame {
    pub const CALL: u32 = 0;
    pub const NMI: u32 = 1;
    pub const IRQ: u32 = 2;
    pub const BRK: u32 = 3;
    pub const COP: u32 = 4;
}

pub mod watch {
    pub const WRITE: u32 = 0;
    pub const READ: u32 = 1;
    pub const BOTH: u32 = 2;
}

pub mod condition {
    pub const LOG_ONLY: u32 = 1;
    pub const DISARM: u32 = 2;
}

pub mod coverage {
    pub const ARM: u32 = 0;
    pub const DISARM: u32 = 1;
    pub const CLEAR: u32 = 2;
    pub const MARK: u32 = 3;
    pub const CLEAR_MARK: u32 = 4;
    pub const STATUS: u32 = 5;
    pub const QUERY: u32 = 6;
    pub const NEW_SINCE_MARK: u32 = 7;
    pub const ENTRY_POINTS: u32 = 8;
    pub const EXPORT: u32 = 9;
    pub const IMPORT: u32 = 10;
    pub const MERGE: u32 = 11;
}

pub mod profile {
    pub const ARM: u32 = 0;
    pub const DISARM: u32 = 1;
    pub const CLEAR: u32 = 2;
    pub const STATUS: u32 = 3;
    pub const HOTTEST: u32 = 4;
    /// Forgets the live call chain (since 1.1).
    pub const RESET_STACK: u32 = 5;
}

pub mod counters {
    pub const ARM: u32 = 0;
    pub const DISARM: u32 = 1;
    pub const CLEAR: u32 = 2;
    pub const STATUS: u32 = 3;
    pub const AT: u32 = 4;
    pub const TOTALS: u32 = 5;
    pub const HOTTEST: u32 = 6;
    pub const UNTOUCHED: u32 = 7;
}

pub mod dma_log {
    pub const ARM: u32 = 0;
    pub const DISARM: u32 = 1;
    pub const CLEAR: u32 = 2;
    pub const STATUS: u32 = 3;
    pub const TAIL: u32 = 4;
    pub const CHANNELS: u32 = 5;
    pub const DESTINATIONS: u32 = 6;
}

pub mod flow {
    pub const ARM: u32 = 0;
    pub const DISARM: u32 = 1;
    pub const CLEAR: u32 = 2;
    pub const STATUS: u32 = 3;
    pub const TAIL: u32 = 4;
    pub const STATS: u32 = 5;
}

pub mod cheat {
    pub const LIST: u32 = 0;
    pub const ADD: u32 = 1;
    pub const POKE: u32 = 2;
    pub const ROM_PATCH: u32 = 3;
    pub const ENABLE: u32 = 4;
    pub const DISABLE: u32 = 5;
    pub const REMOVE: u32 = 6;
    pub const CLEAR: u32 = 7;
    pub const MASTER: u32 = 8;
    pub const EXPORT: u32 = 9;
    pub const IMPORT: u32 = 10;
    pub const DECODE: u32 = 11;
    pub const DB_SYSTEMS: u32 = 12;
    pub const DB_GAMES: u32 = 13;
    pub const DB_FIND: u32 = 14;
    pub const CLAIMS: u32 = 15;
    pub const CHT_PARSE: u32 = 16;
    pub const CHT_EXPORT: u32 = 17;
    pub const FILES: u32 = 18;
    pub const FILE_READ: u32 = 19;
    pub const FILE_WRITE: u32 = 20;
    pub const DB_STATUS: u32 = 21;
    pub const DB_PARSE: u32 = 22;
    pub const DB_PRUNE: u32 = 23;
    pub const DB_UPDATE: u32 = 24;
}

pub mod telemetry {
    pub const CPU: u32 = 0;
    pub const VIDEO: u32 = 1;
    pub const APU: u32 = 2;
    pub const COPROCESSOR: u32 = 3;
    pub const SPRITES: u32 = 4;
    pub const PALETTES: u32 = 5;
    pub const AUDIO_CHANNELS: u32 = 6;
    pub const HARDWARE_LOAD: u32 = 7;
}

pub mod event {
    pub const RUN: u32 = 1;
}

type Ctx = *mut c_void;
type Out = *mut u8;
type In = *const u8;

/// The table, in the header's order; every function may be null in a provider that lacks it.
#[repr(C)]
pub struct emusen_session_v1 {
    pub size: u32,
    pub version: u32,
    pub ctx: Ctx,
    pub last_error: Option<unsafe extern "C" fn(Ctx, Out, usize) -> i64>,

    pub sessions: Option<unsafe extern "C" fn(Ctx, Out, usize) -> i64>,
    pub info: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub summary: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,

    pub run_control: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, i64) -> i64>,
    pub halt_state: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,

    pub spaces: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub space_read: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, Out, usize) -> i64>,
    pub space_write: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, In, usize) -> i32>,
    pub resolve_physical: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,

    pub processors: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub registers: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub set_register: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, u64) -> i32>,

    pub disassemble: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, u32, u32, In, usize, Out, usize) -> i64>,

    pub bp_add: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize) -> i64>,
    pub bp_add_data: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize) -> i64>,
    pub bp_set_enabled: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, u32) -> i32>,
    pub bp_remove: Option<unsafe extern "C" fn(Ctx, u32, u32, u32) -> i32>,
    pub bp_list: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub bp_log: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, Out, usize) -> i64>,
    pub bp_log_clear: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,
    pub bp_arm_condition: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, u32) -> i32>,
    pub bp_conditions: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub bp_depth_guard: Option<unsafe extern "C" fn(Ctx, u32, u32, i32) -> i32>,
    pub bp_forbid_range: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, u32) -> i64>,
    pub bp_uninitialized: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, u32) -> i32>,

    pub watch_add: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u32, u32) -> i64>,
    pub watch_remove: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,
    pub watch_list: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub watch_events: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, Out, usize) -> i64>,
    pub watch_summary: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub watch_clear: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,
    pub framelog_add: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u32) -> i64>,
    pub framelog_remove: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,
    pub framelog_list: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub framelog_samples: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, Out, usize) -> i64>,
    pub framelog_clear: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,

    pub coverage_op: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, In, usize, Out, usize) -> i64>,
    pub profile_op: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, In, usize, Out, usize) -> i64>,
    pub counters_op: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,
    pub call_stack: Option<unsafe extern "C" fn(Ctx, u32, u32, u32, Out, usize) -> i64>,

    pub freeze_add: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u32) -> i64>,
    pub freeze_remove: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,
    pub freeze_list: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub freeze_clear: Option<unsafe extern "C" fn(Ctx, u32) -> i32>,
    pub label_add: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, In, usize) -> i32>,
    pub label_remove: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32) -> i32>,
    pub label_list: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub label_at: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub label_load: Option<unsafe extern "C" fn(Ctx, u32, In, usize, Out, usize) -> i64>,
    pub label_save: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub label_clear: Option<unsafe extern "C" fn(Ctx, u32) -> i32>,

    pub dma_channels: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub dma_log_op: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,
    pub register_flow_op: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,
    pub coprocessor_history: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub vectors: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub tilemap_entry: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, Out, usize) -> i64>,
    pub tile_pixels: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u32, Out, usize) -> i64>,
    pub sprites: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub palettes: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub audio_channels: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub set_channel_muted: Option<unsafe extern "C" fn(Ctx, u32, u32, u32) -> i32>,

    pub evaluate: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,
    pub symbols: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,

    pub cheat_op: Option<unsafe extern "C" fn(Ctx, u32, u32, In, usize, Out, usize) -> i64>,
    pub cheat_codecs: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,

    pub trace: Option<unsafe extern "C" fn(Ctx, u32, u32, u32) -> i32>,
    pub trace_off: Option<unsafe extern "C" fn(Ctx, u32, u32) -> i32>,

    pub telemetry: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,
    pub tile_sheet: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub palette_swatch: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,
    pub audio_peek: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,

    pub state_save: Option<unsafe extern "C" fn(Ctx, u32, In, usize) -> i32>,
    pub state_load: Option<unsafe extern "C" fn(Ctx, u32, In, usize) -> i32>,
    pub state_default_path: Option<unsafe extern "C" fn(Ctx, u32, Out, usize) -> i64>,

    pub log_switch: Option<unsafe extern "C" fn(Ctx, u32) -> i32>,
    pub log_status: Option<unsafe extern "C" fn(Ctx) -> i32>,

    pub events_drain: Option<unsafe extern "C" fn(Ctx, u32, u32, Out, usize) -> i64>,

    pub games: Option<unsafe extern "C" fn(Ctx, Out, usize) -> i64>,
    pub open_game: Option<unsafe extern "C" fn(Ctx, u32, In, usize, In, usize) -> i64>,
    pub open_console: Option<unsafe extern "C" fn(Ctx, u32, In, usize, In, usize) -> i64>,
    pub freeze_add_wide: Option<unsafe extern "C" fn(Ctx, u32, In, usize, u32, u64, u32, u32) -> i64>,
}

/// The offset of each field, by name, for the test that holds them to a C compiler's.
pub fn offsets() -> Vec<(&'static str, usize)> {
    macro_rules! at {
        ($($f:ident),*) => { vec![$((stringify!($f), std::mem::offset_of!(emusen_session_v1, $f))),*] };
    }
    at!(size, version, ctx, last_error, sessions, info, summary, run_control, halt_state, spaces, space_read, space_write,
        resolve_physical, processors, registers, set_register, disassemble, bp_add, bp_add_data, bp_set_enabled, bp_remove,
        bp_list, bp_log, bp_log_clear, bp_arm_condition, bp_conditions, bp_depth_guard, bp_forbid_range, bp_uninitialized,
        watch_add, watch_remove, watch_list, watch_events, watch_summary, watch_clear, framelog_add, framelog_remove,
        framelog_list, framelog_samples, framelog_clear, coverage_op, profile_op, counters_op, call_stack, freeze_add,
        freeze_remove, freeze_list, freeze_clear, label_add, label_remove, label_list, label_at, label_load, label_save,
        label_clear, dma_channels, dma_log_op, register_flow_op, coprocessor_history, vectors, tilemap_entry, tile_pixels,
        sprites, palettes, audio_channels, set_channel_muted, evaluate, symbols, cheat_op, cheat_codecs, trace, trace_off,
        telemetry, tile_sheet, palette_swatch, audio_peek, state_save, state_load, state_default_path, log_switch,
        log_status, events_drain, games, open_game, open_console, freeze_add_wide)
}

/// # Safety: If non-null, `out` must be valid for writes of at least `len.min(src.len())` bytes.
pub unsafe fn copy_answer(src: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() && len >= src.len() {
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), out, src.len()) };
    }
    src.len() as i64
}

/// Returns a zeroed `emusen_session_v1` table at `VERSION` with all function pointers set to `None`.
pub const fn blank_table() -> emusen_session_v1 {
    emusen_session_v1 {
        size: std::mem::size_of::<emusen_session_v1>() as u32,
        version: VERSION,
        ctx: std::ptr::null_mut(),
        last_error: None,
        sessions: None,
        info: None,
        summary: None,
        run_control: None,
        halt_state: None,
        spaces: None,
        space_read: None,
        space_write: None,
        resolve_physical: None,
        processors: None,
        registers: None,
        set_register: None,
        disassemble: None,
        bp_add: None,
        bp_add_data: None,
        bp_set_enabled: None,
        bp_remove: None,
        bp_list: None,
        bp_log: None,
        bp_log_clear: None,
        bp_arm_condition: None,
        bp_conditions: None,
        bp_depth_guard: None,
        bp_forbid_range: None,
        bp_uninitialized: None,
        watch_add: None,
        watch_remove: None,
        watch_list: None,
        watch_events: None,
        watch_summary: None,
        watch_clear: None,
        framelog_add: None,
        framelog_remove: None,
        framelog_list: None,
        framelog_samples: None,
        framelog_clear: None,
        coverage_op: None,
        profile_op: None,
        counters_op: None,
        call_stack: None,
        freeze_add: None,
        freeze_remove: None,
        freeze_list: None,
        freeze_clear: None,
        label_add: None,
        label_remove: None,
        label_list: None,
        label_at: None,
        label_load: None,
        label_save: None,
        label_clear: None,
        dma_channels: None,
        dma_log_op: None,
        register_flow_op: None,
        coprocessor_history: None,
        vectors: None,
        tilemap_entry: None,
        tile_pixels: None,
        sprites: None,
        palettes: None,
        audio_channels: None,
        set_channel_muted: None,
        evaluate: None,
        symbols: None,
        cheat_op: None,
        cheat_codecs: None,
        trace: None,
        trace_off: None,
        telemetry: None,
        tile_sheet: None,
        palette_swatch: None,
        audio_peek: None,
        state_save: None,
        state_load: None,
        state_default_path: None,
        log_switch: None,
        log_status: None,
        events_drain: None,
        games: None,
        open_game: None,
        open_console: None,
        freeze_add_wide: None,
    }
}
