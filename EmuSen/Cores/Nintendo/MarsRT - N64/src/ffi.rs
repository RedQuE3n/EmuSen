//! MarsRT's C ABI for the machine and its state. A negative return is a `StateError` status. See Mars_Native.md §5.1.

use std::ptr;

use crate::machine::Machine;
use crate::state::StateResult;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = -1;

fn status(result: StateResult<usize>) -> i64 {
    match result {
        Ok(n) => n as i64,
        Err(e) => e.status() as i64,
    }
}

/// # Safety
/// `data` must be valid for `len` bytes, or null with `len` zero.
unsafe fn input<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// A machine with 4 MB or 8 MB of RDRAM, or null for any other size.
#[unsafe(no_mangle)]
pub extern "C" fn mars_machine_new(rdram_bytes: u32) -> *mut Machine {
    Machine::new(rdram_bytes as usize).map_or(ptr::null_mut(), |m| Box::into_raw(Box::new(m)))
}

/// # Safety
/// `machine` must come from `mars_machine_new` and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_free(machine: *mut Machine) {
    if !machine.is_null() {
        drop(unsafe { Box::from_raw(machine) });
    }
}

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_rdram_bytes(machine: *const Machine) -> u32 {
    unsafe { machine.as_ref() }.map_or(0, |m| m.rdram_bytes() as u32)
}

/// Zero, or a negative status; a failed load leaves the machine as it was.
///
/// # Safety
/// `machine` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_load_state(machine: *mut Machine, data: *const u8, len: usize) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    match m.load_state(unsafe { input(data, len) }) {
        Ok(()) => 0,
        Err(e) => e.status(),
    }
}

/// The version of the state last loaded: 1 a state, 2 a snapshot, 0 none.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_state_kind(machine: *const Machine) -> i32 {
    unsafe { machine.as_ref() }.map_or(0, |m| m.loaded_version)
}

/// The bytes a save of this kind takes, or a negative status. `snapshot` nonzero asks for a snapshot.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_save_state_size(machine: *const Machine, snapshot: u32) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    status(m.state_size(snapshot != 0))
}

/// The bytes written, or a negative status.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_save_state(machine: *const Machine, out: *mut u8, len: usize, snapshot: u32) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    status(m.save_state(unsafe { std::slice::from_raw_parts_mut(out, len) }, snapshot != 0))
}

/// The state's layout as UTF-8 text, copied up to `len` bytes; returns its whole length, or a negative status.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_state_layout(machine: *const Machine, snapshot: u32, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    status(m.layout(snapshot != 0).map(|text| {
        if !out.is_null() {
            let n = text.len().min(len);
            unsafe { ptr::copy_nonoverlapping(text.as_ptr(), out, n) };
        }
        text.len()
    }))
}
