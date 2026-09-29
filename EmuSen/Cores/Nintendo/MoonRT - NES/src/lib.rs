//! MoonRT, the NES core in Rust, called through a C ABI. See Moon_Native.md.

pub mod apu;
pub mod cpu;
pub mod ffi;
pub mod machine;
pub mod memory;
#[cfg(test)]
mod naming;
pub mod ppu;
pub mod state;

use std::cell::Cell;
use std::ffi::{CStr, c_char};
use std::sync::OnceLock;

/// The interface version; C# refuses a library whose number is not the one it was written against.
pub const INTERFACE_VERSION: u32 = 2;

/// A field C# marks `[SkipInState]`; see emusen-native.
pub use emusen_native::Skip;

/// A C# exception the machine would have thrown; Rust records the first and carries on, and the frame reports it. See Moon_Native.md §6.2, D4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Fault {
    IndexOutOfRange = 1,
    DivideByZero = 2,
    ArgumentOutOfRange = 3,
}

thread_local! {
    static FAULT: Cell<Option<Fault>> = const { Cell::new(None) };
}

/// Records the first fault since the last `take_fault`.
#[cold]
pub fn fault(kind: Fault) {
    FAULT.with(|f| {
        if f.get().is_none() {
            f.set(Some(kind));
        }
    });
}

pub fn take_fault() -> Option<Fault> {
    FAULT.with(|f| f.take())
}

/// C#'s `array[index]` on a read: the element, or a recorded `IndexOutOfRangeException` and zero.
#[inline(always)]
pub fn at(array: &[u8], index: i32) -> u8 {
    match array.get(index as usize) {
        Some(&v) => v,
        None => {
            fault(Fault::IndexOutOfRange);
            0
        }
    }
}

/// C#'s `array[index] = value`.
#[inline(always)]
pub fn put(array: &mut [u8], index: i32, value: u8) {
    match array.get_mut(index as usize) {
        Some(v) => *v = value,
        None => fault(Fault::IndexOutOfRange),
    }
}

/// C#'s `a % b` on `int`: truncated, with a recorded `DivideByZeroException` for zero.
#[inline(always)]
pub fn rem(a: i32, b: i32) -> i32 {
    if b == 0 {
        fault(Fault::DivideByZero);
        0
    } else {
        a.wrapping_rem(b)
    }
}

/// C#'s `table[index]` on an `int` table.
#[inline(always)]
pub fn at_i32(table: &[i32], index: i32) -> i32 {
    match table.get(index as usize) {
        Some(&v) => v,
        None => {
            fault(Fault::IndexOutOfRange);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn moon_interface_version() -> u32 {
    INTERFACE_VERSION
}

static CRASH_LOG: OnceLock<String> = OnceLock::new();

/// Where a panic is written before the process aborts, since a panic must never cross into C#. See Mars_Native.md §2.
///
/// # Safety
/// `path` must be a valid NUL-terminated UTF-8 string, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_set_crash_log(path: *const c_char) {
    if path.is_null() {
        return;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    if CRASH_LOG.set(path).is_ok() {
        std::panic::set_hook(Box::new(|info| {
            let text = format!("MoonRT panic: {info}\n{}\n", std::backtrace::Backtrace::force_capture());
            if let Some(path) = CRASH_LOG.get() {
                let _ = std::fs::write(path, &text);
            }
            eprintln!("{text}");
        }));
    }
}
