//! The YM2612 (OPN2) as the buses see it: the address register and its part, the register file of both parts, the
//! two timers and the status register with its busy flag, channel 6's DAC, and the FM unit (`fm.rs`) each sample. A
//! sample is 144 of the chip's clocks (the 68000's), 1,008 master clocks. The chip shares the Z80's reset line.
//! Nephrite_Native.md §18, §19, §21 and §24 are the record, with the sources each rule comes from.

use emusen_native::{StateReader, StateWriter, Truncated};

use crate::fm::{Fm, Tables};

static TABLES: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();

/// Master clocks a sample, and of one of its 24 slots, six of the chip's input clocks.
pub const SAMPLE: u64 = 144 * 7;
pub const SLOT: u64 = 6 * 7;
/// The busy flag after a data write: from the first slot's edge at or after the write, 33 slots, as the board's
/// ASIC has it (Nephrite_Disputes.md D-13). The slots' edges fall this far into each slot after a sample's deadline
/// (Nephrite_Native.md §24).
pub const BUSY: u64 = 33 * SLOT;
pub const SLOT_EDGE: u64 = 32;
/// The reset line, as the board has it (Nephrite_Disputes.md D-19). The sample cycle restarts at the line's
/// assertion: the first deadline is `RESTART` master clocks after it, its key moment 87, and one every 1,008 from
/// there. A release up to `RELEASE_LEEWAY` after a key moment still starts the envelope's counter with that sample,
/// and the counter's first cycle is the fourth sample from its start.
pub const RESTART: u64 = 87 + KEY_LEAD;
/// The first sample's deadline after power-on, where the reset line starts asserted: where the board's falls against
/// the picture (Nephrite_Disputes.md D-2, D-19), its key moment at 793 (Nephrite_Native.md §24).
pub const POWER_ON: u64 = 793 + KEY_LEAD;
pub const RELEASE_LEEWAY: u64 = 67;
/// A sample's deadline is the moment up to which a write to most registers is in time for it. The key register is
/// taken sooner, a channel at a time, a slot apart: a write keying channel 1 must come `KEY_LEAD` master clocks
/// before the deadline, channel 6's five slots less. A timer's load bit must come `LOAD_LEAD` before it; the timers
/// tick then, and a flag the tick raises reads set from `FLAG_LEAD` before the deadline (Nephrite_Disputes.md D-15,
/// Nephrite_Native.md §24).
pub const KEY_LEAD: u64 = 472;
pub const LOAD_LEAD: u64 = KEY_LEAD - 2 * SLOT;
pub const FLAG_LEAD: [u64; 2] = [KEY_LEAD - 196, KEY_LEAD - 196 - SLOT];
/// Writes that can wait for their sample at once.
const LATE: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ym2612 {
    /// Both parts' registers, `$00`-`$FF`; at power-on all zero but the left and right bits of `$B4`-`$B6`.
    pub regs: [[u8; 256]; 2],
    /// The address register: the address written and the part (0 or 1) of the port it was written through.
    pub address: u8,
    pub part: u8,
    /// Timer A's and timer B's counters, and the samples to timer B's next count.
    pub counter_a: u16,
    pub counter_b: u16,
    pub divider_b: u8,
    /// A load bit has risen and waits for a tick; a tick has asked for the counter's value, which the next slot gives.
    pub load: [bool; 2],
    pub reload: [bool; 2],
    /// `$27` as the timers have it, which is at once.
    pub control: u8,
    /// The deadline of the next sample whose timers have not ticked: they tick `LOAD_LEAD` before it.
    pub timers_next: u64,
    /// The master clock from which each timer's flag, raised by a tick, reads set; none while it is `u64::MAX`.
    pub rise: [u64; 2],
    /// Writes that came too late for the sample then due (its deadline, the part, the register, the value): they
    /// are taken once it is made. Only the key register waits, and a program that writes it faster than the chip
    /// takes it has its oldest key taken at once.
    pub late: Vec<(u64, u8, u8, u8)>,
    /// Bits 0 and 1: timer A's and timer B's overflow flags.
    pub flags: u8,
    /// The master clock until which the busy flag reads set.
    pub busy_until: u64,
    /// The discrete YM2612 of model 1 answers its busy flag at port 0 only; the model 2 ASIC's copy at every port.
    pub discrete: bool,
    /// The master clock of the next sample.
    pub next: u64,
    /// The reset line, the Z80's: while it is held the chip is at rest and takes no write. `held_at` is the first
    /// sample of the cycle its assertion started.
    pub held: bool,
    pub held_at: u64,
    /// The left and right output last made, and each channel's nine-bit output.
    pub out: [i32; 2],
    pub channels: [i32; 6],
    pub fm: Fm,
    /// Each sample's channel outputs, kept while set for comparison with a recording; not part of a state.
    pub trace: Option<Vec<[i32; 6]>>,
}

impl Ym2612 {
    pub fn new(discrete: bool) -> Ym2612 {
        let mut regs = [[0u8; 256]; 2];
        for part in &mut regs {
            part[0xB4..=0xB6].fill(0xC0);
        }
        // The DAC's data stands at the middle of its range, as the board's does before any write (§24).
        regs[0][0x2A] = 0x80;
        Ym2612 { regs, address: 0, part: 0, counter_a: 0, counter_b: 0, divider_b: 0, load: [false; 2], reload: [false; 2], control: 0, timers_next: SAMPLE, rise: [u64::MAX; 2], late: Vec::new(), flags: 0, busy_until: 0, discrete, next: SAMPLE, held: false, held_at: 0, out: [0; 2], channels: [0; 6], fm: Fm::default(), trace: None }
    }

    fn timer_a(&self) -> u16 {
        (self.regs[0][0x24] as u16) << 2 | (self.regs[0][0x25] & 3) as u16
    }

    /// A read of port `port & 3` at master clock `t`: the status register, its busy flag missing outside port 0 on
    /// the discrete chip.
    pub fn read(&self, port: u16, t: u64) -> u8 {
        let busy = t < self.busy_until && (port & 3 == 0 || !self.discrete);
        (busy as u8) << 7 | self.flags | (t >= self.rise[0]) as u8 | ((t >= self.rise[1]) as u8) << 1
    }

    /// The reset line at master clock `t`, the chip already brought up to `t`. Its assertion puts every register,
    /// the timers, the keys and the operators back to their power-on state and restarts the sample cycle from that
    /// moment; its release starts the envelope's counter from 0 (Nephrite_Disputes.md D-19).
    pub fn reset_line(&mut self, held: bool, t: u64) {
        if held && !self.held {
            let fresh = Ym2612::new(self.discrete);
            let trace = self.trace.take();
            *self = Ym2612 { next: t + RESTART, timers_next: t + RESTART, held: true, held_at: t + RESTART, trace, ..fresh };
        } else if !held && self.held {
            // The release is taken as a key is, `KEY_LEAD` before a sample's deadline: the sample whose key moment
            // was the last is the counter's first if the release is within the leeway of that moment, the one after
            // it if not, and the counter's first cycle is the fourth sample from there. `due` is that sample's
            // deadline, which may be past.
            let due = self.next - (self.next + SAMPLE - KEY_LEAD - t - 1) / SAMPLE * SAMPLE;
            let with_last = due >= self.held_at && t + KEY_LEAD - due <= RELEASE_LEEWAY;
            let first = if with_last { due } else { due + SAMPLE };
            (self.fm.eg_counter, self.fm.eg_div) = (0, ((first + 3 * SAMPLE - self.next) / SAMPLE) as u8);
        }
        self.held = held;
    }

    /// A write of `v` to port `port & 3` at master clock `t`, the chip already brought up to `t`: an address
    /// through port 0 or 2, which also names the part; data through either data port, to that part.
    pub fn write(&mut self, port: u16, v: u8, t: u64) {
        if self.held {
            return;
        }
        if port & 1 == 0 {
            self.address = v;
            self.part = (port >> 1 & 1) as u8;
            return;
        }
        let into_slot = (t as i64 - (self.next + SLOT_EDGE) as i64).rem_euclid(SLOT as i64) as u64;
        self.busy_until = t + (SLOT - into_slot) % SLOT + BUSY;
        let (part, a) = (self.part as usize, self.address as usize);
        // The chip's own registers, $21-$2F, are part 0's only.
        if part == 1 && (0x21..=0x2F).contains(&a) {
            return;
        }
        if part == 0 && a == 0x27 {
            // The timers have their own moment, and take the control register at once.
            let was = self.control;
            self.settle(t);
            self.flags &= !(v >> 4 & 3);
            self.load = [self.load[0] || (v & 1 != 0 && was & 1 == 0), self.load[1] || (v & 2 != 0 && was & 2 == 0)];
            self.control = v;
        }
        // A write that comes within its register's lead of the sample due, or behind one still waiting for the
        // same register, waits until that sample is made.
        let same = |w: &(u64, u8, u8, u8)| w.1 == part as u8 && w.2 == a as u8 && (a != 0x28 || w.3 & 7 == v & 7);
        if self.next < t + Self::lead(a, v) || self.late.iter().any(same) {
            if self.late.len() == LATE {
                let (_, p, r, value) = self.late.remove(0);
                self.apply(p as usize, r as usize, value);
            }
            self.late.push((self.next, part as u8, a as u8, v));
        } else {
            self.apply(part, a, v);
        }
    }

    /// How long before a sample's deadline a write to register `a` must come to be in time for it: a channel's key
    /// is taken a slot after the last channel's; every other register is taken at once (Nephrite_Disputes.md D-15).
    fn lead(a: usize, v: u8) -> u64 {
        match a {
            0x28 => KEY_LEAD - ((v as u64 & 7) / 4 * 3 + (v as u64 & 3)).min(5) * SLOT,
            _ => 0,
        }
    }

    /// A write's effect on the registers and the FM unit.
    fn apply(&mut self, part: usize, a: usize, v: u8) {
        match a {
            0x28 => self.fm.key(v),
            0xA0..=0xA6 | 0xA8..=0xAE => {
                self.fm.frequency(&mut self.regs, part, a, v);
                return;
            }
            _ => {}
        }
        self.regs[part][a] = v;
    }

    /// The flags whose rise has come by `t` join the status.
    fn settle(&mut self, t: u64) {
        for i in 0..2 {
            if t >= self.rise[i] {
                self.flags |= 1 << i;
                self.rise[i] = u64::MAX;
            }
        }
    }

    /// The timer slots of the sample whose deadline is `at`: whether CSM is keyed. A tick comes once a sample, timer
    /// B's every sixteenth; a load or an overflow at a tick asks for the counter's value, which the next slot gives,
    /// and at the sample's own tick either keys CSM. The test register's bit 2 makes every slot a tick for both
    /// timers, so that a period is its count of slots and one more, and CSM is keyed only where an overflow falls on
    /// the sample's own tick (Nephrite_Native.md §24).
    fn timers(&mut self, at: u64) -> bool {
        if self.held {
            return false;
        }
        let control = self.control;
        let fast = self.regs[0][0x21] & 4 != 0;
        self.divider_b = (self.divider_b + 1) & 15;
        let values = [self.timer_a(), self.regs[0][0x26] as u16];
        let mut csm = false;
        for slot in 0..if fast { 24 } else { 2 } {
            for i in 0..2 {
                let counter = if i == 0 { &mut self.counter_a } else { &mut self.counter_b };
                let tick = fast || (slot == 0 && (i == 0 || self.divider_b == 0));
                if self.reload[i] {
                    (self.reload[i], *counter) = (false, values[i]);
                } else if slot == 0 && self.load[i] {
                    (self.load[i], self.reload[i]) = (false, true);
                    csm |= i == 0;
                } else if tick && control >> i & 1 != 0 {
                    *counter += 1;
                    if *counter >= [1024, 256][i] {
                        self.reload[i] = true;
                        if control >> (2 + i) & 1 != 0 {
                            self.rise[i] = self.rise[i].min(at.saturating_sub(FLAG_LEAD[i]) + slot * SLOT);
                        }
                        csm |= i == 0 && slot == 0;
                    }
                }
            }
        }
        csm
    }

    /// The DAC's level: `$2A` with `$2C` bit 3 below it, as a signed nine-bit value.
    pub fn dac_level(&self) -> i32 {
        (((self.regs[0][0x2A] as i32) << 1) | (self.regs[0][0x2C] >> 3 & 1) as i32) - 256
    }

    /// Channel 6's DAC, where `$2B` enables it.
    pub fn dac(&self) -> Option<i32> {
        (self.regs[0][0x2B] & 0x80 != 0).then(|| self.dac_level())
    }

    /// One sample: the timers count, then each channel's nine-bit output goes to the sides its `$B4` register names.
    fn sample(&mut self) {
        if self.held {
            (self.channels, self.out) = ([0; 6], [0; 2]);
            if let Some(t) = &mut self.trace {
                t.push(self.channels);
            }
            return;
        }
        let tables = TABLES.get_or_init(Tables::new);
        self.channels = self.fm.sample(&self.regs, tables);
        if let Some(d) = self.dac() {
            self.channels[5] = d;
        }
        // The test register `$2C`'s bit 5 puts the DAC's level on every channel, whether `$2B` enables the DAC or not.
        if self.regs[0][0x2C] & 0x20 != 0 {
            self.channels = [self.dac_level(); 6];
        }
        if let Some(t) = &mut self.trace {
            t.push(self.channels);
        }
        let mut out = [0i32; 2];
        for ch in 0..6 {
            let (part, i) = (ch / 3, ch % 3);
            let value = self.channels[ch];
            let pan = self.regs[part][0xB4 + i];
            if pan & 0x80 != 0 {
                out[0] += value;
            }
            if pan & 0x40 != 0 {
                out[1] += value;
            }
        }
        self.out = out;
    }

    /// Every sample whose time has come by `t`, each output handed to `level` with the sample's master clock.
    pub fn run(&mut self, t: u64, mut level: impl FnMut(u64, [i32; 2])) {
        loop {
            if self.timers_next <= self.next && self.timers_next < t + LOAD_LEAD {
                // The timers of the sample due tick once no load can still reach them, and key CSM in that sample.
                let at = self.timers_next;
                self.settle(at.saturating_sub(SAMPLE));
                self.fm.csm = self.timers(at);
                self.timers_next += SAMPLE;
            } else if self.next <= t {
                let at = self.next;
                self.sample();
                level(at, self.out);
                self.next += SAMPLE;
                let waiting = std::mem::take(&mut self.late);
                for (due, part, a, v) in waiting {
                    if due <= at {
                        self.apply(part as usize, a as usize, v);
                    } else {
                        self.late.push((due, part, a, v));
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Part0", &self.regs[0]);
        w.bytes("Part1", &self.regs[1]);
        w.bytes("AddressPartFlags", &[self.address, self.part, self.flags, self.divider_b, self.discrete as u8, self.held as u8]);
        w.u16s("Counters", &[self.counter_a, self.counter_b]);
        w.u64s("Times", &[self.busy_until, self.next, self.held_at]);
        w.i32s("Out", &self.out);
        w.i32s("Channels", &self.channels);
        w.bytes("Timers", &[self.load[0] as u8, self.load[1] as u8, self.reload[0] as u8, self.reload[1] as u8, self.control, self.late.len() as u8]);
        let mut times = vec![self.timers_next, self.rise[0], self.rise[1]];
        for k in 0..LATE {
            let (due, part, r, v) = self.late.get(k).copied().unwrap_or_default();
            times.extend_from_slice(&[due, (part as u64) << 16 | (r as u64) << 8 | v as u64]);
        }
        w.u64s("TimersAndLate", &times);
        self.fm.write_state(w);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        r.bytes(&mut self.regs[0])?;
        r.bytes(&mut self.regs[1])?;
        let mut b = [0u8; 6];
        r.bytes(&mut b)?;
        (self.address, self.part, self.flags, self.divider_b, self.discrete, self.held) = (b[0], b[1], b[2], b[3], b[4] != 0, b[5] != 0);
        let mut c = [0u16; 2];
        r.u16s(&mut c)?;
        (self.counter_a, self.counter_b) = (c[0], c[1]);
        let mut t = [0u64; 3];
        r.u64s(&mut t)?;
        (self.busy_until, self.next, self.held_at) = (t[0], t[1], t[2]);
        r.i32s(&mut self.out)?;
        r.i32s(&mut self.channels)?;
        let mut b = [0u8; 6];
        r.bytes(&mut b)?;
        (self.load, self.reload, self.control) = ([b[0] != 0, b[1] != 0], [b[2] != 0, b[3] != 0], b[4]);
        let mut times = [0u64; 3 + 2 * LATE];
        r.u64s(&mut times)?;
        (self.timers_next, self.rise) = (times[0], [times[1], times[2]]);
        self.late = (0..(b[5] as usize).min(LATE)).map(|k| {
            let packed = times[4 + 2 * k];
            (times[3 + 2 * k], (packed >> 16) as u8, (packed >> 8) as u8, packed as u8)
        }).collect();
        self.fm.read_state(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(y: &mut Ym2612, part: u16, a: u8, v: u8, t: u64) {
        y.write(part * 2, a, t);
        y.write(part * 2 + 1, v, t);
    }

    /// The address register keeps the part it was written through: data through either port goes there.
    #[test]
    fn a_data_write_goes_to_the_part_the_address_was_written_through() {
        let mut y = Ym2612::new(true);
        y.write(0, 0x30, 0);
        y.write(3, 0x11, 0);
        y.write(2, 0x31, 0);
        y.write(1, 0x22, 0);
        assert_eq!((y.regs[0][0x30], y.regs[1][0x30], y.regs[1][0x31], y.regs[0][0x31]), (0x11, 0, 0x22, 0));
        set(&mut y, 1, 0x2A, 0x99, 0);
        assert_eq!((y.regs[0][0x2A], y.regs[1][0x2A]), (0x80, 0), "the chip's own registers are part 0's, the DAC's at its middle");
    }

    /// The busy flag (D-13): from the first slot's edge at or after a data write, 33 slots, the edges falling 22 master
    /// clocks into each slot after a sample's deadline; and only at port 0 on the discrete chip.
    #[test]
    fn the_busy_flag_runs_33_slots_from_the_next_slot_edge_and_reads_at_port_0_on_the_discrete_chip() {
        let mut y = Ym2612::new(true);
        let edge = y.next + SLOT_EDGE;
        y.write(0, 0x2A, edge - 5);
        assert_eq!(y.read(0, edge - 4), 0, "an address write does not set it");
        for (at, until) in [(edge - 5, edge + BUSY), (edge, edge + BUSY), (edge + 1, edge + SLOT + BUSY)] {
            y.write(1, 0x80, at);
            assert_eq!((y.read(0, until - 1), y.read(0, until)), (0x80, 0), "a write {} from an edge", at as i64 - edge as i64);
        }
        assert_eq!((y.read(1, edge + 2), y.read(2, edge + 2), y.read(3, edge + 2)), (0, 0, 0));
        let mut asic = Ym2612::new(false);
        asic.write(1, 0, 0);
        assert_eq!((asic.read(1, 1), asic.read(3, 1)), (0x80, 0x80));
    }

    /// Timer A overflows after 1,024 less its value in samples, timer B after 256 less its value in 16 samples.
    #[test]
    fn the_timers_overflow_at_their_periods_and_their_flags_clear_by_reset() {
        let mut y = Ym2612::new(true);
        set(&mut y, 0, 0x24, 0xFF, 0);
        set(&mut y, 0, 0x25, 0x00, 0);
        set(&mut y, 0, 0x26, 0xFE, 0);
        set(&mut y, 0, 0x27, 0x0F, 0);
        let (mut a_at, mut b_at) = (Vec::new(), Vec::new());
        for n in 1..=200u64 {
            let t = n * SAMPLE;
            y.run(t, |_, _| {});
            if y.read(0, t) & 1 != 0 {
                a_at.push(n);
                set(&mut y, 0, 0x27, 0x1F, t);
            }
            if b_at.is_empty() && y.read(0, t) & 2 != 0 {
                b_at.push(n);
            }
        }
        assert_eq!(&a_at[..3], &[5, 9, 13], "timer A at $3FC: loaded at the first tick, four samples a period from the next");
        assert_eq!(b_at[0], 32, "timer B at $FE: loaded at the first tick, counting on every sixteenth");
        set(&mut y, 0, 0x27, 0x2F, 300 * SAMPLE);
        assert_eq!(y.read(0, 300 * SAMPLE) & 2, 0);
        set(&mut y, 0, 0x27, 0x00, 300 * SAMPLE);
        let c = y.counter_a;
        y.run(400 * SAMPLE, |_, _| {});
        assert_eq!(y.counter_a, c, "a timer not loaded stands still");
    }

    /// The test register's bit 2 counts both timers every slot: a period of `n` slots and one more (§24).
    #[test]
    fn the_test_registers_bit_2_counts_the_timers_by_slots() {
        let mut y = Ym2612::new(true);
        set(&mut y, 0, 0x21, 0x04, 0);
        set(&mut y, 0, 0x26, 0xFF - 46, 0);
        set(&mut y, 0, 0x27, 0x0A, 0);
        y.run(10 * SAMPLE, |_, _| {});
        assert_ne!(y.flags & 2, 0, "timer B of 47 slots overflowed within ten samples");
        let mut y = Ym2612::new(true);
        set(&mut y, 0, 0x26, 0xFF - 46, 0);
        set(&mut y, 0, 0x27, 0x0A, 0);
        y.run(10 * SAMPLE, |_, _| {});
        assert_eq!(y.flags & 2, 0, "without the bit it counts every sixteenth sample");
    }

    /// The reset line (D-19): asserted, the registers and timers go and writes are not taken; the sample cycle
    /// restarts with a key moment 87 master clocks on; released within 67 of a key moment, the envelope's counter
    /// starts with that moment's sample, its first cycle the fourth sample from there, and with the next one if not.
    #[test]
    fn the_reset_line_clears_the_chip_restarts_its_cycle_and_starts_the_envelopes_counter() {
        let mut y = Ym2612::new(true);
        set(&mut y, 0, 0x30, 0x71, 0);
        set(&mut y, 0, 0x27, 0x05, 0);
        y.run(5 * SAMPLE, |_, _| {});
        assert_eq!(y.flags, 0, "timer A at 0 has 1,024 samples to go");
        y.reset_line(true, 100 + 5 * SAMPLE);
        assert_eq!((y.regs[0][0x30], y.regs[0][0x27], y.regs[0][0xB4], y.next), (0, 0, 0xC0, 100 + 5 * SAMPLE + RESTART));
        set(&mut y, 0, 0x30, 0x22, 150 + 5 * SAMPLE);
        assert_eq!(y.regs[0][0x30], 0, "held, it takes no write");
        y.reset_line(false, 160 + 5 * SAMPLE);
        assert_eq!((y.fm.eg_counter, y.fm.eg_div), (0, 3), "released before any sample of the new cycle");
        set(&mut y, 0, 0x30, 0x22, 170 + 5 * SAMPLE);
        assert_eq!(y.regs[0][0x30], 0x22);
        for (after_a_key_moment, wait) in [(RELEASE_LEEWAY, 3), (RELEASE_LEEWAY + 1, 4), (SAMPLE - 1, 3)] {
            let mut y = Ym2612::new(true);
            y.reset_line(true, 10);
            let released = 10 + RESTART - KEY_LEAD + 2 * SAMPLE + after_a_key_moment;
            y.run(released, |_, _| {});
            y.reset_line(false, released);
            assert_eq!(y.fm.eg_div, wait, "released {after_a_key_moment} after a key moment");
        }
    }

    #[test]
    fn the_dac_takes_channel_sixs_place_on_the_sides_its_panning_names() {
        let mut y = Ym2612::new(true);
        set(&mut y, 0, 0x2A, 0xFF, 0);
        y.run(SAMPLE, |_, _| {});
        assert_eq!(y.out, [0, 0], "off until $2B enables it");
        set(&mut y, 0, 0x2B, 0x80, SAMPLE);
        set(&mut y, 0, 0x2C, 0x08, SAMPLE);
        y.run(2 * SAMPLE, |_, _| {});
        assert_eq!(y.out, [255, 255], "the ninth bit below $2A's eight");
        set(&mut y, 1, 0xB6, 0x80, 2 * SAMPLE);
        set(&mut y, 0, 0x2A, 0x00, 2 * SAMPLE);
        set(&mut y, 0, 0x2C, 0x00, 2 * SAMPLE);
        y.run(3 * SAMPLE, |_, _| {});
        assert_eq!(y.out, [-256, 0]);
    }
}
