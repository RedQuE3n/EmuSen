//! The EE's four timers (EE User's Manual chapter 4; ps2tek "EE Timers"): 16-bit counters on BUSCLK, BUSCLK/16,
//! BUSCLK/256 or H-blank, gated by H-blank or V-blank, with compare and overflow flags whose rising edges reach the
//! INTC. A timer is evaluated only when it is read, written, at a gate or H-blank edge, or at the tick its next flag
//! is due, never cycle by cycle. Sapphire_Native.md §20.5 has the readings the manual leaves open.

use crate::clock::BUS;

pub mod mode {
    pub const CLKS: u32 = 3;
    pub const GATE: u32 = 1 << 2;
    pub const GATS: u32 = 1 << 3;
    pub const GATM_SHIFT: u32 = 4;
    pub const ZRET: u32 = 1 << 6;
    pub const CUE: u32 = 1 << 7;
    pub const CMPE: u32 = 1 << 8;
    pub const OVFE: u32 = 1 << 9;
    pub const EQUF: u32 = 1 << 10;
    pub const OVFF: u32 = 1 << 11;
    pub const WRITABLE: u32 = 0x3FF;
}

/// Which flags rose, for the INTC's edge (ps2tek: an interrupt only when a flag goes from 0 to 1).
pub const ROSE_EQUF: u32 = 1;
pub const ROSE_OVFF: u32 = 2;

pub const BASE: u32 = 0x1000_0000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Timer {
    pub count: u16,
    pub mode: u32,
    pub comp: u16,
    pub hold: u16,
    /// The time `count` is valid at.
    synced: u64,
    /// The selected gate signal's level, as last seen.
    gate_high: bool,
}

impl Timer {
    /// EE cycles between ticks for the bus clock and its dividers; none for H-blank (CLKS 3).
    fn period(&self) -> Option<u64> {
        match self.mode & mode::CLKS {
            0 => Some(BUS),
            1 => Some(16 * BUS),
            2 => Some(256 * BUS),
            _ => None,
        }
    }

    fn gatm(&self) -> u32 {
        self.mode >> mode::GATM_SHIFT & 3
    }

    /// Whether the gate applies: GATE set, and not H-blank while H-blank is the clock (§4.2, GATS).
    fn gated(&self) -> bool {
        self.mode & mode::GATE != 0 && !(self.mode & mode::GATS == 0 && self.mode & mode::CLKS == 3)
    }

    /// The selected gate signal, given both levels.
    fn gate_of(&self, h: bool, v: bool) -> bool {
        if self.mode & mode::GATS != 0 { v } else { h }
    }

    /// Counting now: CUE set, and not held by a low-level gate (GATM 0) whose signal is high.
    pub fn counting(&self) -> bool {
        self.mode & mode::CUE != 0 && !(self.gated() && self.gatm() == 0 && self.gate_high)
    }

    fn settled(&self) -> bool {
        (self.mode & mode::CMPE == 0 || self.mode & mode::EQUF != 0) && (self.mode & mode::OVFE == 0 || self.mode & mode::OVFF != 0)
    }

    /// Advances `n` ticks: the compare flag when the count reaches COMP with CMPE, the overflow flag when it wraps from
    /// FFFFh with OVFE; with ZRET the tick that reaches COMP gives 0 instead (Sapphire_Disputes.md D-38).
    fn advance(&mut self, mut n: u64) -> u32 {
        let mut rose = 0;
        let zret = self.mode & mode::ZRET != 0;
        let comp = u64::from(self.comp);
        while n > 0 {
            let count = u64::from(self.count);
            if self.settled() {
                self.count = if zret && count < comp {
                    ((count + n) % comp) as u16
                } else if zret && comp > 0 {
                    let to_zero = 0x1_0000 - count;
                    if n < to_zero { (count + n) as u16 } else { ((n - to_zero) % comp) as u16 }
                } else if zret {
                    0
                } else {
                    (count + n) as u16
                };
                return rose;
            }
            let to_match = if comp > count { comp - count } else { 0x1_0000 - count + comp };
            let to_wrap = 0x1_0000 - count;
            let step = to_match.min(to_wrap);
            if n < step {
                self.count = (count + n) as u16;
                return rose;
            }
            n -= step;
            let matched = step == to_match;
            self.count = if matched && zret { 0 } else { (count + step) as u16 };
            if matched && self.mode & mode::CMPE != 0 && self.mode & mode::EQUF == 0 {
                self.mode |= mode::EQUF;
                rose |= ROSE_EQUF;
            }
            if step == to_wrap && !(matched && zret) && self.mode & mode::OVFE != 0 && self.mode & mode::OVFF == 0 {
                self.mode |= mode::OVFF;
                rose |= ROSE_OVFF;
            }
        }
        rose
    }

    /// Brings the count to time `t`; the flags that rose on the way.
    pub fn sync(&mut self, t: u64) -> u32 {
        let mut rose = 0;
        if t > self.synced
            && self.counting()
            && let Some(p) = self.period()
        {
            rose = self.advance(t / p - self.synced / p);
        }
        self.synced = self.synced.max(t);
        rose
    }

    /// The time of the next tick at which a flag would rise, if any; the timer must be synced.
    pub fn next_event(&self) -> u64 {
        let Some(p) = self.period() else { return u64::MAX };
        if !self.counting() || self.settled() {
            return u64::MAX;
        }
        let zret = self.mode & mode::ZRET != 0;
        let (count, comp) = (u64::from(self.count), u64::from(self.comp));
        let to_match = if comp > count { comp - count } else { 0x1_0000 - count + comp };
        let to_wrap = 0x1_0000 - count;
        let mut k = u64::MAX;
        if self.mode & mode::CMPE != 0 && self.mode & mode::EQUF == 0 {
            k = to_match;
        }
        if self.mode & mode::OVFE != 0 && self.mode & mode::OVFF == 0 && !(zret && to_match <= to_wrap) {
            k = k.min(to_wrap);
        }
        if k == u64::MAX { u64::MAX } else { (self.synced / p + k) * p }
    }

    /// Whether this timer needs the H-blank's edges: counting on them, or gated by H-blank.
    pub fn wants_h(&self) -> bool {
        self.mode & mode::CUE != 0 && (self.mode & mode::CLKS == 3 || (self.gated() && self.mode & mode::GATS == 0))
    }

    /// The edges at `t` (rising and falling of H-blank and V-blank) after the timer is synced to `t`: an H-blank tick
    /// for CLKS 3, and the gate's effect, a hold for GATM 0 or a reset for GATM 1 to 3.
    pub fn edges(&mut self, h_rise: bool, h_fall: bool, v_rise: bool, v_fall: bool, h: bool, v: bool) -> u32 {
        let mut rose = 0;
        if h_rise && self.mode & mode::CLKS == 3 && self.counting() {
            rose = self.advance(1);
        }
        if self.gated() {
            let (rise, fall) = if self.mode & mode::GATS != 0 { (v_rise, v_fall) } else { (h_rise, h_fall) };
            if self.mode & mode::CUE != 0 && ((rise && self.gatm() & 1 != 0) || (fall && self.gatm() & 2 != 0)) {
                self.count = 0;
            }
        }
        self.gate_high = self.gate_of(h, v);
        rose
    }

    pub fn read(&mut self, reg: u32) -> u32 {
        match reg {
            0x00 => u32::from(self.count),
            0x10 => self.mode & 0xFFF,
            0x20 => u32::from(self.comp),
            0x30 => u32::from(self.hold),
            _ => 0,
        }
    }

    /// A register write at a synced time; `h` and `v` are the signals' levels now.
    pub fn write(&mut self, reg: u32, value: u32, h: bool, v: bool) {
        match reg {
            0x00 => self.count = value as u16,
            0x10 => {
                let flags = self.mode & (mode::EQUF | mode::OVFF) & !value;
                self.mode = value & mode::WRITABLE | flags;
                self.gate_high = self.gate_of(h, v);
            }
            0x20 => self.comp = value as u16,
            0x30 => self.hold = value as u16,
            _ => {}
        }
    }
}

/// The register at `paddr`: the timer and its offset, for 10000000h + n × 800h (§2.2.1); HOLD on timers 0 and 1 only.
pub fn decode(paddr: u32) -> Option<(usize, u32)> {
    let off = paddr.checked_sub(BASE)?;
    if off >= 0x2000 {
        return None;
    }
    let (n, reg) = ((off / 0x800) as usize, off % 0x800);
    match reg {
        0x00 | 0x10 | 0x20 => Some((n, reg)),
        0x30 if n < 2 => Some((n, reg)),
        _ => None,
    }
}

/// A timer's block of `SPH2` (Sapphire_Native.md §38.2): its registers, the time its count holds at, its gate's level.
impl emusen_native::State for Timer {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u16s("Count", &[self.count, self.comp, self.hold]);
        w.u32("Mode", self.mode);
        w.u64("Synced", self.synced);
        w.bool("GateHigh", self.gate_high);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut c = [0u16; 3];
        r.u16s(&mut c)?;
        [self.count, self.comp, self.hold] = c;
        self.mode = r.u32()?;
        self.synced = r.u64()?;
        self.gate_high = r.bool()?;
        Ok(())
    }
}
