//! MercuryRT's C ABI. A negative return is a status. See Mercury_Native.md §2.3.

use std::ptr;

use crate::machine::Machine;
use crate::memory::cartridge::RomError;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = -1;
/// An image shorter than the 336-byte header.
pub const STATUS_TOO_SHORT: i32 = -9;
/// A cartridge type no board here implements.
pub const STATUS_UNSUPPORTED_BOARD: i32 = -10;

unsafe fn input<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

fn status(result: Result<usize, crate::state::StateError>) -> i64 {
    match result {
        Ok(n) => n as i64,
        Err(e) => e.status() as i64,
    }
}

/// `MercuryCore.LoadRom` from an image; `save_path_len` negative is C#'s null. Null on refusal, with the reason in `status`.
///
/// # Safety
/// `rom` valid for `len` bytes; `save_path` valid for `save_path_len` bytes when that is not negative; `status` writable or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_new(rom: *const u8, len: usize, save_path: *const u8, save_path_len: isize, status: *mut i32) -> *mut Machine {
    let image = unsafe { input(rom, len) }.to_vec();
    let path = (save_path_len >= 0).then(|| String::from_utf8_lossy(unsafe { input(save_path, save_path_len as usize) }).into_owned());
    let (machine, code) = match Machine::load_rom(image, path) {
        Ok(m) => (Box::into_raw(Box::new(m)), 0),
        Err(RomError::TooShort(_)) => (ptr::null_mut(), STATUS_TOO_SHORT),
        Err(RomError::UnsupportedType(_)) => (ptr::null_mut(), STATUS_UNSUPPORTED_BOARD),
    };
    if let Some(s) = unsafe { status.as_mut() } {
        *s = code;
    }
    machine
}

/// # Safety
/// `machine` must come from `mercury_machine_new` and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_free(machine: *mut Machine) {
    if !machine.is_null() {
        drop(unsafe { Box::from_raw(machine) });
    }
}

/// The state's fields; zero, or a negative status, and a failed load changes nothing.
///
/// # Safety
/// `machine` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_load_state(machine: *mut Machine, data: *const u8, len: usize) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    match m.load_state(unsafe { input(data, len) }) {
        Ok(()) => 0,
        Err(e) => e.status(),
    }
}

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_save_state_size(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.state_size() as i64)
}

/// The bytes written, or a negative status.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_save_state(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    status(m.save_state(unsafe { std::slice::from_raw_parts_mut(out, len) }))
}

/// The state's layout as UTF-8 text, copied up to `len` bytes; returns its whole length, or a negative status.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_state_layout(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    let text = m.layout();
    if !out.is_null() {
        let n = text.len().min(len);
        unsafe { ptr::copy_nonoverlapping(text.as_ptr(), out, n) };
    }
    text.len() as i64
}
