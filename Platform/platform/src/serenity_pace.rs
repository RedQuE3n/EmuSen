//! Serenity's display clock through the C interface: the clock is the host's own plain struct, read and written only by
//! these calls, so a frame loop keeps it with no handle to free (EmuSen_RustPlatform.md §51, EmuSen_Qt.md §1.8.3).

use crate::{fail, status};
use emusen_serenity::pace::DisplayClock;

/// emusen_serenity_display_clock: `size` set by the host, the rest zero before the first swap.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    pub size: u32,
    pub seen: u32,
    pub swaps: u64,
    pub at_ns: i64,
    pub period_ns: f64,
}

fn read(c: &Clock) -> DisplayClock {
    DisplayClock { last: (c.seen != 0).then_some((c.swaps, c.at_ns)), period_ns: c.period_ns }
}

fn checked<'a>(clock: *const Clock) -> Result<&'a Clock, i32> {
    // SAFETY: the host passes its own clock, or null.
    let Some(c) = (unsafe { clock.as_ref() }) else { return Err(fail(status::NULL, "no clock")) };
    if (c.size as usize) < std::mem::size_of::<Clock>() {
        return Err(fail(status::BAD_ARGUMENT, "the clock struct is too short"));
    }
    Ok(c)
}

/// A swap's count and its time, steady-clock nanoseconds, seen: 0, or a status.
///
/// # Safety
/// `clock` is null or the host's own clock.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_pace_saw(clock: *mut Clock, swaps: u64, at_ns: i64) -> i32 {
    let mut seen = match checked(clock) {
        Ok(c) => read(c),
        Err(st) => return st,
    };
    seen.saw(swaps, at_ns);
    // SAFETY: checked above.
    let c = unsafe { &mut *clock };
    (c.seen, c.period_ns) = (seen.last.is_some() as u32, seen.period_ns);
    if let Some((n, t)) = seen.last {
        (c.swaps, c.at_ns) = (n, t);
    }
    0
}

/// Whether a game whose frames come every `interval_ns` runs one frame a swap: 1, 0, or a status.
///
/// # Safety
/// `clock` is null or the host's own clock.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_pace_locks(clock: *const Clock, interval_ns: u64) -> i32 {
    match checked(clock) {
        Ok(c) => read(c).locks(interval_ns) as i32,
        Err(st) => st,
    }
}
