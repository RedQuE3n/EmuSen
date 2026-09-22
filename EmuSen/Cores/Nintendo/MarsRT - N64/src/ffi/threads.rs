//! MarsRT's threads over the C ABI: C#'s `ThreadedRdp`, `RdpWorkers`, `DeferredPresentation` and `SkipRepeatedScans`, and the counters a test reads. See Mars_Native.md §5.6.

use std::sync::atomic::Ordering::Relaxed;

use super::Core;

/// Bit 0 runs the RDP on a drain; bit 1 defers the presentation; bit 2 walks a repeated scan; bit 3 checks every byte the drain touches.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_threads(core: *mut Core, flags: u32, workers: u32) {
    let Some(c) = (unsafe { core.as_mut() }) else { return };
    if flags & 8 != 0 {
        c.machine.set_verify_rdp(true);
    }
    c.machine.set_rdp_workers(workers.max(1) as usize);
    c.machine.set_threaded_rdp(flags & 1 != 0);
    let deferred = flags & 2 != 0;
    // Switching deferral off shows the walk in flight at once, as C#'s setter does.
    if !deferred && c.machine.join_presentation(&mut c.scanout) {
        c.frame_serial += 1;
    }
    c.machine.set_deferred(deferred);
    c.scanout.walk_repeats = flags & 4 != 0;
}

/// The values `mars_threads_counters` writes, in order.
pub const COUNTERS: usize = 14 + 2 * crate::memory::dp_threads::site::COUNT;

/// Whether the drain runs, its words, starts and nanoseconds, the machine's waits of each kind, bystanders, narrowed and freed reads, repeated scans, then waits and nanoseconds by site; returns how many.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_threads_counters(core: *const Core, out: *mut i64, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return super::STATUS_NULL as i64 };
    let mut values = vec![0i64; COUNTERS];
    if let Some(t) = c.machine.bus.dp.threads.as_deref() {
        let (n, s) = (&t.counters, t.shared());
        let head = [
            1,
            s.drain_words.load(Relaxed),
            s.drain_starts.load(Relaxed),
            s.drain_nanos.load(Relaxed),
            n.page_waits,
            n.page_wait_nanos,
            n.range_waits,
            n.range_wait_nanos,
            n.joins,
            n.join_nanos,
            n.bystanders,
            n.reads_narrowed,
            n.reads_freed,
        ];
        values[..head.len()].copy_from_slice(&head);
        let sites = crate::memory::dp_threads::site::COUNT;
        values[14..14 + sites].copy_from_slice(&n.waits_per_site);
        values[14 + sites..].copy_from_slice(&n.nanos_per_site);
    }
    values[13] = c.scanout.repeated_scans;
    if !out.is_null() {
        let n = len.min(COUNTERS);
        unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), out, n) };
    }
    COUNTERS as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::memory::bus::RDRAM_SIZE;

    #[test]
    fn the_switches_start_and_stop_the_drain_and_the_counters_say_so() {
        let core = Box::into_raw(Box::new(Core::new(Machine::new(RDRAM_SIZE).unwrap())));
        let mut values = [0i64; COUNTERS];
        unsafe {
            mars_machine_set_threads(core, 1 | 2, 1);
            let c = &*core;
            assert!(c.machine.bus.dp.threads.is_some() && c.machine.options.deferred);
            assert_eq!(mars_threads_counters(core, values.as_mut_ptr(), values.len()), COUNTERS as i64);
            assert_eq!(values[0], 1);
            mars_machine_set_threads(core, 0, 1);
            let c = &*core;
            assert!(c.machine.bus.dp.threads.is_none() && !c.machine.options.deferred);
            mars_threads_counters(core, values.as_mut_ptr(), values.len());
            assert_eq!(values[0], 0);
            drop(Box::from_raw(core));
        }
    }
}
