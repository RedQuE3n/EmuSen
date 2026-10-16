//! The structs and numbers of `dianash_module.h`, the command module ABI, and those of `dianash.h`, the host API,
//! as Rust lays them out. The headers in the shell's `include/` are normative, and the platform library's
//! `tests/dianash_abi.rs` holds these to them. See EmuSen_DianaSh.md §6 and §10.3.

#![allow(non_camel_case_types)]

use std::ffi::c_void;

pub const MODULE_ABI_MAJOR: u32 = 1;
pub const MODULE_ABI_MINOR: u32 = 0;
pub const MODULE_ABI_VERSION: u32 = (MODULE_ABI_MAJOR << 16) | MODULE_ABI_MINOR;

/// The statuses of both surfaces: the project's shared codes, then the band −256 to −319, then the fault.
pub mod status {
    pub const OK: i32 = 0;
    pub const NULL: i32 = -1;
    pub const BAD_STRING: i32 = -5;
    pub const BUFFER_TOO_SMALL: i32 = -7;
    pub const NOT_SUPPORTED: i32 = -256;
    pub const BAD_STRUCT: i32 = -263;
    pub const UNKNOWN_COMMAND: i32 = -265;
    pub const SERVICE_MISSING: i32 = -266;
    pub const BUSY: i32 = -267;
    /// An answer, not a failure: no such variable, no such job.
    pub const ABSENT: i32 = -268;
    /// A session of that name exists already.
    pub const EXISTS: i32 = -269;
    /// No session has that id.
    pub const NO_SUCH_SESSION: i32 = -270;
    /// The last session is never killed.
    pub const LAST_SESSION: i32 = -271;
    /// A manifest that is not JSON, or not one this shell reads.
    pub const BAD_MANIFEST: i32 = -272;
    /// A path above the sandbox's root; the words that refuse it are given in its place.
    pub const OUTSIDE: i32 = -273;
    pub const FAULT: i32 = -320;
}

pub const CALL_READ_ONLY_CONTEXT: u32 = 1 << 0;
pub const CALL_INTERACTIVE: u32 = 1 << 1;

pub const RESOLVE_READ: u32 = 1 << 0;
pub const RESOLVE_WRITE: u32 = 1 << 1;
pub const RESOLVE_REMOVE: u32 = 1 << 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct dianash_bytes {
    pub data: *const u8,
    pub len: usize,
}

impl dianash_bytes {
    pub const ABSENT: dianash_bytes = dianash_bytes { data: std::ptr::null(), len: 0 };

    pub fn of(bytes: &[u8]) -> dianash_bytes {
        dianash_bytes { data: bytes.as_ptr(), len: bytes.len() }
    }

    /// # Safety
    /// `data` must be valid for `len` bytes, or null.
    pub unsafe fn slice<'a>(self) -> Option<&'a [u8]> {
        if self.data.is_null() {
            None
        } else if self.len == 0 {
            Some(&[])
        } else {
            Some(unsafe { std::slice::from_raw_parts(self.data, self.len) })
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct dianash_terminal {
    pub size: u32,
    pub columns: u32,
    pub rows: u32,
    pub raw_capable: u32,
}

pub type OutFn = unsafe extern "C" fn(ctx: *mut c_void, data: *const u8, len: usize) -> i32;
pub type ResolveFn = unsafe extern "C" fn(ctx: *mut c_void, path: *const u8, len: usize, flags: u32, out: *mut u8, cap: usize, needed: *mut i64) -> i32;
pub type DisplayFn = unsafe extern "C" fn(ctx: *mut c_void, real: *const u8, len: usize, out: *mut u8, cap: usize, needed: *mut i64) -> i32;
pub type CancelledFn = unsafe extern "C" fn(ctx: *mut c_void) -> i32;
pub type ReleaseFn = unsafe extern "C" fn(state: *mut c_void);
pub type SlotFn = unsafe extern "C" fn(ctx: *mut c_void, state: *mut *mut c_void, release: *mut Option<ReleaseFn>) -> i32;
pub type EnvironmentFn = unsafe extern "C" fn(ctx: *mut c_void, name: *const u8, len: usize, out: *mut u8, cap: usize, needed: *mut i64) -> i32;
pub type TerminalFn = unsafe extern "C" fn(ctx: *mut c_void, out: *mut dianash_terminal) -> i32;

/// What the shell gives a call: its output streams, paths through the sandbox, cancellation and the per-shell slot.
#[repr(C)]
pub struct dianash_io {
    pub size: u32,
    pub ctx: *mut c_void,
    pub out: OutFn,
    pub err: OutFn,
    pub resolve: ResolveFn,
    pub display: DisplayFn,
    pub cancelled: CancelledFn,
    pub slot: SlotFn,
    pub environment: EnvironmentFn,
    pub terminal: TerminalFn,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct dianash_service {
    pub size: u32,
    pub name: dianash_bytes,
    pub major: u32,
    pub minor: u32,
    pub table: *const c_void,
}

#[repr(C)]
pub struct dianash_call {
    pub size: u32,
    pub command: dianash_bytes,
    pub argv: *const dianash_bytes,
    pub argc: usize,
    pub stdin_data: dianash_bytes,
    pub stdin_present: u32,
    pub flags: u32,
    pub cwd: dianash_bytes,
    pub user: dianash_bytes,
    pub io: *const dianash_io,
    pub services: *const dianash_service,
    pub service_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct dianash_result {
    pub size: u32,
    pub exit_status: i32,
}

/// A host command's result: a module's, and a request of the host's own that the call raised (`dianash.h`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct dianash_host_result {
    pub size: u32,
    pub exit_status: i32,
    /// Zero for none; otherwise the host's value, given back by `dianash_requests_drain`.
    pub request: u64,
}

pub type HostRunFn = unsafe extern "C" fn(ctx: *mut c_void, call: *const dianash_call, result: *mut dianash_result) -> i32;
pub type HostJobsFn = unsafe extern "C" fn(ctx: *mut c_void, call: *const dianash_call) -> i32;
pub type HostKillFn = unsafe extern "C" fn(ctx: *mut c_void, call: *const dianash_call, id: *const u8, id_len: usize, result: *mut dianash_result) -> i32;

/// How a host builds a shell (`dianash.h`).
#[repr(C)]
pub struct dianash_config {
    pub size: u32,
    pub flags: u32,
    pub root: dianash_bytes,
    pub budget_ms: u32,
}

pub const CONFIG_SESSIONS: u32 = 1 << 0;
pub const CONFIG_INTERACTIVE: u32 = 1 << 1;

/// What `dianash_submit` answers when the line ran.
pub const DONE: i32 = 0;
pub const NEEDS_MORE: i32 = 1;
pub const EXIT: i32 = 2;

/// The commands a host provides in-process: a manifest, and the functions that serve every command it names.
#[repr(C)]
pub struct dianash_host_commands {
    pub size: u32,
    pub manifest: dianash_bytes,
    pub ctx: *mut c_void,
    pub run: Option<HostRunFn>,
    pub jobs: Option<HostJobsFn>,
    pub kill: Option<HostKillFn>,
}
