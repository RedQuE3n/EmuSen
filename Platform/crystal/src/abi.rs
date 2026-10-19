//! `emusen_crystal.h`'s numbers and structs, field for field; `tests/crystal_abi.rs` holds the two equal and both to
//! the baseline.

#![allow(non_camel_case_types)]

use std::ffi::c_void;

pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 3;
pub const VERSION: u32 = (MAJOR << 16) | MINOR;

pub mod status {
    pub const NO_ENGINE: i32 = -1109;
    pub const FIRMWARE: i32 = -1110;
    pub const BUSY: i32 = -1111;
    pub const BAD_STRUCT: i32 = -1112;
    pub const IO: i32 = -1113;
    pub const STATE_REFUSED: i32 = -1114;
    /// Since 1.3.
    pub const NOT_APPROVED: i32 = -1115;
}

/// Since 1.2: config's flag listing cores still in development.
pub const CONFIG_DEVELOPMENT: u32 = 1;

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
#[derive(Clone, Copy, Debug)]
pub struct emusen_crystal_config {
    pub size: u32,
    pub flags: u32,
    /// Since 1.2: the host's loaded SQLite, lent for the runtime's life.
    pub sqlite: *mut c_void,
    /// Since 1.2: the cores directories as a JSON array of paths; null for the defaults.
    pub cores: *const u8,
    pub cores_len: usize,
    /// Since 1.3: the player's own cores directories as a JSON array of paths; null for the default.
    pub player_cores: *const u8,
    pub player_cores_len: usize,
}

/// The config's size in 1.2, the least that holds `sqlite` and `cores`.
pub const CONFIG_SIZE_12: usize = 32;

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
    /// Since 1.1: the presenter's Vulkan device, the platform library's `emusen_serenity_device`.
    pub device: *const c_void,
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
    /// Since 1.1.
    pub picture_on_device: u32,
    pub reserved: u32,
}

/// Since 1.1: the shape of `emusen_crystal_raw_copy`'s picture.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct emusen_crystal_raw_info {
    pub size: u32,
    pub element: u32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub reserved: u32,
}

/// The session service's table as the header names it, which `emusen_crystal_session_service` returns; its layout is
/// `emusen_session.h`'s and is checked against that header by the session service's own tests.
pub type emusen_session_v1 = c_void;
