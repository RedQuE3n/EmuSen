//! The debugger's hooks across the C ABI: tables pushed down, a frame that ends early with its reasons, and the logs the observed
//! loop kept, drained by the host. Nothing here calls back. See Mars_Native.md §6.5.

use std::ptr;

use crate::cpu::hooks::{COVERAGE_BYTES, Call, Range, Write};
use crate::ffi::{Core, STATUS_NULL};
use crate::rsp::Trace;

/// The flags `mars_debug_set` takes.
pub mod flag {
    pub const CALLS: u32 = 1;
    pub const WRITES: u32 = 2;
    pub const INTERRUPTS: u32 = 4;
    pub const EACH: u32 = 8;
    pub const COVERAGE: u32 = 16;
    pub const RSP_COVERAGE: u32 = 32;
    pub const PROFILING: u32 = 64;
}

/// The flags, the depth `step over`/`step out` stop at (`i32::MIN` unarmed) and the depth guard (`-1` unarmed), set at once.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_set(core: *mut Core, flags: u32, depth_target: i32, depth_guard: i32) {
    let Some(c) = (unsafe { core.as_mut() }) else { return };
    let on = |bit: u32| flags & bit != 0;
    let hooks = &mut c.machine.cpu.hooks;
    hooks.configure(on(flag::CALLS), on(flag::WRITES), on(flag::INTERRUPTS), on(flag::EACH), on(flag::COVERAGE), on(flag::PROFILING));
    hooks.depth_target = depth_target;
    hooks.depth_guard = depth_guard;
    let trace = &mut c.machine.bus.sp.trace;
    if on(flag::RSP_COVERAGE) != trace.is_some() {
        **trace = on(flag::RSP_COVERAGE).then(Trace::new);
    }
}

/// The enabled breakpoints as `count` pairs of first and last address, signed as the registry holds them.
///
/// # Safety
/// `core` must be live or null; `pairs` valid for `2 * count` values, or null with `count` zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_set_breakpoints(core: *mut Core, pairs: *const i32, count: usize) {
    let Some(c) = (unsafe { core.as_mut() }) else { return };
    let values = if pairs.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(pairs, 2 * count) } };
    c.machine.cpu.hooks.breakpoints = values.as_chunks::<2>().0.iter().map(|p| (p[0], p[1])).collect();
}

/// The stores to report: `count` triples of space, first and last address; `kind` 0 the watches, 1 the data breakpoints.
///
/// # Safety
/// `core` must be live or null; `triples` valid for `3 * count` values, or null with `count` zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_set_ranges(core: *mut Core, kind: u32, triples: *const u32, count: usize) {
    let Some(c) = (unsafe { core.as_mut() }) else { return };
    let values = if triples.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(triples, 3 * count) } };
    let ranges: Vec<Range> = values.as_chunks::<3>().0.iter().map(|t| Range { space: t[0], start: t[1], end: t[2] }).collect();
    let hooks = &mut c.machine.cpu.hooks;
    if kind == 0 { hooks.watch_ranges = ranges } else { hooks.break_ranges = ranges }
}

/// `RunFrame`'s observed loop: the reasons it stopped for (`hooks::stop`), zero at the field's end, and where it stands in `pc`.
/// Bit 0 of `flags` runs the first instruction unchecked, as the one a halt stopped in front of runs; bit 1 continues the frame the
/// last call stopped inside rather than beginning a frame.
///
/// # Safety
/// `core` must be live or null; `pc` valid for one value, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_run_frame(core: *mut Core, flags: u32, pc: *mut u64) -> u32 {
    let Some(c) = (unsafe { core.as_mut() }) else { return 0 };
    let why = c.machine.run_frame_debug(flags & 1 != 0, flags & 2 != 0);
    if !pc.is_null() {
        unsafe { *pc = c.machine.cpu.pc };
    }
    why
}

/// The stores logged, four values each (space, address, value, the storing instruction's address), copied up to `len` values and
/// then forgotten when they all fit; returns how many stores there are.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_writes(core: *mut Core, out: *mut u32, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL as i64 };
    let log = &mut c.machine.cpu.hooks.writes_log;
    drain(log, out, len, |w: &Write| [w.space, w.address, w.value, w.pc])
}

/// The calls and returns logged, three values each (1 for a push with its source and target, 0 for a pop), copied and forgotten as
/// `mars_debug_writes` copies; returns how many there are.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_calls(core: *mut Core, out: *mut u32, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL as i64 };
    let log = &mut c.machine.cpu.hooks.calls_log;
    drain(log, out, len, |e: &Call| [e.kind, e.source, e.target])
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

/// The profile's runs, two values each (the routine's address and the instructions charged to it), copied up to `len` values and
/// forgotten when they all fit; returns how many pairs there are.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_profile(core: *mut Core, out: *mut i64, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL as i64 };
    let hooks = &mut c.machine.cpu.hooks;
    hooks.flush();
    let count = hooks.profile.len();
    if !out.is_null() && len >= count * 2 {
        for (i, (owner, instructions)) in hooks.profile.iter().enumerate() {
            // SAFETY: `out` holds `len` values and `2 * count` are written.
            unsafe { ptr::copy_nonoverlapping([*owner as i64, *instructions].as_ptr(), out.add(i * 2), 2) };
        }
        hooks.profile.clear();
    }
    count as i64
}

/// The coverage bitmap, `which` 0 the processor's (one bit per 24-bit address) and 1 the signal processor's (one per IMEM address),
/// copied when `out` holds it and then cleared, with the instructions recorded since the last copy in `recorded`; returns the
/// bitmap's length in bytes, or zero while that coverage is not armed.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes, or null; `recorded` valid for one value, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_coverage(core: *mut Core, which: u32, out: *mut u8, len: usize, recorded: *mut i64) -> i64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL as i64 };
    let (bits, count): (&mut [u8], &mut i64) = if which == 0 {
        let crate::cpu::hooks::Hooks { coverage, covered, .. } = &mut *c.machine.cpu.hooks;
        let Some(bits) = coverage.as_deref_mut() else { return 0 };
        (bits, covered)
    } else {
        let Some(trace) = c.machine.bus.sp.trace.as_mut() else { return 0 };
        (&mut trace.bits[..], &mut trace.recorded)
    };
    debug_assert!(which != 0 || bits.len() == COVERAGE_BYTES);
    if !out.is_null() && len >= bits.len() {
        // SAFETY: `out` holds `len` bytes, at least the bitmap's length.
        unsafe { ptr::copy_nonoverlapping(bits.as_ptr(), out, bits.len()) };
        bits.fill(0);
        if !recorded.is_null() {
            unsafe { *recorded = *count };
        }
        *count = 0;
    } else if !recorded.is_null() {
        unsafe { *recorded = *count };
    }
    bits.len() as i64
}

/// The call stack's depth, its unmatched returns, and the exceptions entered, in that order.
///
/// # Safety
/// `core` must be live or null; `out` valid for three values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_debug_counters(core: *const Core, out: *mut i64) {
    let Some(c) = (unsafe { core.as_ref() }) else { return };
    if out.is_null() {
        return;
    }
    let hooks = &c.machine.cpu.hooks;
    let values = [hooks.depth() as i64, hooks.unmatched_returns, c.machine.cpu.run.exceptions];
    unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, values.len()) };
}

/// The processor's program counter: the instruction it is about to run.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_pc(core: *const Core) -> u64 {
    unsafe { core.as_ref() }.map_or(0, |c| c.machine.cpu.pc)
}

/// `MarsDebugSpaces.TryPhysical`: kernel mode's view of a 32-bit virtual address; returns 1 and the physical address, or 0 where
/// the TLB has no entry.
///
/// # Safety
/// `core` must be live or null; `physical` valid for one value, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_physical(core: *const Core, address: u32, physical: *mut u32) -> u32 {
    let Some(c) = (unsafe { core.as_ref() }) else { return 0 };
    match c.physical(address) {
        Some(p) => {
            if !physical.is_null() {
                unsafe { *physical = p };
            }
            1
        }
        None => 0,
    }
}

/// A test's way to a device register: `MemoryBus.Write32` at a physical address, as the WiseMan tests program the VI.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_bus_write32(core: *mut Core, physical: u32, value: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.bus.write32(physical, value);
    }
}

/// A test's way to read a device register: `MemoryBus.Read32` at a physical address, side effects and all, as the tests read the semaphore.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_bus_read32(core: *mut Core, physical: u32) -> u32 {
    unsafe { core.as_mut() }.map_or(0, |c| c.machine.bus.read32(physical))
}

/// A test's way to a COP0 register, with everything C# derives after a write outside an instruction.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_cop0(core: *mut Core, register: u32, value: u64) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.cpu.cop0[(register & 31) as usize] = value;
        c.machine.cpu.cop0_written(&c.machine.bus);
    }
}

/// A test's way to an interrupt: the sources unmasked and raised at the MI, as `Mi.Mask` and `Mi.Raise` are set by hand.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_mi_raise(core: *mut Core, sources: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.bus.mi.mask |= sources as i32;
        c.machine.bus.mi.raise(sources as i32);
    }
}

/// A test's way to run the signal processor by itself: `Rsp.Start` at `pc` when `start` is set, then `Rsp.Step` `steps` times.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_rsp_step(core: *mut Core, start: u32, pc: u32, steps: u32) {
    let Some(c) = (unsafe { core.as_mut() }) else { return };
    if start != 0 {
        c.machine.bus.sp.processor.start(pc);
    }
    for _ in 0..steps {
        if !c.machine.bus.sp.processor.halted {
            c.machine.bus.rsp_step_one();
        }
    }
}
