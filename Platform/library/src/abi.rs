//! `emusen_library.h`'s numbers and its one struct, as the header declares them; `tests/library_abi.rs` holds the two to
//! each other and to `abi/v1/baseline.txt`. See EmuSen_Backend.md §28.4.

use std::ffi::c_void;

pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 2;

/// `(MAJOR << 16) | MINOR`.
pub const VERSION: u32 = (MAJOR << 16) | MINOR;

pub const NULL: i32 = -1;
pub const BAD_STRING: i32 = -5;
pub const NOT_SUPPORTED: i32 = -256;
pub const NEWER_FILE: i32 = -1140;
pub const DATABASE: i32 = -1141;
pub const NO_SQLITE: i32 = -1142;
pub const BAD_REQUEST: i32 = -1143;
pub const CANCELLED: i32 = -1144;
pub const BUSY: i32 = -1145;
pub const IO: i32 = -1146;

/// `emusen_library_config`.
#[repr(C)]
pub struct Config {
    pub size: u32,
    pub database: *const u8,
    pub database_len: usize,
    pub sqlite: *mut c_void,
}
