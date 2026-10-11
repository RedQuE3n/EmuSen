//! The three root counters (psx-spx "Timers"): 16-bit counts on the system clock, the dot clock, hblank or the system
//! clock over 8, with their sync modes against the blanks, their targets and their pulse or toggle interrupts. Each is
//! brought to the present lazily from its source's tick count (Diamond_Native.md §11.6).

use crate::video::Video;

/// What a timer counts, by its mode's clock bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    System,
    System8,
    Dot,
    Hblank,
}

pub mod mode {
    pub const SYNC: u32 = 1 << 0;
    pub const ZRET: u32 = 1 << 3;
    pub const CMP: u32 = 1 << 4;
    pub const OVFL: u32 = 1 << 5;
    pub const REPT: u32 = 1 << 6;
    pub const LEVL: u32 = 1 << 7;
    pub const INTF: u32 = 1 << 10;
    pub const EQUF: u32 = 1 << 11;
    pub const OVFF: u32 = 1 << 12;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timer {
    pub count: u32,
    pub target: u32,
    pub mode: u32,
    /// The source's tick count the count was last brought to, and the ticks still to pass before the next increment
    /// (the counter holds after a reset, psx-spx "Reset and Wrap").
    pub(crate) ticks: u64,
    pub(crate) hold: u64,
    /// Whether the next tick resets the counter, having reached its target with ZRET or FFFFh, and whether it then
    /// holds at zero a further tick (a target's reset does, FFFFh's does not).
    pub(crate) wrap: bool,
    pub(crate) wrap_hold: bool,
    /// Sync mode 3 has seen its blank; one-shot mode has fired.
    pub(crate) seen_blank: bool,
    pub(crate) fired: bool,
    /// Whether the counter runs, by the sync mode and the blank's level.
    pub running: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Timers {
    pub t: [Timer; 3],
}

impl Timer {
    pub fn source(&self, n: usize) -> Source {
        match (n, self.mode & 1 << 8 != 0, self.mode & 1 << 9 != 0) {
            (0, true, _) => Source::Dot,
            (1, true, _) => Source::Hblank,
            (2, _, true) => Source::System8,
            _ => Source::System,
        }
    }

    fn gate(&self) -> u32 {
        self.mode >> 1 & 3
    }
}

pub fn ticks(source: Source, video: &Video, t: u64) -> u64 {
    match source {
        Source::System => t,
        Source::System8 => t / 8,
        Source::Dot => video.dots(t),
        Source::Hblank => video.hblanks(t),
    }
}

/// The first CPU cycle at which `source` has counted `k` ticks.
fn time_of(source: Source, video: &Video, k: u64) -> u64 {
    match source {
        Source::System => k,
        Source::System8 => k * 8,
        Source::Dot => video.t_of_dots(k),
        Source::Hblank => video.t_of_hblanks(k),
    }
}

impl Timers {
    pub fn new() -> Timers {
        let mut t = Timers::default();
        for x in &mut t.t {
            x.running = true;
            x.mode = mode::INTF;
        }
        t
    }

    /// Brings timer `n` to cycle `now`, returning whether it interrupted (an I_STAT edge).
    pub fn sync(&mut self, n: usize, video: &Video, now: u64) -> bool {
        let timer = &mut self.t[n];
        let source = timer.source(n);
        let at = ticks(source, video, now);
        let mut elapsed = at.saturating_sub(timer.ticks);
        timer.ticks = at;
        if !timer.running {
            return false;
        }
        let mut irq = false;
        while elapsed > 0 {
            if timer.hold > 0 {
                let h = timer.hold.min(elapsed);
                timer.hold -= h;
                elapsed -= h;
                continue;
            }
            if timer.wrap {
                timer.wrap = false;
                timer.count = 0;
                elapsed -= 1;
                timer.hold = timer.wrap_hold as u64 * holds(n, timer);
                continue;
            }
            let goal = if timer.count < timer.target { timer.target } else { 0xFFFF };
            let step = (goal - timer.count) as u64;
            if elapsed < step {
                timer.count += elapsed as u32;
                break;
            }
            timer.count = goal;
            elapsed -= step;
            irq |= hit(timer);
        }
        irq
    }

    /// The CPU cycle at which timer `n` next reaches its target or FFFFh, while it runs as it does now.
    pub fn next_hit(&self, n: usize, video: &Video) -> u64 {
        let timer = &self.t[n];
        if !timer.running {
            return u64::MAX;
        }
        let mut wait = timer.hold;
        let mut count = timer.count;
        if timer.wrap {
            wait += 1 + timer.wrap_hold as u64 * holds(n, timer);
            count = 0;
        }
        let goal = if count < timer.target { timer.target } else { 0xFFFF };
        let ticks_needed = wait + (goal - count) as u64;
        if ticks_needed == 0 {
            return u64::MAX;
        }
        let source = timer.source(n);
        let t = time_of(source, video, timer.ticks + ticks_needed);
        if source == Source::Hblank && t == u64::MAX { u64::MAX } else { t }
    }

    /// A write to the mode register: the count and the flags reset, the interrupt request returns high.
    pub fn write_mode(&mut self, n: usize, video: &Video, now: u64, value: u32, blank: bool) {
        let timer = &mut self.t[n];
        timer.mode = value & 0x3FF | mode::INTF;
        timer.wrap_hold = false;
        timer.count = 0;
        timer.wrap = false;
        timer.fired = false;
        timer.seen_blank = false;
        timer.ticks = ticks(timer.source(n), video, now);
        timer.hold = holds(n, timer);
        timer.running = running(n, timer, blank);
    }

    pub fn write_count(&mut self, n: usize, value: u32) {
        let timer = &mut self.t[n];
        timer.count = value & 0xFFFF;
        timer.hold = holds(n, timer);
        timer.wrap = false;
    }

    pub fn write_target(&mut self, n: usize, value: u32) {
        self.t[n].target = value & 0xFFFF;
    }

    /// Reading the mode clears the reached flags.
    pub fn read_mode(&mut self, n: usize) -> u32 {
        let m = self.t[n].mode;
        self.t[n].mode &= !(mode::EQUF | mode::OVFF);
        m
    }

    /// Re-reads every timer's source at `now`, after the video timing changed under the dot clock or hblank.
    pub fn rebase(&mut self, video: &Video, now: u64) {
        for n in 0..3 {
            let s = self.t[n].source(n);
            self.t[n].ticks = ticks(s, video, now);
        }
    }

    /// A blank's edge for timer `n`'s sync (hblank for 0, vblank for 1), after the timer was brought to the edge.
    pub fn blank(&mut self, n: usize, start: bool) {
        let timer = &mut self.t[n];
        if timer.mode & mode::SYNC == 0 || n == 2 {
            return;
        }
        let gate = timer.gate();
        // Mode 2 also reads zero outside the blank (ps1-tests' timers on a console, Diamond_Disputes.md D-6).
        if (start && (gate == 1 || gate == 2)) || (!start && gate == 2) {
            timer.count = 0;
            timer.wrap = false;
        }
        if start && gate == 3 {
            timer.seen_blank = true;
        }
        timer.running = running(n, timer, start);
    }
}

/// The extra tick a counter holds at a reset or a written value: psx-spx's two cycles at zero are of the system
/// clock, so a slower source is not held [A].
fn holds(n: usize, timer: &Timer) -> u64 {
    (timer.source(n) == Source::System) as u64
}

/// Whether a timer counts, by its sync mode and the blank's level now.
fn running(n: usize, timer: &Timer, blank: bool) -> bool {
    if timer.mode & mode::SYNC == 0 {
        return true;
    }
    match (n, timer.gate()) {
        (2, g) => g == 1 || g == 2,
        (_, 0) => !blank,
        (_, 1) => true,
        (_, 2) => blank,
        _ => timer.seen_blank,
    }
}

/// The count has just reached the target or FFFFh: the flags, the wrap, and the interrupt by the mode.
fn hit(timer: &mut Timer) -> bool {
    let at_target = timer.count == timer.target;
    let at_max = timer.count == 0xFFFF;
    if at_target {
        timer.mode |= mode::EQUF;
    }
    if at_max {
        timer.mode |= mode::OVFF;
    }
    if at_target && timer.mode & mode::ZRET != 0 {
        timer.wrap = true;
        timer.wrap_hold = true;
    } else if at_max {
        timer.wrap = true;
        timer.wrap_hold = false;
    }
    let wants = (at_target && timer.mode & mode::CMP != 0) || (at_max && timer.mode & mode::OVFL != 0);
    if !wants || (timer.fired && timer.mode & mode::REPT == 0) {
        return false;
    }
    timer.fired = true;
    if timer.mode & mode::LEVL != 0 {
        timer.mode ^= mode::INTF;
        timer.mode & mode::INTF == 0
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system() -> (Timers, Video) {
        (Timers::new(), Video::new(false))
    }

    #[test]
    fn a_target_with_zret_holds_zero_two_cycles() {
        let (mut t, v) = system();
        t.write_mode(2, &v, 0, mode::ZRET, false);
        t.write_target(2, 1);
        let mut seen = Vec::new();
        for c in 0..7 {
            t.sync(2, &v, c);
            seen.push(t.t[2].count);
        }
        assert_eq!(seen, [0, 0, 1, 0, 0, 1, 0], "psx-spx's example");
    }

    #[test]
    fn the_target_interrupts_once_or_repeatedly() {
        let (mut t, v) = system();
        t.write_mode(2, &v, 0, mode::ZRET | mode::CMP, false);
        t.write_target(2, 100);
        assert_eq!(t.next_hit(2, &v), 101);
        assert!(t.sync(2, &v, 101));
        assert!(!t.sync(2, &v, 400), "one-shot");
        t.write_mode(2, &v, 400, mode::ZRET | mode::CMP | mode::REPT, false);
        t.write_target(2, 100);
        let first = t.next_hit(2, &v);
        assert!(t.sync(2, &v, first));
        let second = t.next_hit(2, &v);
        assert_eq!(second - first, 102, "the target, then two cycles at zero");
        assert!(t.sync(2, &v, second));
        assert_eq!(t.read_mode(2) & mode::EQUF, mode::EQUF);
        assert_eq!(t.read_mode(2) & mode::EQUF, 0, "the flag clears when read");
    }

    #[test]
    fn toggle_mode_interrupts_every_second_time() {
        let (mut t, v) = system();
        t.write_mode(2, &v, 0, mode::OVFL | mode::REPT | mode::LEVL, false);
        let a = t.next_hit(2, &v);
        assert!(t.sync(2, &v, a), "INTF 1 to 0 is the edge; the first toggle takes it to 0");
        assert_eq!(t.t[2].mode & mode::INTF, 0);
        let b = t.next_hit(2, &v);
        assert_eq!(b - a, 0x10000, "FFFFh, then zero for one cycle");
        assert!(!t.sync(2, &v, b), "back to 1: no edge");
    }

    #[test]
    fn system_clock_over_8_and_a_stopped_timer_2() {
        let (mut t, v) = system();
        t.write_mode(2, &v, 0, 1 << 9, false);
        t.sync(2, &v, 8 * 1000);
        assert_eq!(t.t[2].count, 1000, "a slower source is not held");
        t.write_mode(2, &v, 8000, mode::SYNC, false);
        t.sync(2, &v, 100_000);
        assert_eq!(t.t[2].count, 0);
    }
}
