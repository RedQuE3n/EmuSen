//! MarsRT's threads: C#'s `ThreadedRdp`, `RdpWorkers`, `DeferredPresentation` and `SkipRepeatedScans`, and the counters a test reads. See Mars_Native.md §5.6.

use std::sync::atomic::Ordering::Relaxed;

use super::Core;

/// The values `Core::thread_counters` gives, in order.
pub const COUNTERS: usize = 14 + 2 * crate::memory::dp_threads::site::COUNT;

impl Core {
    /// `ThreadedRdp`, `DeferredPresentation`, `SkipRepeatedScans` inverted (a repeated scan walked) and the drain's byte checks, and the
    /// workers, as C#'s setters apply them.
    pub fn set_threads(&mut self, threaded: bool, deferred: bool, walk_repeats: bool, verify: bool, workers: u32) {
        if verify {
            self.machine.set_verify_rdp(true);
        }
        self.machine.set_rdp_workers(workers.max(1) as usize);
        self.machine.set_threaded_rdp(threaded);
        // Switching deferral off shows the walk in flight at once, as C#'s setter does.
        if !deferred && self.machine.join_presentation(&mut self.scanout) {
            self.frame_serial += 1;
        }
        self.machine.set_deferred(deferred);
        self.scanout.walk_repeats = walk_repeats;
    }

    /// Whether the drain runs, its words, starts and nanoseconds, the machine's waits of each kind, bystanders, narrowed and freed
    /// reads, repeated scans, then waits and nanoseconds by site.
    pub fn thread_counters(&self) -> [i64; COUNTERS] {
        let mut values = [0i64; COUNTERS];
        if let Some(t) = self.machine.bus.dp.threads.as_deref() {
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
        values[13] = self.scanout.repeated_scans;
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::memory::bus::RDRAM_SIZE;

    #[test]
    fn the_switches_start_and_stop_the_drain_and_the_counters_say_so() {
        let mut c = Core::new(Machine::new(RDRAM_SIZE).unwrap());
        c.set_threads(true, true, false, false, 1);
        assert!(c.machine.bus.dp.threads.is_some() && c.machine.options.deferred);
        assert_eq!(c.thread_counters()[0], 1);
        c.set_threads(false, false, false, false, 1);
        assert!(c.machine.bus.dp.threads.is_none() && !c.machine.options.deferred);
        assert_eq!(c.thread_counters()[0], 0);
    }
}
