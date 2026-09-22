//! A test-only C ABI for the scan-out: a VI and its scan-out behind a handle, fed the C# machine's registers and memory. See Mars_Native.md §5.4.

use std::ptr;

use crate::vi::Vi;
use crate::vi::scan::{self, RASTER_HEIGHT, Scanout};

/// One VI with its scan-out, as the machine will hold them.
#[derive(Default)]
pub struct ViScan {
    vi: Vi,
    out: Scanout,
}

/// A null handle or buffer, or memory whose hidden bits do not cover it.
pub const STATUS_NULL: i32 = -1;
pub const STATUS_SIZE: i32 = -2;

#[unsafe(no_mangle)]
pub extern "C" fn mars_vi_scan_new() -> *mut ViScan {
    Box::into_raw(Box::default())
}

/// # Safety
/// `scan` must come from `mars_vi_scan_new` and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_free(scan: *mut ViScan) {
    if !scan.is_null() {
        drop(unsafe { Box::from_raw(scan) });
    }
}

/// Sets the fourteen registers, and the held lines and blank flag when `held` is not null.
///
/// # Safety
/// `scan` live or null; `registers` valid for 14 words; `held` valid for 625 words, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_set(scan: *mut ViScan, registers: *const u32, held: *const i32, was_blank: u32) -> i32 {
    let Some(s) = (unsafe { scan.as_mut() }) else { return STATUS_NULL };
    if registers.is_null() {
        return STATUS_NULL;
    }
    s.vi.registers.copy_from_slice(unsafe { std::slice::from_raw_parts(registers, 14) });
    if !held.is_null() {
        s.vi.held.copy_from_slice(unsafe { std::slice::from_raw_parts(held, RASTER_HEIGHT) });
        s.vi.was_blank = was_blank != 0;
    }
    0
}

/// Copies out the held lines; returns the blank flag, 0 or 1.
///
/// # Safety
/// `scan` live or null; `held` valid for 625 words.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_held(scan: *const ViScan, held: *mut i32) -> i32 {
    let Some(s) = (unsafe { scan.as_ref() }) else { return STATUS_NULL };
    if held.is_null() {
        return STATUS_NULL;
    }
    unsafe { ptr::copy_nonoverlapping(s.vi.held.as_ptr(), held, RASTER_HEIGHT) };
    s.vi.was_blank as i32
}

/// `RepeatRows` for the compositions that follow.
///
/// # Safety
/// `scan` live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_repeat_rows(scan: *mut ViScan, repeat: u32) {
    if let Some(s) = unsafe { scan.as_mut() } {
        s.out.repeat_rows = repeat != 0;
    }
}

/// One scan and composition: 1 when a walk ran, 0 when none did, or a negative status.
///
/// # Safety
/// `scan` live or null; `rdram` valid for `rdram_len` bytes, `hidden` for `hidden_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_run(scan: *mut ViScan, rdram: *const u8, rdram_len: usize, hidden: *const u8, hidden_len: usize) -> i32 {
    let Some(s) = (unsafe { scan.as_mut() }) else { return STATUS_NULL };
    if rdram.is_null() || hidden.is_null() {
        return STATUS_NULL;
    }
    if hidden_len * 2 != rdram_len {
        return STATUS_SIZE;
    }
    let rdram = unsafe { std::slice::from_raw_parts(rdram, rdram_len) };
    let hidden = unsafe { std::slice::from_raw_parts(hidden, hidden_len) };
    scan::scan(&mut s.vi, rdram, hidden, &mut s.out) as i32
}

/// The last frame's width, height and row repeat into `shape`, and up to `len` of its bytes into `out`; returns its whole length.
///
/// # Safety
/// `scan` live or null; `shape` valid for 3 words, or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_frame(scan: *const ViScan, shape: *mut u32, out: *mut u8, len: usize) -> i64 {
    let Some(s) = (unsafe { scan.as_ref() }) else { return STATUS_NULL as i64 };
    if !shape.is_null() {
        let values = [s.out.width, s.out.height, s.out.row_repeat];
        unsafe { ptr::copy_nonoverlapping(values.as_ptr(), shape, 3) };
    }
    unsafe { copy_out(&s.out.frame, out, len) }
}

/// Up to `len` bytes of the raster, 640 by 625 with the coverage byte; returns its whole length.
///
/// # Safety
/// `scan` live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_vi_scan_raster(scan: *const ViScan, out: *mut u8, len: usize) -> i64 {
    let Some(s) = (unsafe { scan.as_ref() }) else { return STATUS_NULL as i64 };
    unsafe { copy_out(s.out.raster(), out, len) }
}

/// # Safety
/// `out` valid for `len` bytes, or null.
unsafe fn copy_out(bytes: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len().min(len)) };
    }
    bytes.len() as i64
}
