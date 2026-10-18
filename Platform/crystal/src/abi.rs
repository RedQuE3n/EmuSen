//! `emusen_crystal.h`'s numbers and structs, field for field; `tests/crystal_abi.rs` holds the two equal and both to
//! the baseline.

#![allow(non_camel_case_types)]

use std::ffi::c_void;

pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 0;
pub const VERSION: u32 = (MAJOR << 16) | MINOR;

pub mod status {
    pub const NO_ENGINE: i32 = -1109;
    pub const FIRMWARE: i32 = -1110;
    pub const BUSY: i32 = -1111;
    pub const BAD_STRUCT: i32 = -1112;
    pub const IO: i32 = -1113;
}

pub const OPEN_NO_BATTERY: u32 = 1;
pub const OPEN_SINGLE_ROWS: u32 = 2;

pub const SPEED_NORMAL: u32 = 0;
pub const SPEED_TURBO: u32 = 1;
pub const SPEED_SLOW: u32 = 2;
pub const SPEED_DRAW_NONE: u32 = 1;

pub const SCOPE_CONSOLE: u32 = 0;
pub const SCOPE_GAME: u32 = 2;

pub const SERVICE_READ_ONLY: u32 = 1;

pub const EVENT_SESSION: u32 = 1;
pub const EVENT_RUN: u32 = 2;
pub const EVENT_STATE: u32 = 4;
pub const EVENT_REWIND: u32 = 8;
pub const EVENT_SETTINGS: u32 = 16;
pub const EVENT_DISC: u32 = 32;
pub const EVENT_LOG: u32 = 64;

pub const CARDS_LIST: u32 = 0;
pub const CARDS_OPEN: u32 = 1;
pub const CARDS_COPY: u32 = 2;
pub const CARDS_MOVE: u32 = 3;
pub const CARDS_DELETE: u32 = 4;
pub const CARDS_IMPORT: u32 = 5;
pub const CARDS_NEW: u32 = 6;
pub const CARDS_DUPLICATE: u32 = 7;
pub const CARDS_RENAME: u32 = 8;

/// The runtime's handle, never read by a client.
#[repr(C)]
pub struct emusen_crystal_runtime {
    _private: [u8; 0],
}

/// A subscription's handle, never read by a client.
#[repr(C)]
pub struct emusen_crystal_subscription {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct emusen_crystal_config {
    pub size: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct emusen_crystal_open_params {
    pub size: u32,
    pub flags: u32,
    pub path: *const u8,
    pub path_len: usize,
    pub console: *const u8,
    pub console_len: usize,
    pub engine: *const u8,
    pub engine_len: usize,
    pub audio_buffer_max_samples: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct emusen_crystal_frame_info {
    pub size: u32,
    pub ran: u32,
    pub halted: u32,
    pub rendered: u32,
    pub picture_serial: u64,
    pub width: i32,
    pub height: i32,
    pub row_repeat: i32,
    pub display_lines: i32,
    pub display_aspect: f64,
    pub audio_frames: u32,
    pub audio_rate: u32,
    pub play_audio: u32,
    pub speed_percent: u32,
    pub frame_interval_ns: u64,
    pub total_frames: u64,
    pub frame_rate_hz: f64,
    pub paused: u32,
    pub halted_processor: u32,
    pub halted_address: u64,
}

/// The session service's table as the header names it, which `emusen_crystal_session_service` returns; its layout is
/// `emusen_session.h`'s and is checked against that header by the session service's own tests.
pub type emusen_session_v1 = c_void;
