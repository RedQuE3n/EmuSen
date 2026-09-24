//! The debugger's half of the C ABI: the registries pushed down as tables, the observed frame, and the logs drained. See
//! Mercury_Native.md §8.5.

use std::ptr;

use super::{STATUS_ILLEGAL_OPCODE, STATUS_NULL};
use crate::debug::{Call, Hooks, Range, Write};
use crate::machine::Machine;

/// The flags `mercury_debug_set` takes.
pub mod flag {
    pub const CALLS: u32 = 1;
    pub const WRITES: u32 = 2;
    pub const INTERRUPTS: u32 = 4;
    pub const EACH: u32 = 8;
    pub const COVERAGE: u32 = 16;
    pub const PROFILING: u32 = 32;
}

/// The flags, the depth `step over`/`step out` stop at (`i32::MIN` unarmed) and the depth guard (`-1` unarmed), set at once.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_set(machine: *mut Machine, flags: u32, depth_target: i32, depth_guard: i32) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    let on = |bit: u32| flags & bit != 0;
    let hooks = &mut m.hooks;
    hooks.configure(on(flag::CALLS), on(flag::WRITES), on(flag::INTERRUPTS), on(flag::EACH), on(flag::COVERAGE), on(flag::PROFILING));
    hooks.depth_target = depth_target;
    hooks.depth_guard = depth_guard;
}

/// The registry's call stack, `count` targets innermost last, so the depth and the profile's owner here are the registry's.
///
/// # Safety
/// `machine` must be live or null; `targets` valid for `count` values, or null with `count` zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_set_stack(machine: *mut Machine, targets: *const u16, count: usize) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    let values = if targets.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(targets, count) } };
    m.hooks.set_stack(values);
}

/// The enabled breakpoints as `count` pairs of first and last address, as the registry holds them.
///
/// # Safety
/// `machine` must be live or null; `pairs` valid for `2 * count` values, or null with `count` zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_set_breakpoints(machine: *mut Machine, pairs: *const i32, count: usize) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    let values = if pairs.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(pairs, 2 * count) } };
    m.hooks.breakpoints = values.as_chunks::<2>().0.iter().map(|p| (p[0], p[1])).collect();
}

/// The stores to report: `count` triples of space, first and last offset; `kind` 0 the watches, 1 the data breakpoints.
///
/// # Safety
/// `machine` must be live or null; `triples` valid for `3 * count` values, or null with `count` zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_set_ranges(machine: *mut Machine, kind: u32, triples: *const u32, count: usize) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    let values = if triples.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(triples, 3 * count) } };
    let ranges: Vec<Range> = values.as_chunks::<3>().0.iter().map(|t| Range { space: t[0], start: t[1], end: t[2] }).collect();
    let hooks = &mut m.hooks;
    if kind == 0 {
        hooks.watch_ranges = ranges
    } else {
        hooks.break_ranges = ranges
    }
}

/// `RunFrame`'s observed loop: the reasons it stopped for (`debug::stop`), zero at the frame's end, with the program counter it stands
/// at in `pc`; or `STATUS_ILLEGAL_OPCODE` with `detail` as `mercury_machine_run_frame` gives it. `flags` are `machine::RUN_*`.
///
/// # Safety
/// `machine` must be live or null; `pc` and `detail` writable or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_run_frame(machine: *mut Machine, flags: u32, pc: *mut i32, detail: *mut u32) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    let result = m.run_frame_debug(flags);
    if let Some(p) = unsafe { pc.as_mut() } {
        *p = m.cpu.pc as i32;
    }
    match result {
        Ok(why) => why as i32,
        Err(e) => {
            if let Some(d) = unsafe { detail.as_mut() } {
                *d = ((e.opcode as u32) << 16) | e.pc as u32;
            }
            STATUS_ILLEGAL_OPCODE
        }
    }
}

/// The stores logged, four values each (space, offset, value, the storing instruction's address), copied up to `len` values and
/// then forgotten when they all fit; returns how many stores there are.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_writes(machine: *mut Machine, out: *mut u32, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    drain(&mut m.hooks.writes_log, out, len, |w: &Write| [w.space, w.address, w.value, w.pc])
}

/// The calls, dispatches and returns logged, three values each (`debug::Call`), copied and forgotten as `mercury_debug_writes`
/// copies; returns how many there are.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_calls(machine: *mut Machine, out: *mut u32, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    drain(&mut m.hooks.calls_log, out, len, |e: &Call| [e.kind, e.source, e.target])
}

/// A log copied whole into `out` and then cleared, `N` values an entry; a log that does not fit is left for a larger buffer.
fn drain<T, const N: usize>(log: &mut Vec<T>, out: *mut u32, len: usize, fields: impl Fn(&T) -> [u32; N]) -> i64 {
    let count = log.len();
    if !out.is_null() && len >= count * N {
        for (i, entry) in log.iter().enumerate() {
            let values = fields(entry);
            // SAFETY: `out` holds `len` values and `count * N` of them are written, each inside that.
            unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out.add(i * N), N) };
        }
        log.clear();
    }
    count as i64
}

/// The profile's runs, two values each (the routine's address and the instructions charged to it), the open run closed first,
/// copied up to `len` values and forgotten when they all fit; returns how many pairs there are.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_profile(machine: *mut Machine, out: *mut i64, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    let hooks: &mut Hooks = &mut m.hooks;
    hooks.flush();
    let count = hooks.profile.len();
    if !out.is_null() && len >= 2 * count {
        for (i, (owner, instructions)) in std::mem::take(&mut hooks.profile).into_iter().enumerate() {
            // SAFETY: `out` holds `len` values, at least two for each run.
            unsafe {
                *out.add(2 * i) = owner as i64;
                *out.add(2 * i + 1) = instructions;
            }
        }
    }
    count as i64
}

/// The coverage bitmap (one bit an address, the CPU's 64K), copied into `out` and cleared when `len` holds it, with the steps it
/// stands for in `recorded`, which are then forgotten too; returns the bitmap's length, zero while coverage is off.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null; `recorded` writable or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_coverage(machine: *mut Machine, out: *mut u8, len: usize, recorded: *mut i64) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    let Hooks { coverage, covered, .. } = &mut **m.hooks;
    let Some(bits) = coverage.as_deref_mut() else { return 0 };
    if !out.is_null() && len >= bits.len() {
        // SAFETY: `out` holds `len` bytes, at least the bitmap's length.
        unsafe { ptr::copy_nonoverlapping(bits.as_ptr(), out, bits.len()) };
        bits.fill(0);
        if let Some(r) = unsafe { recorded.as_mut() } {
            *r = *covered;
        }
        *covered = 0;
    } else if let Some(r) = unsafe { recorded.as_mut() } {
        *r = *covered;
    }
    bits.len() as i64
}

/// The program counter, the address of the step the machine stands in front of.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_pc(machine: *const Machine) -> i32 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL, |m| m.cpu.pc as i32)
}

/// The call stack's depth as MercuryRT holds it; for tests.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_debug_depth(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.hooks.depth() as i64)
}
