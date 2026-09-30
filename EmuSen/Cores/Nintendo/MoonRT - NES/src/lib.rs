//! MoonRT, the NES core in Rust, called through the common native interface (EmuSen_NativeCores.md). See Moon_Native.md.

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
