//! The IOP's six timers (ps2tek "IOP Timers"; psx-spx "Timers" for the three the PlayStation had): timers 0 to 2 of 16
//! bits and 3 to 5 of 32, on the IOP's clock, its prescalers, the pixel clock or H-blank, gated by H-blank, V-blank or
//! a stop, with the target and overflow flags and the interrupt request bit's one-shot, repeat, pulse and toggle modes.
//! A timer is a count valid at a time, advanced in closed form; it is evaluated only when it is read, written, at a
//! blanking edge it uses or at the tick its next interrupt is due. Times are EE cycles (`clock`). Sapphire_Native.md
//! §22.6 has the readings.

use crate::pcrtc::timing::Edges;
use crate::clock::{IOP, PIXEL};

pub mod mode {
    pub const GATE: u32 = 1;
    pub const GATM_SHIFT: u32 = 1;
    pub const ZRET: u32 = 1 << 3;
    pub const CMP: u32 = 1 << 4;
    pub const OVF: u32 = 1 << 5;
    pub const REPT: u32 = 1 << 6;
    pub const LEVL: u32 = 1 << 7;
    pub const EXTC: u32 = 1 << 8;
    /// Timer 2's prescaler, the IOP's clock over 8.
    pub const PSCL2: u32 = 1 << 9;
    /// The interrupt request bit ("interrupts enabled" in ps2tek, psx-spx's INTF, 0 meaning a request).
    pub const INTF: u32 = 1 << 10;
    pub const EQUF: u32 = 1 << 11;
    pub const OVFF: u32 = 1 << 12;
    /// Timers 4 and 5's prescaler: 1, 8, 16 or 256.
    pub const PSCL45_SHIFT: u32 = 13;
    /// What a write keeps: bits 0 to 9 and 13 to 14 [A: ps2tek's table, the read-only 10 to 12 apart].
    pub const WRITABLE: u32 = 0x63FF;
    /// What a read returns [C: 15 to 31 "unused/garbage", read as zero].
    pub const READ: u32 = 0x7FFF;
}

/// Each timer's count register; mode and target follow at +4 and +8 (ps2tek's I/O map).
pub const BASES: [u32; 6] = [0x1F80_1100, 0x1F80_1110, 0x1F80_1120, 0x1F80_1480, 0x1F80_1490, 0x1F80_14A0];

/// What a timer counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clock {
    /// A tick every so many EE cycles, the dividers running freely from time 0 [A, as the EE timers' §20.3 6].
    Period(u64),
    /// The pixel clock, `clock::PIXEL`.
    Pixel,
    /// One tick at each H-blank's rise, taken at the edge.
    Hblank,
}

/// The signal a timer's gate follows: H-blank (timer 0), V-blank (timers 1 and 3), or none, where the modes stop or
/// run freely (timers 2, 4 and 5) [R psx-spx for 0 to 2; A for 3 to 5, ps2tek being silent: D-67].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gate {
    H,
    V,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timer {
    pub n: usize,
    pub count: u32,
    pub mode: u32,
    pub target: u32,
    /// The time `count` is valid at.
    synced: u64,
    /// The gate signal's level as last seen, and whether it has risen since the mode was written (gate mode 3).
    gate_high: bool,
    gate_seen: bool,
}

impl Timer {
    pub fn new(n: usize) -> Timer {
        Timer { n, count: 0, mode: mode::INTF, target: 0, synced: 0, gate_high: false, gate_seen: false }
    }

    /// Timer `n` as a reset leaves it at EE cycle `t`: counting from zero from then.
    pub fn reset_at(n: usize, t: u64) -> Timer {
        Timer { synced: t, ..Timer::new(n) }
    }

    /// The largest count: FFFFh on timers 0 to 2, FFFFFFFFh on 3 to 5.
    pub fn max(&self) -> u64 {
        if self.n < 3 { 0xFFFF } else { 0xFFFF_FFFF }
    }

    pub fn clock(&self) -> Clock {
        let ext = self.mode & mode::EXTC != 0;
        match self.n {
            0 if ext => Clock::Pixel,
            1 | 3 if ext => Clock::Hblank,
            2 if self.mode & mode::PSCL2 != 0 => Clock::Period(8 * IOP),
            4 | 5 => Clock::Period(IOP * [1, 8, 16, 256][(self.mode >> mode::PSCL45_SHIFT & 3) as usize]),
            _ => Clock::Period(IOP),
        }
    }

    fn gate(&self) -> Gate {
        match self.n {
            0 => Gate::H,
            1 | 3 => Gate::V,
            _ => Gate::Stop,
        }
    }

    fn gatm(&self) -> u32 {
        self.mode >> mode::GATM_SHIFT & 3
    }

    /// Counting now, by the gate's mode (psx-spx's synchronisation modes): for a blanking gate, mode 0 pauses during
    /// the blank, 1 runs, 2 runs only during it, 3 waits for its first rise; for the others, 0 and 3 stop, 1 and 2 run.
    pub fn counting(&self) -> bool {
        if self.mode & mode::GATE == 0 {
            return true;
        }
        match (self.gate(), self.gatm()) {
            (Gate::Stop, m) => m == 1 || m == 2,
            (_, 0) => !self.gate_high,
            (_, 1) => true,
            (_, 2) => self.gate_high,
            _ => self.gate_seen,
        }
    }

    /// Whether a tick can still change the request bit: an interrupt enabled, and the bit set or toggling.
    fn live(&self) -> bool {
        self.mode & (mode::CMP | mode::OVF) != 0 && (self.mode & mode::INTF != 0 || self.mode & (mode::REPT | mode::LEVL) == mode::REPT | mode::LEVL)
    }

    /// Ticks from `count` until it next equals the target and the maximum, `u64::MAX` for never.
    fn distances(&self) -> (u64, u64) {
        let (c, t, max) = (u64::from(self.count), u64::from(self.target), self.max());
        let zret = self.mode & mode::ZRET != 0;
        let to_target = if c < t {
            t - c
        } else if c == t && zret {
            t + 1
        } else {
            max - c + 1 + t
        };
        let to_max = if zret && t < max && c <= t {
            u64::MAX
        } else if c < max {
            max - c
        } else if zret && t < max {
            u64::MAX
        } else {
            max + 1
        };
        (to_target, to_max)
    }

    /// The count `j` ticks on, when no target or maximum falls within them: the reset after one comes first.
    fn plain(&self, j: u64) -> u32 {
        let c = u64::from(self.count);
        let reset = (self.mode & mode::ZRET != 0 && self.count == self.target) || c == self.max();
        (if j == 0 {
            c
        } else if reset {
            j - 1
        } else {
            c + j
        }) as u32
    }

    /// A tick that brings the count to the target or the maximum: the flags, and the request bit by the modes
    /// (ps2tek: a request needs bit 10; one-shot clears it, toggle inverts it, pulse leaves it); whether a request rose.
    fn event(&mut self, at_target: bool, at_max: bool) -> bool {
        if at_target {
            self.mode |= mode::EQUF;
        }
        if at_max {
            self.mode |= mode::OVFF;
        }
        if !((at_target && self.mode & mode::CMP != 0) || (at_max && self.mode & mode::OVF != 0)) {
            return false;
        }
        let fire = self.mode & mode::INTF != 0;
        if self.mode & mode::REPT == 0 {
            self.mode &= !mode::INTF;
        } else if self.mode & mode::LEVL != 0 {
            self.mode ^= mode::INTF;
        }
        fire
    }

    /// Advances `n` ticks; whether a request rose. Event by event while a request can still rise, in closed form once
    /// none can.
    fn advance(&mut self, mut n: u64) -> bool {
        let mut rose = false;
        while n > 0 {
            let (to_target, to_max) = self.distances();
            let first = to_target.min(to_max);
            if n < first {
                self.count = self.plain(n);
                return rose;
            }
            if !self.live() {
                self.settle(n, to_target, to_max);
                return rose;
            }
            n -= first;
            let value = if to_target <= to_max { self.target } else { self.max() as u32 };
            self.count = value;
            rose |= self.event(first == to_target, first == to_max);
        }
        rose
    }

    /// `n` ticks with no request possible: the flags of every target and maximum on the way, and the count by the
    /// period it falls into after the first.
    fn settle(&mut self, n: u64, to_target: u64, to_max: u64) {
        let (t, max) = (u64::from(self.target), self.max());
        if n >= to_target {
            self.mode |= mode::EQUF;
        }
        if n >= to_max {
            self.mode |= mode::OVFF;
        }
        let first = to_target.min(to_max);
        let at = if to_target <= to_max { t } else { max };
        let r = n - first;
        self.count = if r == 0 {
            at
        } else if self.mode & mode::ZRET != 0 && t < max {
            (r - 1) % (t + 1)
        } else {
            (at + r) % (max + 1)
        } as u32;
    }

    fn ticks(&self, from: u64, to: u64) -> u64 {
        match self.clock() {
            Clock::Period(p) => to / p - from / p,
            Clock::Pixel => pixels(to) - pixels(from),
            Clock::Hblank => 0,
        }
    }

    /// Brings the count to time `t`; whether a request rose.
    pub fn sync(&mut self, t: u64) -> bool {
        let mut rose = false;
        if t > self.synced && self.counting() {
            rose = self.advance(self.ticks(self.synced, t));
        }
        self.synced = self.synced.max(t);
        rose
    }

    /// The time of the tick at which a request could next rise; the timer must be synced.
    pub fn next_event(&self) -> u64 {
        if !self.live() || !self.counting() {
            return u64::MAX;
        }
        let (to_target, to_max) = self.distances();
        let mut k = u64::MAX;
        if self.mode & mode::CMP != 0 {
            k = to_target;
        }
        if self.mode & mode::OVF != 0 {
            k = k.min(to_max);
        }
        if k == u64::MAX {
            return u64::MAX;
        }
        match self.clock() {
            Clock::Period(p) => (self.synced / p).saturating_add(k).saturating_mul(p),
            Clock::Pixel => pixel_time(pixels(self.synced).saturating_add(k)),
            Clock::Hblank => u64::MAX,
        }
    }

    /// Whether this timer needs H-blank's edges: counting them, or gated by H-blank.
    pub fn wants_h(&self) -> bool {
        self.clock() == Clock::Hblank || (self.gate() == Gate::H && self.mode & mode::GATE != 0)
    }

    /// The blanking edges at a time the timer is synced to, with the signals' levels after them: an H-blank tick, and
    /// the gate's resets (mode 1 at the rise, mode 2 at the rise and the fall, D-67) and first rise (mode 3); whether a
    /// request rose.
    pub fn edges(&mut self, e: Edges, h: bool, v: bool) -> bool {
        let mut rose = false;
        if e.h_rise && self.clock() == Clock::Hblank && self.counting() {
            rose = self.advance(1);
        }
        let (rise, fall, level) = match self.gate() {
            Gate::H => (e.h_rise, e.h_fall, h),
            Gate::V => (e.v_rise, e.v_fall, v),
            Gate::Stop => (false, false, false),
        };
        if self.mode & mode::GATE != 0 {
            let m = self.gatm();
            if (rise && (m == 1 || m == 2)) || (fall && m == 2) {
                self.count = 0;
            }
            if rise {
                self.gate_seen = true;
            }
        }
        self.gate_high = level;
        rose
    }

    /// A read at a synced time; the mode's read clears the two flags (ps2tek).
    pub fn read(&mut self, reg: u32) -> u32 {
        match reg {
            0 => self.count,
            4 => {
                let v = self.mode & mode::READ;
                self.mode &= !(mode::EQUF | mode::OVFF);
                v
            }
            8 => self.target,
            _ => 0,
        }
    }

    /// A write at a synced time, `h` and `v` the signals' levels now: the count; the mode, which resets the count and
    /// the flags and sets the request bit; the target, which sets the request bit unless LEVL is set (ps2tek).
    pub fn write(&mut self, reg: u32, value: u32, h: bool, v: bool) {
        let max = self.max() as u32;
        match reg {
            0 => self.count = value & max,
            4 => {
                self.mode = value & mode::WRITABLE | mode::INTF;
                self.count = 0;
                self.gate_seen = false;
                self.gate_high = match self.gate() {
                    Gate::H => h,
                    Gate::V => v,
                    Gate::Stop => false,
                };
            }
            8 => {
                self.target = value & max;
                if self.mode & mode::LEVL == 0 {
                    self.mode |= mode::INTF;
                }
            }
            _ => {}
        }
    }
}

/// Pixel-clock ticks in the first `t` EE cycles.
fn pixels(t: u64) -> u64 {
    (u128::from(t) * u128::from(PIXEL.1) / u128::from(PIXEL.0)) as u64
}

/// The first EE cycle by which `m` pixel ticks have passed.
fn pixel_time(m: u64) -> u64 {
    let (a, b) = (u128::from(PIXEL.0), u128::from(PIXEL.1));
    (u128::from(m) * a).div_ceil(b).min(u128::from(u64::MAX)) as u64
}

/// The register at `paddr`: the timer and its offset (0 count, 4 mode, 8 target).
pub fn decode(paddr: u32) -> Option<(usize, u32)> {
    BASES.iter().position(|&b| (b..b + 12).contains(&paddr)).map(|n| (n, (paddr - BASES[n]) & !3))
}

/// An IOP timer's block of `SPH2` (Sapphire_Native.md §38.2): its registers, the time its count holds at, its gate.
impl emusen_native::State for Timer {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u32s("Registers", &[self.count, self.mode, self.target]);
        w.u64("Synced", self.synced);
        w.bools("Gate", &[self.gate_high, self.gate_seen]);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut g = [0u32; 3];
        r.u32s(&mut g)?;
        [self.count, self.mode, self.target] = g;
        self.synced = r.u64()?;
        let mut b = [false; 2];
        r.bools(&mut b)?;
        [self.gate_high, self.gate_seen] = b;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(t: &mut Timer, ticks: u64) -> bool {
        let p = match t.clock() {
            Clock::Period(p) => p,
            _ => unreachable!(),
        };
        t.sync(t.synced + ticks * p)
    }

    #[test]
    fn sixteen_bit_timers_wrap_at_ffffh_and_thirty_two_bit_ones_do_not() {
        let mut a = Timer::new(0);
        let mut b = Timer::new(3);
        a.write(0, 0xFFF0, false, false);
        b.write(0, 0xFFF0, false, false);
        at(&mut a, 0x20);
        at(&mut b, 0x20);
        assert_eq!(a.count, 0x10);
        assert_ne!(a.mode & mode::OVFF, 0);
        assert_eq!(b.count, 0x1_0010);
        assert_eq!(b.mode & mode::OVFF, 0);
    }

    #[test]
    fn zret_counts_through_the_target_and_a_repeated_pulse_requests_each_time() {
        let mut t = Timer::new(2);
        t.write(4, mode::ZRET | mode::CMP | mode::REPT, false, false);
        t.write(8, 4, false, false);
        assert!(!at(&mut t, 3));
        assert_eq!(t.count, 3);
        assert!(at(&mut t, 1), "the count reaches the target");
        assert_eq!(t.count, 4);
        assert!(!at(&mut t, 1));
        assert_eq!(t.count, 0, "then restarts");
        assert!(at(&mut t, 4));
        assert_ne!(t.mode & mode::INTF, 0, "pulse mode leaves the bit set");
    }

    #[test]
    fn one_shot_clears_the_request_bit_and_toggle_requests_every_second_time() {
        let mut t = Timer::new(4);
        t.write(4, mode::ZRET | mode::CMP, false, false);
        t.write(8, 9, false, false);
        assert!(at(&mut t, 9));
        assert_eq!(t.mode & mode::INTF, 0);
        assert!(!at(&mut t, 10), "one shot");
        assert_eq!(t.next_event(), u64::MAX);
        t.write(4, mode::ZRET | mode::CMP | mode::REPT | mode::LEVL, false, false);
        t.write(8, 9, false, false);
        let fired: Vec<bool> = (0..4).map(|_| at(&mut t, 10)).collect();
        assert_eq!(fired, [true, false, true, false]);
    }

    #[test]
    fn the_settled_count_is_the_events_count() {
        for (zret, target, start, n) in [(true, 5, 0, 1000), (false, 5, 0, 70_000), (true, 5, 9, 70_000), (true, 0xFFFF, 3, 200_000)] {
            let mut a = Timer::new(1);
            a.write(4, if zret { mode::ZRET } else { 0 }, false, false);
            a.write(8, target, false, false);
            a.write(0, start, false, false);
            let mut b = a.clone();
            at(&mut a, n);
            for _ in 0..n {
                at(&mut b, 1);
            }
            assert_eq!((a.count, a.mode), (b.count, b.mode), "zret {zret} target {target} start {start} n {n}");
        }
    }

    #[test]
    fn the_prescalers_and_the_pixel_clock() {
        let mut t = Timer::new(5);
        t.write(4, 3 << mode::PSCL45_SHIFT, false, false);
        t.sync(256 * IOP * 10);
        assert_eq!(t.count, 10);
        let mut p = Timer::new(0);
        p.write(4, mode::EXTC, false, false);
        p.sync(294_912_000);
        assert_eq!(p.count, (13_500_000u64 & 0xFFFF) as u32, "13.5 MHz for a second, wrapped at 16 bits");
    }
}
