// Raw FFI for the probe C ABI Mesen gains from probe-c-api.patch, which creates
// Core/Shared/EmuSenProbeApi.cpp - see EmuSen_Debugging_Tools_Reference_v5.md §3.52.
//
// Hand-declared rather than generated, unlike libretro. bindgen is right there
// because libretro.h is a real distro artifact that must not be allowed to drift
// from the installed ABI; here the header would have to live in one repository
// and be consumed by the other, so a generated binding would only move the drift
// somewhere less visible. What guards this instead is PROBE_ABI_VERSION, checked
// once before the first call.
//
// The enum values below are the contract. They must not be renumbered without
// bumping the version on both sides.
use std::os::raw::{c_char, c_int, c_void};

pub const PROBE_ABI_VERSION: u32 = 1;

// Mirrors backend::ProbeButton's declaration order. Read only by the test that
// pins the two orders together, which is the whole of its job.
#[allow(dead_code)]
pub const BUTTON_COUNT: u32 = 12;

// Mirrors backend::ScreenFormat's declaration order.
pub const FORMAT_NONE: c_int = 0;
pub const FORMAT_PALETTE_INDEX16: c_int = 1;
pub const FORMAT_BGR555: c_int = 2;
pub const FORMAT_RGB565: c_int = 3;
pub const FORMAT_XRGB8888: c_int = 4;

// Mirrors backend::TraceKind's declaration order.
pub const TRACE_CPU: c_int = 0;
pub const TRACE_GSU: c_int = 1;
pub const TRACE_APU_WRITES: c_int = 2;

unsafe extern "C" {
    pub fn probe_abi_version() -> u32;

    pub fn probe_set_schedule(entries: *const u32, count: u32);
    pub fn probe_trace_begin(kind: c_int) -> c_int;
    pub fn probe_trace_end(kind: c_int, data: *mut *const u8, size: *mut u32) -> c_int;

    pub fn probe_load(
        rom_path: *const c_char,
        ram_state: *const c_char,
        wav_path: *const c_char,
        home_folder: *const c_char,
    ) -> c_int;
    pub fn probe_shutdown();
    pub fn probe_run_until(frame: u32);
    pub fn probe_frame_count() -> u32;
    pub fn probe_system() -> *const c_char;

    pub fn probe_space_count() -> u32;
    pub fn probe_space_name(index: u32) -> *const c_char;
    pub fn probe_space_data(index: u32) -> *const u8;
    pub fn probe_space_size(index: u32) -> u32;

    pub fn probe_screen(
        data: *mut *const c_void,
        width: *mut u32,
        height: *mut u32,
        bytes: *mut u32,
        format: *mut c_int,
    ) -> c_int;

    pub fn probe_set_button(button: c_int, held: c_int);
    pub fn probe_identity(
        board: *mut *const c_char,
        region: *mut *const c_char,
        prg_bytes: *mut u64,
        chr_bytes: *mut u64,
        save_loaded: *mut c_int,
    );
    pub fn probe_state_line() -> *const c_char;
}
