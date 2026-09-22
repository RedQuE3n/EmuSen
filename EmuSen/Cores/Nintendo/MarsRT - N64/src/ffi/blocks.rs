//! The recompiler's switch and counters. See Mars_Native.md §5.8.

use std::ptr;

use crate::cpu::blocks::Tier;
use crate::ffi::Core;

/// Bit 0 turns the recompiler on and bit 1 its verifier; bits 4 to 7 name the tier, 0 the furthest built.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_recompiler(core: *mut Core, flags: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.set_recompiler(flags & 1 != 0);
        c.machine.blocks.verify = flags & 2 != 0 || crate::cpu::blocks::Blocks::verify_by_default();
        c.machine.blocks.tier = Tier::from_number((flags >> 4) & 0xF);
    }
}

/// The counters in `Blocks::counters`' order, copied up to `len`; returns how many there are.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_blocks_counters(core: *const Core, out: *mut i64, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return 0 };
    let values = c.machine.blocks.counters();
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, values.len().min(len)) };
    }
    values.len() as i64
}
