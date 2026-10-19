//! `emusen_scrape.h`'s numbers and its one struct, as the header declares them. See EmuSen_Backend.md §30.3.

use std::ffi::c_void;

pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 0;
pub const VERSION: u32 = (MAJOR << 16) | MINOR;

pub const NULL: i32 = -1;
pub const BAD_STRING: i32 = -5;
pub const NOT_SUPPORTED: i32 = -256;
pub const NEWER_FILE: i32 = -1160;
pub const DATABASE: i32 = -1161;
pub const NO_SQLITE: i32 = -1162;
pub const BAD_REQUEST: i32 = -1163;
pub const BUSY: i32 = -1164;
pub const IO: i32 = -1165;
pub const NO_LIBRARY: i32 = -1166;

/// `emusen_scrape_config`.
#[repr(C)]
pub struct Config {
    pub size: u32,
    pub root: *const u8,
    pub root_len: usize,
    pub sqlite: *mut c_void,
    pub library: *const c_void,
    pub embedded: *const u8,
    pub embedded_len: usize,
    pub settings: *const u8,
    pub settings_len: usize,
}
