//! Serenity's display clock (`emusen_serenity_pace_*`, EmuSen_Qt.md §1.8.3): the swaps' count and times a window
//! reports, the period read from them, and whether a game's frames run one a swap. The clock is the caller's plain
//! struct; the library reads and writes it.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;
use std::sync::Arc;

/// The clock a frame loop keeps, fed each swap it sees.
pub struct DisplayClock {
    platform: Arc<Platform>,
    clock: raw::emusen_serenity_display_clock,
}

impl DisplayClock {
    /// A clock that has seen no swap.
    pub fn new(platform: &Arc<Platform>) -> DisplayClock {
        let clock = raw::emusen_serenity_display_clock { size: size_of::<raw::emusen_serenity_display_clock>() as u32, ..Default::default() };
        DisplayClock { platform: platform.clone(), clock }
    }

    /// The clock as new, having seen no swap: for a loop whose window stopped swapping, as under a paused game.
    pub fn restart(&mut self) {
        self.clock = raw::emusen_serenity_display_clock { size: self.clock.size, ..Default::default() };
    }

    /// A swap's count and its time, steady-clock nanoseconds, seen.
    pub fn saw(&mut self, swaps: u64, at_ns: i64) -> Result<()> {
        let n = unsafe { self.platform.lib.emusen_serenity_pace_saw(&mut self.clock, swaps, at_ns) };
        self.platform.status(n.into()).map(drop)
    }

    /// Whether a game whose frames come every `interval_ns` runs one frame a swap.
    pub fn locks(&self, interval_ns: u64) -> Result<bool> {
        let n = unsafe { self.platform.lib.emusen_serenity_pace_locks(&self.clock, interval_ns) };
        self.platform.status(n.into()).map(|n| n == 1)
    }

    /// The display's period in nanoseconds, zero until two swaps have been seen.
    pub fn period_ns(&self) -> f64 {
        self.clock.period_ns
    }
}
