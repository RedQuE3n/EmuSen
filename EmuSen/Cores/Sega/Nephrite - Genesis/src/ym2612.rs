//! The YM2612 (OPN2) as the buses see it: the address register and its part, the register file of both parts, the
//! two timers and the status register with its busy flag, channel 6's DAC, and the FM unit (`fm.rs`) each sample. A
//! sample is 144 of the chip's clocks (the 68000's), 1,008 master clocks. Nephrite_Native.md §18 and §19 are the
//! record, with the sources each rule comes from.

use emusen_native::{StateReader, StateWriter, Truncated};

use crate::fm::{Fm, Tables};

static TABLES: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();

/// Master clocks a sample, and the busy flag's span after a data write: 32 of the chip's internal clocks, which are
/// six of its input clocks each (Eke's measurement on the YM2612 and the ASIC alike; Nephrite_Native.md §18).
pub const SAMPLE: u64 = 144 * 7;
pub const BUSY: u64 = 32 * 6 * 7;

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
    /// Bits 0 and 1: timer A's and timer B's overflow flags.
    pub flags: u8,
    /// The master clock until which the busy flag reads set.
    pub busy_until: u64,
    /// The discrete YM2612 of model 1 answers its busy flag at port 0 only; the model 2 ASIC's copy at every port.
    pub discrete: bool,
    /// The master clock of the next sample.
    pub next: u64,
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
        Ym2612 { regs, address: 0, part: 0, counter_a: 0, counter_b: 0, divider_b: 0, flags: 0, busy_until: 0, discrete, next: SAMPLE, out: [0; 2], channels: [0; 6], fm: Fm::default(), trace: None }
    }

    fn timer_a(&self) -> u16 {
        (self.regs[0][0x24] as u16) << 2 | (self.regs[0][0x25] & 3) as u16
    }

    /// A read of port `port & 3` at master clock `t`: the status register, its busy flag missing outside port 0 on
    /// the discrete chip.
    pub fn read(&self, port: u16, t: u64) -> u8 {
        let busy = t < self.busy_until && (port & 3 == 0 || !self.discrete);
        (busy as u8) << 7 | self.flags
    }

    /// A write of `v` to port `port & 3` at master clock `t`, the chip already brought up to `t`: an address
    /// through port 0 or 2, which also names the part; data through either data port, to that part.
    pub fn write(&mut self, port: u16, v: u8, t: u64) {
        if port & 1 == 0 {
            self.address = v;
            self.part = (port >> 1 & 1) as u8;
            return;
        }
        self.busy_until = t + BUSY;
        let (part, a) = (self.part as usize, self.address as usize);
        // The chip's own registers, $21-$2F, are part 0's only.
        if part == 1 && (0x21..=0x2F).contains(&a) {
            return;
        }
        let was = self.regs[part][a];
        match a {
            0x28 => self.fm.key(v),
            0xA0..=0xA6 | 0xA8..=0xAE => {
                self.fm.frequency(&mut self.regs, part, a, v);
                return;
            }
            _ => {}
        }
        self.regs[part][a] = v;
        if part == 0 && a == 0x27 {
            if v & 1 != 0 && was & 1 == 0 {
                self.counter_a = self.timer_a();
            }
            if v & 2 != 0 && was & 2 == 0 {
                self.counter_b = self.regs[0][0x26] as u16;
            }
            self.flags &= !(v >> 4 & 3);
        }
    }

    /// Channel 6's DAC: `$2A` with `$2C` bit 3 below it, as a signed nine-bit value, and whether `$2B` enables it.
    pub fn dac(&self) -> Option<i32> {
        (self.regs[0][0x2B] & 0x80 != 0).then(|| (((self.regs[0][0x2A] as i32) << 1) | (self.regs[0][0x2C] >> 3 & 1) as i32) - 256)
    }

    /// One sample: the timers count, then each channel's nine-bit output goes to the sides its `$B4` register names.
    fn sample(&mut self) {
        let control = self.regs[0][0x27];
        if control & 1 != 0 {
            self.counter_a += 1;
            if self.counter_a >= 1024 {
                self.counter_a = self.timer_a();
                self.flags |= control >> 2 & 1;
            }
        }
        self.divider_b = (self.divider_b + 1) & 15;
        if self.divider_b == 0 && control & 2 != 0 {
            self.counter_b += 1;
            if self.counter_b >= 256 {
                self.counter_b = self.regs[0][0x26] as u16;
                self.flags |= control >> 2 & 2;
            }
        }
        let tables = TABLES.get_or_init(Tables::new);
        self.channels = self.fm.sample(&self.regs, tables);
        if let Some(d) = self.dac() {
            self.channels[5] = d;
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
        while self.next <= t {
            let at = self.next;
            self.sample();
            level(at, self.out);
            self.next += SAMPLE;
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Part0", &self.regs[0]);
        w.bytes("Part1", &self.regs[1]);
        w.bytes("AddressPartFlags", &[self.address, self.part, self.flags, self.divider_b, self.discrete as u8]);
        w.u16s("Counters", &[self.counter_a, self.counter_b]);
        w.u64s("Times", &[self.busy_until, self.next]);
        w.i32s("Out", &self.out);
        w.i32s("Channels", &self.channels);
        self.fm.write_state(w);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        r.bytes(&mut self.regs[0])?;
        r.bytes(&mut self.regs[1])?;
        let mut b = [0u8; 5];
        r.bytes(&mut b)?;
        (self.address, self.part, self.flags, self.divider_b, self.discrete) = (b[0], b[1], b[2], b[3], b[4] != 0);
        let mut c = [0u16; 2];
        r.u16s(&mut c)?;
        (self.counter_a, self.counter_b) = (c[0], c[1]);
        let mut t = [0u64; 2];
        r.u64s(&mut t)?;
        (self.busy_until, self.next) = (t[0], t[1]);
        r.i32s(&mut self.out)?;
        r.i32s(&mut self.channels)?;
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
        assert_eq!((y.regs[0][0x2A], y.regs[1][0x2A]), (0, 0), "the chip's own registers are part 0's");
    }

    #[test]
    fn the_busy_flag_follows_a_data_write_for_192_of_the_68000s_clocks_and_reads_at_port_0_on_the_discrete_chip() {
        let mut y = Ym2612::new(true);
        y.write(0, 0x2A, 1000);
        assert_eq!(y.read(0, 1001), 0, "an address write does not set it");
        y.write(1, 0x80, 1000);
        assert_eq!(y.read(0, 1000 + BUSY - 1), 0x80);
        assert_eq!(y.read(0, 1000 + BUSY), 0);
        assert_eq!((y.read(1, 1001), y.read(2, 1001), y.read(3, 1001)), (0, 0, 0));
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
        let mut a_at = Vec::new();
        let mut b_at = Vec::new();
        let (mut fa, mut fb) = (false, false);
        for n in 1..=200u64 {
            y.run(n * SAMPLE, |_, _| {});
            if !fa && y.flags & 1 != 0 {
                a_at.push(n);
                set(&mut y, 0, 0x27, 0x1F, n * SAMPLE);
            }
            if !fb && y.flags & 2 != 0 {
                b_at.push(n);
                fb = true;
            }
            fa = y.flags & 1 != 0;
        }
        assert_eq!(&a_at[..3], &[4, 8, 12], "timer A at $3FC: four samples");
        assert_eq!(b_at[0] % 16, 0, "timer B counts on the sixteenth sample");
        assert!(b_at[0] <= 32, "timer B at $FE: two counts of sixteen samples, from where the divider stood");
        set(&mut y, 0, 0x27, 0x2F, 300 * SAMPLE);
        assert_eq!(y.flags & 2, 0);
        set(&mut y, 0, 0x27, 0x00, 300 * SAMPLE);
        let c = y.counter_a;
        y.run(400 * SAMPLE, |_, _| {});
        assert_eq!(y.counter_a, c, "a timer not loaded stands still");
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
