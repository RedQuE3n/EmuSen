//! The display's clock, read from the times a window's frames were swapped: the period smoothed over the swaps between two
//! looks, and whether a game whose frames come at some interval runs one frame a swap (EmuSen_Qt.md §1.8.3, §1.9). The
//! frame loop that feeds it the swaps keeps it; nothing here waits or sleeps.

/// How far a game's interval may sit from the display's period for one frame to run at each swap.
pub const LOCK_SHARE: f64 = 0.01;

/// The swaps seen so far and the period read from them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DisplayClock {
    /// The last swap seen: its count and its time, steady-clock nanoseconds.
    pub last: Option<(u64, i64)>,
    /// The display's period in nanoseconds, zero until two swaps have been seen.
    pub period_ns: f64,
}

impl DisplayClock {
    /// A swap's count and time seen; a look that saw several swaps since the last counts each as one period. A count of
    /// zero is no swap, whose time is no sample.
    pub fn saw(&mut self, swaps: u64, at_ns: i64) {
        if swaps == 0 {
            return;
        }
        if let Some((n, t)) = self.last
            && swaps > n
            && at_ns > t
        {
            let one = (at_ns - t) as f64 / (swaps - n) as f64;
            self.period_ns = if self.period_ns == 0.0 { one } else { self.period_ns * 0.9 + one * 0.1 };
        }
        self.last = Some((swaps, at_ns));
    }

    /// Whether a game whose frames come every `interval_ns` runs one frame a swap.
    pub fn locks(&self, interval_ns: u64) -> bool {
        self.period_ns > 0.0 && interval_ns > 0 && ((interval_ns as f64 - self.period_ns).abs() / self.period_ns) <= LOCK_SHARE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_game_locks_to_the_display_only_within_a_hundredth_of_its_period() {
        let mut clock = DisplayClock::default();
        assert!(!clock.locks(16_666_667), "no swap seen yet");
        clock.saw(0, 0);
        let start = 1_000_000_000;
        clock.saw(1, start);
        clock.saw(2, start + 16_666_667);
        assert!((clock.period_ns - 16_666_667.0).abs() < 1.0, "the first two swaps give the period, a look before any none");
        for n in 3..=10u64 {
            clock.saw(n, start + (n - 1) as i64 * 16_666_667);
        }
        assert!((clock.period_ns - 16_666_667.0).abs() < 1.0);
        assert!(clock.locks(16_639_276), "the NES's 60.0988 Hz on a 60 Hz display");
        assert!(!clock.locks(20_000_000), "a 50 Hz game runs on its own clock");
        clock.saw(12, start + 11 * 16_666_667);
        assert!((clock.period_ns - 16_666_667.0).abs() < 1.0, "two swaps between looks are one period each");
        clock.saw(12, start + 12 * 16_666_667);
        assert!((clock.period_ns - 16_666_667.0).abs() < 1.0, "a look with no new swap changes nothing");
    }
}
